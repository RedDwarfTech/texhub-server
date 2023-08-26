use crate::{
    model::request::file::{file_add_req::TexFileAddReq, file_del::TexFileDelReq},
    service::file::file_service::{
        create_file, delete_file_recursive, get_file_by_fid, get_file_list, get_file_tree,
        get_main_file_list, get_text_file_code, file_init_complete,
    },
};
use actix_web::{web, HttpResponse, Responder};
use rust_wheel::model::response::api_response::ApiResponse;

#[derive(serde::Deserialize)]
pub struct AppParams {
    parent: String,
}

#[derive(serde::Deserialize)]
pub struct MainFileParams {
    pub project_id: String,
}

#[derive(serde::Deserialize)]
pub struct FileCodeParams {
    pub file_id: String,
}

#[derive(serde::Deserialize)]
pub struct FileQueryParams {
    file_id: String,
}

pub async fn get_file(params: web::Query<FileQueryParams>) -> impl Responder {
    let docs = get_file_by_fid(&params.file_id);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_files(params: web::Query<AppParams>) -> impl Responder {
    let docs = get_file_list(&params.parent);
    let res = ApiResponse {
        result: docs,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_main_file(params: web::Query<MainFileParams>) -> impl Responder {
    let docs = get_main_file_list(&params.project_id);
    let res = ApiResponse {
        result: docs[0].clone(),
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_file_code(params: web::Query<FileCodeParams>) -> impl Responder {
    let file_text = get_text_file_code(&params.file_id);
    let res = ApiResponse {
        result: file_text,
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

pub async fn update_file_init(form: web::Json<FileCodeParams>) -> impl Responder {
    let new_file = file_init_complete(&form.0);
    let res = ApiResponse {
        result: new_file,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn del_file(form: web::Json<TexFileDelReq>) -> impl Responder {
    let db_file = get_file_by_fid(&form.file_id);
    if db_file.main_flag == 1 {
        let res = ApiResponse {
            result: "main file could not be delete",
            resultCode: "delete_main_forbidden".to_owned(),
            ..Default::default()
        };
        return HttpResponse::Ok().json(res);
    }
    let new_file = delete_file_recursive(&form.0).unwrap();
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
            .route("/tree", web::get().to(get_files_tree))
            .route("/del", web::delete().to(del_file))
            .route("/main", web::get().to(get_main_file))
            .route("/code", web::get().to(get_file_code))
            .route("/inited", web::put().to(update_file_init))
            .route("/detail", web::get().to(get_file)),
    );
}
