use log::error;
use redis::{
    streams::{StreamId, StreamKey, StreamReadOptions, StreamReadReply},
    Commands, RedisResult,
};
use redlock::{Lock, RedLock};
use rust_wheel::config::{app::app_conf_reader::get_app_config, cache::redis_util::get_redis_conn};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::task;

use crate::{
    model::request::project::edit::edit_proj_nickname::EditProjNickname,
    service::project::project_service::handle_update_nickname,
};
pub fn consume_sys_events() {
    task::spawn_blocking({
        || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(listen_nickname_update());
        }
    });
}

pub async fn listen_nickname_update() {
    let redis_conn_str = env::var("TEXHUB_REDIS_URL").unwrap();
    let mut con = get_redis_conn(redis_conn_str.as_str());
    let stream_key = get_app_config("texhub.sys_events_stream");
    let stream_id = "0";
    loop {
        let rl = RedLock::new(vec![redis_conn_str.as_str()]);
        let lock;
        loop {
            match rl.lock("sys-event-mutex".as_bytes(), 1000) {
                Ok(Some(l)) => {
                    lock = l;
                    break;
                }
                Ok(None) => (),
                Err(e) => panic!("Error communicating with redis: {}", e),
            }
        }
        let options = StreamReadOptions::default().count(1).block(1000).noack();
        let result: RedisResult<StreamReadReply> =
            con.xread_options(&[stream_key.as_str()], &[stream_id], &options);
        if let Err(e) = result.as_ref() {
            error!("read stream failed: {}", e);
            break;
        }
        let stream_reply: StreamReadReply = result.unwrap();
        for sk in stream_reply.clone().keys {
            match sk.key.as_str() {
                "sys:event" => {
                    handle_proj_compile_stream(sk, &rl, &lock).await;
                }
                _ => {
                    error!("not implement");
                }
            }
        }
    }
}

pub async fn handle_proj_compile_stream(sk: StreamKey, rl: &RedLock, lock: &Lock<'_>) {
    for stream_id in sk.clone().ids {
        handle_proj_compile_record(stream_id, rl, lock).await;
    }
}

async fn handle_proj_compile_record(stream_id: StreamId, rl: &RedLock, lock: &Lock<'_>) {
    let param = do_task(&stream_id);
    if param.is_some() {
        handle_update_nickname(&param.unwrap()).await;
    }
    rl.unlock(&lock);
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct Data {
    nickname: String,
    userId: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
struct SysEvent {
    data: Data,
    id: i64,
    msgType: String,
}

fn do_task(stream_id: &StreamId) -> Option<EditProjNickname> {
    let payload: &redis::Value = stream_id.map.get("payload").unwrap();
    let msg = extract_string_value(payload).unwrap();
    let event: Result<SysEvent, serde_json::Error> = serde_json::from_str(&msg);
    if let Err(e) = event {
        error!("parse system event failed,{}",e);
        return None;
    }
    let naked_event = event.unwrap();
    if naked_event.msgType == "NICKNAME_UPDATE" {
        let param: EditProjNickname = EditProjNickname {
            user_id: naked_event.data.userId,
            nickname: naked_event.data.nickname,
        };
        return Some(param);
    } else {
        return None;
    }
}

fn extract_string_value(value: &redis::Value) -> Option<String> {
    if let redis::Value::Data(data) = value {
        Some(String::from_utf8_lossy(&data).into_owned())
    } else {
        None
    }
}
