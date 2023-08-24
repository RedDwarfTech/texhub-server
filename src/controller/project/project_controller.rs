use crate::{
    model::{
        request::project::{
            tex_compile_project_req::TexCompileProjectReq, tex_del_project_req::TexDelProjectReq,
            tex_project_req::TexProjectReq,
        },
        response::project::latest_compile::LatestCompile,
    },
    service::project::project_service::{
        compile_project, create_empty_project, del_project, get_prj_by_id, get_prj_list,
        get_project_pdf,
    },
};
use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use rust_wheel::model::{
    response::api_response::ApiResponse, user::login_user_info::LoginUserInfo,
};

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

#[derive(serde::Deserialize)]
pub struct GetPrjParams {
    pub project_id: String,
}

pub async fn get_projects(
    params: web::Query<AppParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let projects = get_prj_list(&params.tag, &login_user_info);
    let res = ApiResponse {
        result: projects,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_project(params: web::Query<GetPrjParams>) -> impl Responder {
    let prj = get_prj_by_id(&params.project_id);
    let res = ApiResponse {
        result: prj,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn create_project(
    form: actix_web_validator::Json<TexProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let d_name = form.name.clone();
    let projects = create_empty_project(&d_name, &login_user_info.userId);
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

pub async fn compile_proj(form: web::Json<TexCompileProjectReq>) -> impl Responder {
    let compile_result = compile_project(&form.0).await;
    let res = ApiResponse {
        result: compile_result.unwrap(),
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_latest_pdf(params: web::Query<GetPrjParams>) -> impl Responder {
    let version_no = get_project_pdf(&params.0).await;
    let pdf_result: LatestCompile = LatestCompile {
        path: version_no,
        project_id: params.0.project_id,
    };
    let res = ApiResponse {
        result: pdf_result,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/list", web::get().to(get_projects))
            .route("/add", web::post().to(create_project))
            .route("/del", web::delete().to(del_proj))
            .route("/pdf", web::get().to(get_latest_pdf))
            .route("/compile", web::put().to(compile_proj)),
    );
}
