use crate::{
    model::request::doc::tex_doc_req::TexDocReq,
    service::project::project_service::{create_doc, get_prj_list},
};
use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

pub async fn get_docs(params: web::Query<AppParams>) -> impl Responder {
    let docs = get_prj_list(&params.tag);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn add_doc(form: web::Json<TexDocReq>) -> impl Responder {
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
        web::scope("/tex/project")
            .route("/list", web::get().to(get_docs))
            .route("/add", web::post().to(add_doc)),
    );
}
