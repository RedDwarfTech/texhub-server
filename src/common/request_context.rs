use std::future::Future;

use log::warn;
use tokio::task_local;

task_local! {
    static REQUEST_ID: String;
}

pub fn try_get_request_id() -> Option<String> {
    REQUEST_ID.try_with(|id| id.clone()).ok()
}

pub async fn with_request_id<F, T>(request_id: String, f: F) -> T
where
    F: Future<Output = T>,
{
    REQUEST_ID.scope(request_id, f).await
}

fn generate_request_id(reason: &str) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    warn!("x-request-id auto-generated ({}): {}", reason, id);
    id
}

pub fn extract_request_id(header_value: Option<&str>) -> String {
    match header_value {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => generate_request_id("missing x-request-id header in incoming request"),
    }
}

/// 出站调用使用的 request-id：优先取当前请求上下文，否则生成新 ID 并告警。
pub fn outbound_request_id() -> String {
    try_get_request_id().unwrap_or_else(|| generate_request_id("no request context for outbound call"))
}
