use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;
use crate::{service::doc::doc_service::{get_doc_list, create_doc}, model::request::doc::tex_doc_req::TexDocReq};

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

pub async fn get_demo(
    params: web::Query<AppParams>, // login_user: LoginUserInfo
) -> impl Responder {
    let docs = get_doc_list(&params.tag);
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
        web::scope("/tex/doc")
            .route("/list", web::get().to(get_demo))
            .route("/add", web::post().to(add_doc)),
    );
}