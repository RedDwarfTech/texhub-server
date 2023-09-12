use super::render_client::construct_headers;
use crate::model::request::file::file_initial_req::FileInitialReq;
use log::error;
use reqwest::Client;
use rust_wheel::config::app::app_conf_reader::get_app_config;

pub async fn initial_file_request(proj_id: &String, file_id: &String) {
    let client = Client::new();
    let url_path = format!("{}", "/y-websocket/file/initial");
    let url = format!(
        "{}{}",
        get_app_config("texhub.y_websocket_api_url"),
        url_path
    );
    let initial_req: FileInitialReq = FileInitialReq {
        project_id: proj_id.to_string(),
        doc_id: file_id.to_string(),
        file_content: "\\documentclass{article}\n\n\\begin{document}\nHello, World!\n\\end{document}".to_string(),
    };
    let response = client
        .post(url)
        .headers(construct_headers())
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
