use std::env;
use log::error;
use reqwest::Client;
use rust_wheel::model::response::api_response::ApiResponse;
use tokio::task;

fn get_uniq_id() -> Option<i64> {
    task::block_in_place(|| {
        // This is safe if you're already within a Tokio runtime
        tokio::runtime::Handle::current().block_on(get_snowflake_id())
    })
}

pub fn get_uniq_id_legacy() -> Option<i64> {
    // 构建一个 tokio 运行时： Runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build().unwrap();

    // 使用运行时来调用异步的连接方法
    let inner = rt.block_on(get_snowflake_id());
    inner
}
pub async fn get_snowflake_id() -> Option<i64> {
    let client = Client::new();
    let infra_url = env::var("INFRA_URL").expect("INFRA_URL must be set");
    let url = format!("{}{}", infra_url, "/infra-inner/util/uniqid/gen");
    let resp = client
        .get(format!("{}", url))
        .body("{}")
        .send()
        .await;
    if let Err(e) = resp {
        error!("get user failed: {}", e);
        return None;
    }
    let text_response = resp.unwrap().text().await;
    if let Err(e) = text_response {
        error!("extract text failed: {}", e);
        return None;
    }
    let resp_str = text_response.unwrap_or_default();
    let resp_result = serde_json::from_str::<ApiResponse<i64>>(&resp_str);
    if let Err(pe) = resp_result {
        error!("parse failed: {}, response: {}", pe, &resp_str);
        return None;
    }
    Some(resp_result.unwrap().result)
}

