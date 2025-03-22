use std::env;
use log::error;
use reqwest::Client;
use rust_wheel::model::response::api_response::ApiResponse;

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

