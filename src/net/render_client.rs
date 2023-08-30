use log::{error, info};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;

use crate::{model::{
    diesel::tex::custom_tex_models::TexProject,
    request::project::tex_compile_project_req::TexCompileProjectReq,
}, common::proj::proj_util::get_proj_compile_req};

pub async fn render_request(
    params: &TexCompileProjectReq,
    proj: &TexProject,
) -> Option<serde_json::Value> {
    let client = Client::new();
    let url_path = format!("{}", "/render/compile/v1/project");
    let url = format!("{}{}", get_app_config("texhub.render_api_url"), url_path);
    let req_value = get_proj_compile_req(params, proj);
    let response = client
        .post(url)
        .headers(construct_headers())
        .json(&req_value)
        .send()
        .await;
    match response {
        Ok(r) => {
            if !r.status().is_success() {
                error!(
                    "send compile failed,status:{}, msg: {}",
                    r.status(),
                    r.text().await.unwrap()
                );
                return None;
            }
            let resp: Result<serde_json::Value, reqwest::Error> = r.json().await;
            match resp {
                Ok(content) => {
                    let result = content.get("result").unwrap();
                    info!("compile request result,{}", result);
                    return Some(result.clone());
                }
                Err(e) => {
                    error!("get response failed: {}", e);
                    return None;
                }
            }
        }
        Err(e) => {
            error!("request compile error: {}", e);
            return None;
        }
    }
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let token: String = get_app_config("texhub.x_access_token").to_owned();
    headers.insert("x-access-token", HeaderValue::from_str(&token).unwrap());
    headers.insert("user-id", HeaderValue::from_static("1"));
    headers.insert("app-id", HeaderValue::from_static("1"));
    headers.insert("x-request-id", HeaderValue::from_static("reqwest"));
    headers.insert("device-id", HeaderValue::from_static("reqwest"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}
