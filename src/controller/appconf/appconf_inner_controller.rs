use crate::{
    model::app::conf::{github_token_query::GithubTokenQuery},
    service::config::user_config_service::{get_user_config_by_key},
};
use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::{
    response::api_response::ApiResponse
};

pub async fn get_config_by_key(
    params: web::Query<GithubTokenQuery>
) -> impl Responder {
    let conf = get_user_config_by_key(&params.0.user_id, params.0.key);
    let res = ApiResponse {
        result: conf,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inner-tex/appconf")
            .route("/user-one-config", web::get().to(get_config_by_key)),
    );
}
