use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;
use crate::service::tpl::template_service::{get_tpl_list, get_tempalte_by_id};

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

#[derive(serde::Deserialize)]
pub struct TplQueryParams {
    id: i64,
}

pub async fn get_tpl(
    params: web::Query<AppParams>,
) -> impl Responder {
    let docs = get_tpl_list(&params.tag);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_tpl_detail(
    params: web::Query<TplQueryParams>,
) -> impl Responder {
    let docs = get_tempalte_by_id(&params.id);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/tpl")
            .route("/list", web::get().to(get_tpl))
            .route("/detail", web::get().to(get_tpl_detail))
    );
}