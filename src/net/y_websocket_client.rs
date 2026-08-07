use super::render_client::construct_headers;
use crate::model::request::file::file_initial_req::FileInitialReq;
use log::error;
use reqwest::Client;
use rust_wheel::{
    config::app::app_conf_reader::get_app_config, model::user::login_user_info::LoginUserInfo,
};
use std::time::Duration;

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
    let response = client
        .post(&url)
        .headers(construct_headers(&url))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await;
    match response {
        Ok(r) => {
            if !r.status().is_success() {
                let msg = format!("flush project failed, status: {}", r.status());
                error!("{}", msg);
                return Err(msg);
            }
            let resp: serde_json::Value = match r.json().await {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("parse flush project response failed: {}", e);
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let code = resp.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            if code != 200 {
                let msg = format!("flush project response code: {}", code);
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
                    "flush project partially failed, project_id: {}, failed file count: {}",
                    project_id, failed_files
                );
                error!("{}", msg);
                return Err(msg);
            }
            Ok(())
        }
        Err(e) => {
            let msg = format!("flush project request error: {}", e);
            error!("{}", msg);
            Err(msg)
        }
    }
}

/// 查看历史版本前通知 texhub-broadcast 强制刷新各文件的最新历史快照。
/// 绕过 60s 节流，保证历史面板能展示最新的历史版本。
/// 返回 Ok 表示调用成功（部分文件失败不阻塞，仅记日志）。
pub async fn flush_project_history_before_view(
    project_id: &String,
    file_ids: &Vec<String>,
) -> Result<(), String> {
    let client = Client::new();
    let url = format!(
        "{}{}",
        get_app_config("texhub.y_websocket_api_url"),
        "/doc/flush/history"
    );
    let body = serde_json::json!({
        "project_id": project_id,
        "file_ids": file_ids,
    });
    let response = client
        .post(&url)
        .headers(construct_headers(&url))
        .json(&body)
        .timeout(Duration::from_secs(15))
        .send()
        .await;
    match response {
        Ok(r) => {
            if !r.status().is_success() {
                let msg = format!("flush project history failed, status: {}", r.status());
                error!("{}", msg);
                return Err(msg);
            }
            let resp: serde_json::Value = match r.json().await {
                Ok(v) => v,
                Err(e) => {
                    let msg = format!("parse flush project history response failed: {}", e);
                    error!("{}", msg);
                    return Err(msg);
                }
            };
            let code = resp.get("code").and_then(|c| c.as_i64()).unwrap_or(-1);
            if code != 200 {
                let msg = format!("flush project history response code: {}", code);
                error!("{}", msg);
                return Err(msg);
            }
            Ok(())
        }
        Err(e) => {
            let msg = format!("flush project history request error: {}", e);
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
