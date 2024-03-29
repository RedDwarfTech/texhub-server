use super::render_client::construct_headers;
use crate::model::request::file::file_initial_req::FileInitialReq;
use log::error;
use reqwest::Client;
use rust_wheel::{
    config::app::app_conf_reader::get_app_config, model::user::login_user_info::LoginUserInfo,
};

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
