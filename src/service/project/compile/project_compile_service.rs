use crate::{
    model::{
        diesel::tex::custom_tex_models::TexCompQueue,
        request::project::tex_compile_queue_log::TexCompileQueueLog,
        response::project::{compile_resp::CompileResp, latest_compile::LatestCompile},
    },
    service::{
        global::proj::proj_util::get_proj_base_dir,
        project::project_service::{get_cached_queue_status, get_proj_latest_pdf},
    },
};
use log::{error, info};
use redis::{
    streams::{StreamReadOptions, StreamReadReply},
    Commands, RedisResult,
};
use rust_wheel::{
    common::util::{net::sse_message::SSEMessage, rd_file_util::get_filename_without_ext},
    config::cache::redis_util::get_redis_conn,
    model::user::login_user_info::LoginUserInfo,
};
use std::io::{BufRead, BufReader};
use std::process::{ChildStdout, Command, Stdio};
use tokio::{sync::mpsc::UnboundedSender, task};

/// Read compile logs from Redis stream and forward as SSE messages.
pub async fn get_redis_comp_log_stream(
    params: &TexCompileQueueLog,
    tx: UnboundedSender<SSEMessage<String>>,
    _login_user_info: &LoginUserInfo,
) -> Result<String, redis::RedisError> {
    // Use spawn_blocking so the sync redis client (and its blocking xread) won't block
    // the async runtime thread. We detect a special end marker `====END====` in the
    // assembled message and exit the loop when it appears. After exit we close the
    // provided `tx` so the SSE stream consumer can finish.
    use std::env;
    let redis_conn_str = env::var("TEXHUB_RENDER_REDIS_URL").unwrap();
    let stream_key = format!("texhub:compile:log:{}", params.project_id);
    let tx_block = tx.clone();
    // clone params we need inside the blocking closure
    let project_id_block = params.project_id.clone();
    let qid_block = params.qid;
    // Move blocking work into a dedicated blocking task
    let stream_key_block = stream_key.clone();
    let redis_conn_str_block = redis_conn_str.clone();

    let join_res = task::spawn_blocking(move || -> Result<(), redis::RedisError> {
        let mut con = get_redis_conn(redis_conn_str_block.as_str());
        // Mask password in connection string for logs
        let mut masked_conn = redis_conn_str_block.clone();
        if let Some(at_idx) = masked_conn.find('@') {
            if let Some(scheme_idx) = masked_conn.find("//") {
                // keep scheme and mask credential part
                let after = &masked_conn[at_idx + 1..];
                let scheme = &masked_conn[..scheme_idx + 2];
                masked_conn = format!("{}***@{}", scheme, after);
            } else {
                masked_conn = "***@...".to_string();
            }
        }
        // Diagnostic: list matching keys in this Redis DB to confirm we're in the expected DB/instance
        match redis::cmd("KEYS")
            .arg(format!("{}*", stream_key_block))
            .query::<Vec<String>>(&mut con)
        {
            Ok(found_keys) => {}
            Err(e) => {
                error!(
                    "get_redis_comp_log_stream: KEYS error for pattern '{}*': {}",
                    stream_key_block, e
                );
            }
        }
        // Start from the beginning of the stream so existing messages are picked up.
        // Using "$" would only deliver new messages appended after subscription.
        let mut last_id_local = "0-0".to_string();
        loop {
            if tx_block.is_closed() {
                // caller unsubscribed, exit loop
                break;
            }
            let options = StreamReadOptions::default().count(10).block(5000);
            // Diagnostic checks: confirm the key exists and stream length
            match redis::cmd("EXISTS")
                .arg(&stream_key_block)
                .query::<i32>(&mut con)
            {
                Ok(v) => {}
                Err(e) => error!(
                    "get_redis_comp_log_stream: EXISTS error for {}: {}",
                    stream_key_block, e
                ),
            }
            match redis::cmd("XLEN")
                .arg(&stream_key_block)
                .query::<i64>(&mut con)
            {
                Ok(len) => {}
                Err(e) => error!(
                    "get_redis_comp_log_stream: XLEN error for {}: {}",
                    stream_key_block, e
                ),
            }
            let result: RedisResult<StreamReadReply> = con.xread_options(
                &[stream_key_block.as_str()],
                &[last_id_local.as_str()],
                &options,
            );
            if let Err(e) = result.as_ref() {
                error!("get_redis_comp_log_stream: read redis stream failed: {}", e);
                // Continue and retry; don't return so transient errors don't terminate the stream
                continue;
            }
            let stream_reply: StreamReadReply = result.unwrap();
            if stream_reply.keys.is_empty() {
                info!(
                    "get_redis_comp_log_stream: xread returned no keys for key={}",
                    stream_key_block
                );
                continue;
            }
            for sk in stream_reply.clone().keys {
                info!(
                    "get_redis_comp_log_stream: got {} ids for key={}",
                    sk.ids.len(),
                    sk.key
                );
                for sid in sk.ids {
                    // Extract message from Redis stream record
                    // Expected format: {"msg": "content", ...other optional fields...}
                    let mut message_content = String::new();

                    // Try to find and parse the "msg" field
                    if let Some(msg_value) = sid.map.get("msg") {
                        if let Ok(msg_str) = redis::from_redis_value::<String>(msg_value) {
                            // Try to parse as JSON first
                            if let Ok(json_obj) =
                                serde_json::from_str::<serde_json::Value>(&msg_str)
                            {
                                if let Some(msg_field) = json_obj.get("msg") {
                                    if let Some(msg_text) = msg_field.as_str() {
                                        message_content = msg_text.to_string();
                                    }
                                }
                            }
                            // If not JSON or no "msg" field found, use the value directly
                            if message_content.is_empty() {
                                message_content = msg_str;
                            }
                        }
                    }

                    // Fallback: if still empty, construct from all fields (for backwards compatibility)
                    if message_content.is_empty() {
                        let mut parts: Vec<String> = Vec::new();
                        for (k, v) in sid.map.iter() {
                            let val_str = match redis::from_redis_value::<String>(v) {
                                Ok(s) => s,
                                Err(_) => format!("{:?}", v),
                            };
                            parts.push(format!("{}:{}", k, val_str));
                        }
                        message_content = parts.join(" |");
                    }

                    let joined = message_content;
                    // Check for end marker first
                    if joined.contains("====END====") {
                        // fetch latest compile and cached queue for final message using a temporary runtime
                        let mut final_latest = LatestCompile::default();
                        let mut final_queue = TexCompQueue::default();
                        match tokio::runtime::Runtime::new() {
                            Ok(rt) => {
                                let project_id_clone = project_id_block.clone();
                                let qid_clone = qid_block;
                                let comp = rt.block_on(async move {
                                    let cr = get_proj_latest_pdf(&project_id_clone, &0).await;
                                    let queue = get_cached_queue_status(qid_clone).await;
                                    (cr, queue)
                                });
                                if let Ok(latest) = comp.0 {
                                    final_latest = latest;
                                }
                                if let Some(q) = comp.1 {
                                    final_queue = q;
                                }
                            }
                            Err(e) => {
                                error!("get_redis_comp_log_stream: create runtime failed: {}", e);
                            }
                        }
                        let end_resp = CompileResp::from((final_latest, final_queue));
                        if let Ok(end_json) = serde_json::to_string(&end_resp) {
                            do_msg_send_sync(&end_json, &tx_block, &"TEX_COMP_END".to_string());
                        }
                        return Ok(());
                    } else {
                        // For regular messages, send plain message content (like get_comp_log_stream does)
                        let msg_with_newline = format!("{}\n", joined);
                        do_msg_send_sync(&msg_with_newline, &tx_block, &"TEX_COMP_LOG".to_string());
                    }
                    last_id_local = sid.id.clone();
                }
            }
        }
        Ok(())
    })
    .await;

    // After the blocking task finishes (normal exit or error), ensure we close the sender
    // so the SSE stream is ended. Drop the sender to close the channel.
    drop(tx);

    match join_res {
        Ok(Ok(())) => Ok("".to_string()),
        Ok(Err(e)) => Err(e),
        Err(join_err) => {
            error!(
                "get_redis_comp_log_stream spawn_blocking panicked: {:?}",
                join_err
            );
            Ok("".to_string())
        }
    }
}

