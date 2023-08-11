use crate::{
    model::request::project::{
        tex_del_project_req::TexDelProjectReq, tex_project_req::TexProjectReq, tex_compile_project_req::TexCompileProjectReq,
    },
    service::project::project_service::{create_project, del_project, get_prj_list, compile_project},
};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
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

pub async fn add_project(form: web::Json<TexProjectReq>) -> impl Responder {
    let d_name = form.doc_name.clone();
    let projects = create_project(&d_name);
    match projects {
        Ok(project) => {
            let res = ApiResponse {
                result: project,
                ..Default::default()
            };
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            let err = format!("create project failed,{}", e);
            let res = ApiResponse {
                result: err,
                ..Default::default()
            };
            HttpResponse::Ok().json(res)
        }
    }
}

pub async fn del_proj(form: web::Json<TexDelProjectReq>) -> impl Responder {
    let d_name = form.project_id.clone();
    del_project(&d_name);
    let res = ApiResponse {
        result: "ok",
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn compile_proj(_form: web::Json<TexCompileProjectReq>) -> impl Responder {
    compile_project();
    let res = ApiResponse {
        result: "ok",
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/list", web::get().to(get_docs))
            .route("/add", web::post().to(add_project))
            .route("/del", web::delete().to(del_proj))
            .route("/compile", web::put().to(compile_proj)),
    );
}
