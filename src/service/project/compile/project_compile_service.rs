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
use log::{error, info, warn};
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

/// Extract and parse message content from Redis stream entry map
fn extract_message_content(
    stream_entry_map: &std::collections::HashMap<String, redis::Value>,
) -> String {
    let mut message_content = String::new();

    // Log all fields in this stream entry for debugging
    info!(
        "extract_message_content: fields count={}",
        stream_entry_map.len()
    );
    for (k, v) in stream_entry_map.iter() {
        let val_str = match redis::from_redis_value::<String>(v) {
            Ok(s) => s,
            Err(_) => format!("{:?}", v),
        };
        info!(
            "extract_message_content: field key='{}', value='{}'",
            k, val_str
        );
    }

    // Try to find and parse the "msg" field
    if let Some(msg_value) = stream_entry_map.get("msg") {
        match redis::from_redis_value::<String>(msg_value) {
            Ok(msg_str) => {
                info!(
                    "extract_message_content: found 'msg' field, raw value: '{}' (len={})",
                    msg_str,
                    msg_str.len()
                );
                message_content = msg_str;
            }
            Err(decode_err) => {
                error!(
                    "extract_message_content: failed to decode 'msg' field from Redis value: {:?}",
                    decode_err
                );
            }
        }
    } else {
        error!("extract_message_content: 'msg' field not found in stream entry, fallback to all fields");
    }
    if message_content.is_empty() {
        message_content = " ".to_string();
    }
    message_content
}

/// Handle end marker: fetch latest compile and cached queue, send completion response
/// Note: This function must be called from async context, NOT from spawn_blocking
async fn handle_end_marker(project_id: &str, qid: i64, tx: &UnboundedSender<SSEMessage<String>>) {
    let mut final_latest = LatestCompile::default();
    let mut final_queue = TexCompQueue::default();

    let project_id_clone = project_id.to_string();
    let cr = get_proj_latest_pdf(&project_id_clone, &0).await;
    let queue = get_cached_queue_status(qid).await;

    if let Ok(latest) = cr {
        final_latest = latest;
    } else {
        error!("handle_end_marker: get_proj_latest_pdf failed");
    }

    if let Some(q) = queue {
        final_queue = q;
    } else {
        error!("handle_end_marker: get_cached_queue_status failed or returned None");
    }

    let end_resp = CompileResp::from((final_latest, final_queue));
    if let Ok(end_json) = serde_json::to_string(&end_resp) {
        do_msg_send_sync(&end_json, tx, "TEX_COMP_END");
        info!("handle_end_marker: sent end response successfully");
    } else {
        error!("handle_end_marker: failed to serialize end response");
    }
}

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

    let join_res = task::spawn_blocking(move || -> Result<bool, redis::RedisError> {
        let mut con = get_redis_conn(redis_conn_str_block.as_str());

        // Start from the beginning of the stream so existing messages are picked up.
        // Using "$" would only deliver new messages appended after subscription.
        let mut last_id_local = "0-0".to_string();
        loop {
            if tx_block.is_closed() {
                // caller unsubscribed, exit loop
                break;
            }
            let options = StreamReadOptions::default().count(10).block(5000);

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
                    let message_content = extract_message_content(&sid.map);

                    // Check for end marker first
                    if message_content.contains("====END====") {
                        // Return true to signal that we found the end marker
                        return Ok(true);
                    } else {
                        // For regular messages, send plain message content
                        let msg_with_newline = format!("{}\n换行", message_content);
                        do_msg_send_sync(&msg_with_newline, &tx_block, "TEX_COMP_LOG");
                    }
                    last_id_local = sid.id.clone();
                }
            }
        }
        Ok(false)
    })
    .await;

    // After the blocking task finishes, handle the end marker in async context
    // This ensures we have proper async support for database queries
    let should_send_end = match join_res {
        Ok(Ok(found_end)) => found_end,
        Ok(Err(e)) => {
            error!("get_redis_comp_log_stream: redis error: {}", e);
            false
        }
        Err(join_err) => {
            error!(
                "get_redis_comp_log_stream spawn_blocking panicked: {:?}",
                join_err
            );
            false
        }
    };

    // Now handle the end marker in async context
    if should_send_end {
        handle_end_marker(&params.project_id, params.qid, &tx).await;
    }

    // Finally close the sender
    drop(tx);

    Ok("".to_string())
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
                                        do_msg_send_sync(&end_json, &tx, "TEX_COMP_END");
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
                    do_msg_send_sync(&msg_content, &tx, "TEX_COMP_LOG");
                }
            }
            Err(e) => {
                error!("comp_log_file_read_blocking: read line error: {}", e);
            }
        }
    }
    drop(tx);
}

pub fn do_msg_send_sync(line: &str, tx: &UnboundedSender<SSEMessage<String>>, msg_type: &str) {
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
