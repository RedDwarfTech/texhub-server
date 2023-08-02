use actix_web::{HttpResponse, Responder, web};
use rust_wheel::model::response::api_response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

pub async fn get_demo(params: web::Query<AppParams>) -> impl Responder {
    let res = ApiResponse {
        result: "ok",
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/tex/doc").route("/list", web::get().to(get_demo)));
}