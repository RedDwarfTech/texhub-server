use crate::{model::app::conf::github_token_req::GithubTokenReq, service::config::user_config_service::save_user_config};
use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo};

pub async fn save_github_token(
    params: web::Json<GithubTokenReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    save_user_config(&login_user_info.userId, &"GITHUB_TOKEN".to_string(),&params.0.token);
    let res = ApiResponse {
        result: "ok",
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/appconf").route("/github-token", web::put().to(save_github_token)),
    );
}
