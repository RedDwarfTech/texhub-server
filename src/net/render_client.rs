use log::{error, info};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;

use crate::model::{
    diesel::tex::custom_tex_models::TexProject,
    request::project::tex_compile_project_req::TexCompileProjectReq,
};

pub async fn render_request(params: &TexCompileProjectReq, proj: &TexProject) -> Option<serde_json::Value> {
    let client = Client::new();
    let url_path = format!("{}", "/render/compile/v1/project/");
    let url = format!("{}{}", get_app_config("render.render_api_url"), url_path);
    let file_path = format!("/opt/data/project/{}/{}", &params.project_id, proj.doc_name);
    let out_path = format!("/opt/data/project/{}", &params.project_id);
    let json_data = serde_json::json!({
        "file_path": file_path,
        "out_path": out_path,
        "req_time": params.req_time,
        "project_id": proj.project_id
    });
    let response = client
        .post(url)
        .headers(construct_headers())
        .json(&json_data)
        .send()
        .await;
    match response {
        Ok(r) => {
            let txt = r.text().await;
            match txt {
                Ok(info) => {
                    info!("success:{}",info);
                },
                Err(e) => {
                    error!("parse body error:{}",e);
                },
            }
            return None;
        }
        Err(e) => { 
            error!("request compile error: {}", e); 
            return None;
        },
    }
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
}
