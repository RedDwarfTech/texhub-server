use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;
use crate::{service::{project::project_service::create_doc, tpl::template_service::{get_tpl_list, get_tempalte_by_id}}, model::request::doc::tex_doc_req::TexDocReq};

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
            .route("/add", web::post().to(add_doc))
            .route("/detail", web::get().to(get_tpl_detail))
    );
}