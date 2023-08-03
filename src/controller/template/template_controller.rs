use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;
use crate::{service::{doc::doc_service::create_doc, tpl::template_service::get_tpl_list}, model::request::doc::tex_doc_req::TexDocReq};

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
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

pub async fn add_doc(form: web::Json<TexDocReq>) -> impl Responder{
    let d_name = form.doc_name.clone();
    let docs = create_doc(&d_name);
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
            .route("/add", web::post().to(add_doc)),
    );
}