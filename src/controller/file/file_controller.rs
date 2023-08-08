use crate::{
    model::request::file::file_add_req::TexFileAddReq,
    service::file::file_service::{create_file, get_file_list, get_file_tree},
};
use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AppParams {
    parent: String,
}

pub async fn get_files(params: web::Query<AppParams>) -> impl Responder {
    let docs = get_file_list(&params.parent);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_files_tree(params: web::Query<AppParams>) -> impl Responder {
    let docs = get_file_tree(&params.parent);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn add_file(form: web::Json<TexFileAddReq>) -> impl Responder {
    let new_file = create_file(&form.0);
    let res = ApiResponse {
        result: new_file,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/file")
            .route("/list", web::get().to(get_files))
            .route("/add", web::post().to(add_file))
            .route("/tree", web::get().to(get_files_tree)),
    );
}
