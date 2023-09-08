use crate::{
    model::{
        request::project::{
            tex_compile_project_req::TexCompileProjectReq, tex_del_project_req::TexDelProjectReq,
            tex_join_project_req::TexJoinProjectReq, tex_project_req::TexProjectReq, tex_compile_queue_req::TexCompileQueueReq, queue::queue_status_req::QueueStatusReq, tex_compile_queue_status::TexCompileQueueStatus, tex_compile_queue_log::TexCompileQueueLog,
        },
        response::project::latest_compile::LatestCompile,
    },
    service::{
        file::file_service::get_main_file_list,
        project::project_service::{
            add_compile_to_queue, compile_project, create_empty_project, del_project, edit_proj,
            get_compiled_log, get_prj_by_id, get_proj_by_type, get_project_pdf, join_project,
            send_render_req, get_cached_queue_status, compile_status_update, get_comp_log_stream,
        },
    },
};
use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web::{self},
    HttpResponse, Responder,
};
use log::error;
use rust_wheel::{
    common::{
        util::net::{sse_message::SSEMessage, sse_stream::SseStream},
        wrapper::actix_http_resp::box_actix_rest_response,
    },
    model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo},
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task,
};

#[derive(serde::Deserialize)]
pub struct ProjQueryParams {
    pub tag: String,
    pub role_id: Option<i32>,
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
    params: web::Query<ProjQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let projects = get_proj_by_type(&params.0, &login_user_info);
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
    let projects = create_empty_project(&d_name, &login_user_info).await;
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

pub async fn del_proj(
    form: web::Json<TexDelProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let d_name = form.project_id.clone();
    del_project(&d_name, &login_user_info);
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

pub async fn add_compile_req_to_queue(
    form: web::Json<TexCompileQueueReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    return add_compile_to_queue(&form.0, &login_user_info).await;
}

pub async fn add_compile_req_to_db(
    form: web::Json<TexCompileQueueReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    return add_compile_to_queue(&form.0, &login_user_info).await;
}

pub async fn update_compile_status(
    form: web::Json<TexCompileQueueStatus>,
) -> impl Responder {
    return compile_status_update(&form.0).await;
}

pub async fn get_compile_log(form: web::Query<TexCompileProjectReq>) -> impl Responder {
    let main_file = get_main_file_list(&form.project_id);
    let log_output = get_compiled_log(main_file.unwrap().clone()).await;
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

/**
 * the server sent event did not support http header
 * put the temp auth code in parameter to do a compile requst
 *
 * using polyfill will facing issue:
 *  https://stackoverflow.com/questions/75841904/why-did-not-found-the-chatgpt-event-stream-data-in-google-chrome-devtools
 *  https://github.com/Yaffle/EventSource/issues/79
 *  https://stackoverflow.com/questions/77015804/why-the-event-source-polyfill-did-not-fetch-the-sse-api-data
 *
 */
pub async fn get_temp_auth_code() -> impl Responder {
    return box_actix_rest_response("123456");
}

pub async fn sse_handler(form: web::Query<TexCompileProjectReq>) -> HttpResponse {
    let (tx, rx): (
        UnboundedSender<SSEMessage<String>>,
        UnboundedReceiver<SSEMessage<String>>,
    ) = tokio::sync::mpsc::unbounded_channel();
    task::spawn(async move {
        let output = send_render_req(&form.0, tx).await;
        if let Err(e) = output {
            error!("handle sse req error: {}", e);
        }
    });
    let response = HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .content_type("text/event-stream")
        .streaming(SseStream { receiver: Some(rx) });
    response
}

pub async fn get_proj_compile_log_stream(form: web::Query<TexCompileQueueLog>) -> HttpResponse {
    let (tx, rx): (
        UnboundedSender<SSEMessage<String>>,
        UnboundedReceiver<SSEMessage<String>>,
    ) = tokio::sync::mpsc::unbounded_channel();
    task::spawn(async move {
        let output = get_comp_log_stream(&form.0, tx).await;
        if let Err(e) = output {
            error!("handle sse req error: {}", e);
        }
    });
    let response = HttpResponse::Ok()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .content_type("text/event-stream")
        .streaming(SseStream { receiver: Some(rx) });
    response
}


pub async fn get_queue_status(form: web::Query<QueueStatusReq>) -> HttpResponse {
    let result = get_cached_queue_status(&form.0).await;
    return box_actix_rest_response(result.unwrap_or_default());
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
            .route("/temp/code", web::get().to(get_temp_auth_code))
            .route("/compile", web::put().to(compile_proj))
            .route("/queue/status", web::get().to(get_queue_status))
            .route("/compile/qlog",web::get().to(get_proj_compile_log_stream))
            .route("/compile/queue", web::post().to(add_compile_req_to_queue))
            .route("/compile/store", web::post().to(add_compile_req_to_db))
            .route("/compile/status", web::put().to(update_compile_status)),
    );
}
