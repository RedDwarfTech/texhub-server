use std::{future::Future, sync::OnceLock};

use log::warn;
use rust_wheel::config::app::app_conf_reader::get_app_config;
use tokio::task_local;

task_local! {
    static REQUEST_ID: String;
    static REQUEST_LOG_CONTEXT: RequestLogContext;
}

/// 用于告警日志的请求上下文信息。
#[derive(Clone, Debug, Default)]
pub struct RequestLogContext {
    pub method: Option<String>,
    pub uri: Option<String>,
}

impl RequestLogContext {
    pub fn new(method: impl Into<String>, uri: impl Into<String>) -> Self {
        Self {
            method: Some(method.into()),
            uri: Some(uri.into()),
        }
    }
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

pub async fn with_request_scope<F, T>(
    request_id: String,
    log_context: RequestLogContext,
    f: F,
) -> T
where
    F: Future<Output = T>,
{
    REQUEST_LOG_CONTEXT
        .scope(log_context, async { REQUEST_ID.scope(request_id, f).await })
        .await
}

fn format_log_context(context: Option<&RequestLogContext>) -> String {
    match context {
        Some(ctx) => {
            let method = ctx.method.as_deref().unwrap_or("-");
            let uri = ctx.uri.as_deref().unwrap_or("-");
            format!("method={method}, uri={uri}")
        }
        None => try_get_request_log_context()
            .map(|ctx| format_log_context(Some(&ctx)))
            .unwrap_or_else(|| "method=-, uri=-".to_string()),
    }
}

fn try_get_request_log_context() -> Option<RequestLogContext> {
    REQUEST_LOG_CONTEXT.try_with(|ctx| ctx.clone()).ok()
}

fn request_id_warn_ignore_paths() -> &'static [String] {
    static IGNORE_PATHS: OnceLock<Vec<String>> = OnceLock::new();
    IGNORE_PATHS.get_or_init(|| {
        let configured = get_app_config("texhub.request_id_warn_ignore_paths");
        if configured.is_empty() {
            vec!["/texhub/actuator/liveness".to_string()]
        } else {
            configured
                .split(',')
                .map(str::trim)
                .filter(|path| !path.is_empty())
                .map(str::to_string)
                .collect()
        }
    })
}

fn should_ignore_request_id_warning(context: Option<&RequestLogContext>) -> bool {
    let Some(uri) = context.and_then(|ctx| ctx.uri.as_deref()) else {
        return false;
    };
    let path = uri.split('?').next().unwrap_or(uri);
    request_id_warn_ignore_paths()
        .iter()
        .any(|ignored| path == ignored.as_str())
}

fn generate_request_id(reason: &str, context: Option<&RequestLogContext>, suppress_warn: bool) -> String {
    let id = uuid::Uuid::new_v4().to_string();
    if !suppress_warn {
        warn!(
            "x-request-id auto-generated ({}): {} [{}]",
            reason,
            id,
            format_log_context(context)
        );
    }
    id
}

pub fn extract_request_id(header_value: Option<&str>, context: RequestLogContext) -> String {
    match header_value {
        Some(id) if !id.is_empty() => id.to_string(),
        _ => generate_request_id(
            "missing x-request-id header in incoming request",
            Some(&context),
            should_ignore_request_id_warning(Some(&context)),
        ),
    }
}

/// 出站调用使用的 request-id：优先取当前请求上下文，否则生成新 ID 并告警。
/// `hint_uri` 可在无上下文时补充出站目标 URL，便于定位调用来源。
pub fn outbound_request_id(hint_uri: Option<&str>) -> String {
    try_get_request_id().unwrap_or_else(|| {
        let context = hint_uri.map(|uri| RequestLogContext {
            method: Some("OUTBOUND".to_string()),
            uri: Some(uri.to_string()),
        });
        generate_request_id("no request context for outbound call", context.as_ref(), false)
    })
}
