use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AppParams {
    _tag: String,
}

pub async fn get_demo(_params: web::Query<AppParams>) -> impl Responder {
    let res = ApiResponse {
        result: "ok",
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/collar")
            .route("/list", web::get().to(get_demo)),
    );
}
