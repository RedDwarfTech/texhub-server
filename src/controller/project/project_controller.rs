use crate::{
    model::{
        request::project::{
            tex_compile_project_req::TexCompileProjectReq, tex_del_project_req::TexDelProjectReq,
            tex_join_project_req::TexJoinProjectReq, tex_project_req::TexProjectReq,
        },
        response::project::latest_compile::LatestCompile,
    },
    service::{
        file::file_service::get_main_file_list,
        project::project_service::{
            compile_project, create_empty_project, del_project, edit_proj, get_compiled_log,
            get_prj_by_id, get_prj_list, get_project_pdf, join_project,
        },
    },
};
use actix_web::{
    web::{self},
    HttpResponse, Responder, HttpRequest,
};
use rust_wheel::{
    common::wrapper::actix_http_resp::box_actix_rest_response,
    model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo},
};
use futures::StreamExt;

#[derive(serde::Deserialize)]
pub struct AppParams {
    tag: String,
}

#[derive(serde::Deserialize)]
pub struct GetPrjParams {
    pub project_id: String,
}

#[derive(serde::Deserialize)]
pub struct EditPrjReq {
    pub project_id: String,
    pub proj_name: String,
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

pub async fn edit_project(params: web::Json<EditPrjReq>) -> impl Responder {
    let prj = edit_proj(&params);
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

pub async fn join_proj(
    form: web::Json<TexJoinProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let result = join_project(&form.0, &login_user_info);
    let res = ApiResponse {
        result: result.unwrap(),
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

pub async fn get_compile_log(
    form: web::Query<TexCompileProjectReq>
) -> impl Responder {
    let main_file = get_main_file_list(&form.project_id);
    let log_output = get_compiled_log( main_file[0].clone()).await;
    box_actix_rest_response(log_output)
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

async fn sse_handler(req: HttpRequest, stream: web::Payload) -> HttpResponse {
    let mut res = HttpResponse::Ok()
        .content_type("text/event-stream")
        .streaming(Box::pin(stream.map(|chunk| {
            std::io::Result::Ok(actix_web::web::Bytes::from(format!(
                "data: {}\n\n",
                String::from_utf8(chunk.unwrap().to_vec()).unwrap()
            )))
        })));

    if req.headers().get("cache-control").is_none() {
        res.headers_mut().insert(
            actix_web::http::header::CACHE_CONTROL,
            actix_web::http::header::HeaderValue::from_static("no-cache"),
        );
    }

    res
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/list", web::get().to(get_projects))
            .route("/add", web::post().to(create_project))
            .route("/del", web::delete().to(del_proj))
            .route("/pdf", web::get().to(get_latest_pdf))
            .route("/edit", web::put().to(edit_project))
            .route("/join", web::post().to(join_proj))
            .route("/log", web::get().to(get_compile_log))
            .route("/log/stream", web::get().to(sse_handler))
            .route("/compile", web::put().to(compile_proj)),
    );
}
