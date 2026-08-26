use super::render_client::construct_headers;
use crate::model::request::file::file_initial_req::FileInitialReq;
use log::error;
use reqwest::Client;
use rust_wheel::{
    config::app::app_conf_reader::get_app_config, model::user::login_user_info::LoginUserInfo,
};
use std::time::Duration;

/// 截断过长的响应体/请求体，避免日志爆炸（保留首尾各 512 字节）。
fn truncate(s: &str, max: usize) -> &str {
    if s.len() <= max {
        s
    } else {
        let tail_start = s.len() - 512;
        &s[..512]
    }
}

/// 编译前通知 texhub-broadcast 将项目所有文件的最新内容强制写盘。
/// 返回 Err 表示 flush 失败，调用方应阻止编译入队，避免使用陈旧内容。
pub async fn flush_project_before_compile(
    project_id: &String,
    file_ids: &Vec<String>,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}{}",
        get_app_config("texhub.y_websocket_api_url"),
        "/doc/flush/project"
    );
    let body = serde_json::json!({
        "project_id": project_id,
        "file_ids": file_ids,
    });
    let body_str = body.to_string();
    let response = client
        .post(&url)
        .headers(construct_headers(&url))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await;
    match response {
        Ok(r) => {
            let status = r.status();
            if !status.is_success() {
                let resp_body = r.text().await.unwrap_or_default();
                let msg = format!(
                    "flush project failed, status: {}, url: {}, project_id: {}, file_count: {}, response: {}",
                    status, url, project_id, file_ids.len(), truncate(&resp_body, 1024)
                );
                error!("{}", msg);
                return Err(msg);
            }
            let resp_text = match r.text().await {
                Ok(t) => t,
                Err(e) => {
                    let msg = format!(
                        "flush project read response body failed, url: {}, project_id: {}, err: {}",
                        url, project_id, e
                    );
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let resp: serde_json::Value = match serde_json::from_str(&resp_text) {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!(
                        "flush project parse response failed, url: {}, project_id: {}, err: {}, response: {}",
                        url, project_id, e, truncate(&resp_text, 1024)
                    );
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let code = resp.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            if code != 200 {
                let msg = format!(
                    "flush project response code: {}, url: {}, project_id: {}, response: {}",
                    code, url, project_id, truncate(&resp_text, 1024)
                );
                error!("{}", msg);
                return Err(msg);
            }
            let failed_files = resp
                .get("result")
                .and_then(|r| r.get("failed"))
                .and_then(|f| f.as_array())
                .map(|arr| arr.len())
                .unwrap_or(0);
            if failed_files > 0 {
                let msg = format!(
                    "flush project partially failed, project_id: {}, failed file count: {}, response: {}",
                    project_id, failed_files, truncate(&resp_text, 1024)
                );
                error!("{}", msg);
                return Err(msg);
            }
            Ok(())
        }
        Err(e) => {
            let msg = format!(
                "flush project request error, url: {}, project_id: {}, file_count: {}, timeout: 15s, request_body: {}, err: {}",
                url, project_id, file_ids.len(), truncate(&body_str, 1024), e
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}

/// 查看历史版本前通知 texhub-broadcast 强制刷新项目的待写历史快照。
/// 具体哪些文件需要 flush 由 texhub-broadcast 侧自行决定（内存节流池 + Redis 待写标记），
/// 不依赖 texhub-server 传递文件列表。
/// 返回 Ok 表示调用成功（部分文件失败不阻塞，仅记日志）。
pub async fn flush_project_history_before_view(
    project_id: &String,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}{}",
        get_app_config("texhub.y_websocket_api_url"),
        "/doc/flush/history"
    );
    let body = serde_json::json!({
        "project_id": project_id,
    });
    let body_str = body.to_string();
    let response = client
        .post(&url)
        .headers(construct_headers(&url))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await;
    match response {
        Ok(r) => {
            let status = r.status();
            if !status.is_success() {
                let resp_body = r.text().await.unwrap_or_default();
                let msg = format!(
                    "flush project history failed, status: {}, url: {}, project_id: {}, response: {}",
                    status, url, project_id, truncate(&resp_body, 1024)
                );
                error!("{}", msg);
                return Err(msg);
            }
            let resp_text = match r.text().await {
                Ok(t) => t,
                Err(e) => {
                    let msg = format!(
                        "flush project history read response body failed, url: {}, project_id: {}, err: {}",
                        url, project_id, e
                    );
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let resp: serde_json::Value = match serde_json::from_str(&resp_text) {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!(
                        "flush project history parse response failed, url: {}, project_id: {}, err: {}, response: {}",
                        url, project_id, e, truncate(&resp_text, 1024)
                    );
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let code = resp.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            if code != 200 {
                let msg = format!(
                    "flush project history response code: {}, url: {}, project_id: {}, response: {}",
                    code, url, project_id, truncate(&resp_text, 1024)
                );
                error!("{}", msg);
                return Err(msg);
            }
            Ok(())
        }
        Err(e) => {
            let msg = format!(
                "flush project history request error, url: {}, project_id: {}, timeout: 15s, request_body: {}, err: {}",
                url, project_id, truncate(&body_str, 1024), e
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}

pub async fn initial_file_request(
    proj_id: &String,
    file_id: &String,
    file_content: &String,
    login_user_info: &LoginUserInfo,
) {
    let client = Client::new();
    let url_path = format!("{}{}{}", "/doc/initial?access_token=", login_user_info.token,"&from=server-initial");
    let url = format!(
        "{}{}",
        get_app_config("texhub.y_websocket_api_url"),
        url_path
    );
    let initial_req: FileInitialReq = FileInitialReq {
        project_id: proj_id.to_string(),
        doc_id: file_id.to_string(),
        file_content: file_content.to_string(),
    };
    let response = client
        .post(&url)
        .headers(construct_headers(&url))
        .json(&initial_req)
        .send()
        .await;
    match response {
        Ok(_r) => {}
        Err(e) => {
            error!("request compile error: {}", e);
        }
    }
}