pub async fn get_comp_log_stream(
    params: &TexCompileQueueLog,
    tx: UnboundedSender<SSEMessage<String>>,
    login_user_info: &LoginUserInfo,
) -> Result<String, reqwest::Error> {
    let file_name_without_ext = get_filename_without_ext(&params.file_name);
    let base_compile_dir: String = get_proj_base_dir(&params.project_id);
    let file_path = format!("{}/{}.log", base_compile_dir, file_name_without_ext);
    let mut cmd = Command::new("tail")
        .arg("-n")
        .arg("+1")
        .arg("-f")
        .arg(file_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let log_stdout = cmd.stdout.take().unwrap();
    let reader = std::io::BufReader::new(log_stdout);
    // spawn a blocking task to stream the child stdout without creating a new runtime
    task::spawn_blocking({
        let queue_log_params = params.clone();
        let uid = login_user_info.userId;
        let tx_clone = tx.clone();
        move || {
            comp_log_file_read_blocking(reader, tx_clone, queue_log_params, uid);
        }
    });
    Ok("".to_owned())
}

pub fn comp_log_file_read_blocking(
    reader: BufReader<ChildStdout>,
    tx: UnboundedSender<SSEMessage<String>>,
    params: TexCompileQueueLog,
    uid: i64,
) {
    // Blocking read loop on ChildStdout
    for line in reader.lines() {
        match line {
            Ok(line) => {
                let msg_content = format!("{}\n", line.to_owned());
                if msg_content.contains("====END====") {
                    // when finished, try to fetch latest pdf and queue status synchronously
                    // Note: these helper functions are async; here we try to call their sync wrappers if present,
                    // otherwise we spawn a temporary runtime to run them quickly.
                    let rt = match tokio::runtime::Handle::try_current() {
                        Ok(_) => {
                            // we're in a blocking thread, create a local runtime
                            match tokio::runtime::Runtime::new() {
                                Ok(r) => Some(r),
                                Err(e) => {
                                    error!(
                                        "comp_log_file_read_blocking: create runtime failed: {}",
                                        e
                                    );
                                    None
                                }
                            }
                        }
                        Err(_) => match tokio::runtime::Runtime::new() {
                            Ok(r) => Some(r),
                            Err(e) => {
                                error!("comp_log_file_read_blocking: create runtime failed: {}", e);
                                None
                            }
                        },
                    };
                    if let Some(rt) = rt {
                        let params_clone = params.clone();
                        let uid_clone = uid;
                        let comp = rt.block_on(async move {
                            let cr =
                                get_proj_latest_pdf(&params_clone.project_id, &uid_clone).await;
                            let queue = get_cached_queue_status(params_clone.qid.clone()).await;
                            (cr, queue)
                        });
                        match comp.0 {
                            Ok(latest) => {
                                if let Some(queue) = comp.1 {
                                    let comp_resp = CompileResp::from((latest, queue));
                                    if let Ok(end_json) = serde_json::to_string(&comp_resp) {
                                        do_msg_send_sync(
                                            &end_json,
                                            &tx,
                                            &"TEX_COMP_END".to_string(),
                                        );
                                    }
                                } else {
                                    error!("comp_log_file_read_blocking: failed to get cached queue status");
                                }
                            }
                            Err(err) => {
                                error!("get compile log failed,{}", err);
                            }
                        }
                        // runtime drops here
                        break;
                    } else {
                        error!("comp_log_file_read_blocking: cannot create runtime to fetch compile result");
                        break;
                    }
                } else {
                    do_msg_send_sync(&msg_content.to_string(), &tx, &"TEX_COMP_LOG".to_string());
                }
            }
            Err(e) => {
                error!("comp_log_file_read_blocking: read line error: {}", e);
            }
        }
    }
    drop(tx);
}

pub fn do_msg_send_sync(line: &String, tx: &UnboundedSender<SSEMessage<String>>, msg_type: &str) {
    if tx.is_closed() {
        return;
    }
    let sse_msg: SSEMessage<String> =
        SSEMessage::from_data(line.to_string(), &msg_type.to_string());
    let send_result = tx.send(sse_msg);
    match send_result {
        Ok(_) => {
            log::debug!(
                "do_msg_send_sync: sent message, type={}, len={}",
                msg_type,
                line.len()
            );
        }
        Err(e) => {
            error!("send xelatex compile log facing error: {}", e);
        }
    }
}
