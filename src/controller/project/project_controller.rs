use crate::{
    model::{
        diesel::custom::project::upload::proj_upload_file::ProjUploadFile,
        request::project::{
            add::tex_project_tpl_req::TexProjectTplReq,
            edit::edit_proj_req::EditProjReq,
            query::{get_proj_params::GetProjParams, proj_query_params::ProjQueryParams},
            queue::queue_status_req::QueueStatusReq,
            tex_compile_project_req::TexCompileProjectReq,
            tex_compile_queue_log::TexCompileQueueLog,
            tex_compile_queue_req::TexCompileQueueReq,
            tex_compile_queue_status::TexCompileQueueStatus,
            tex_del_project_req::TexDelProjectReq,
            tex_join_project_req::TexJoinProjectReq,
            tex_project_req::TexProjectReq,
        },
    },
    service::{
        project::project_service::{
            add_compile_to_queue, compile_project, compile_status_update, create_empty_project,
            create_tpl_project, del_project, edit_proj, get_cached_proj_info,
            get_cached_queue_status, get_comp_log_stream, get_compiled_log, get_proj_by_type,
            get_proj_latest_pdf, join_project, save_proj_file, send_render_req,
        },
        tpl::template_service::get_tempalte_by_id,
    },
};
use actix_multipart::form::MultipartForm;
use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse, Responder,
};
use log::error;
use rust_wheel::{
    common::{
        util::net::{sse_message::SSEMessage, sse_stream::SseStream},
        wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    },
    model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo},
};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task,
};

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

pub async fn get_project(params: web::Query<GetProjParams>) -> impl Responder {
    let proj = get_cached_proj_info(&params.project_id);
    return box_actix_rest_response(proj.unwrap());
}

pub async fn edit_project(params: web::Json<EditProjReq>) -> impl Responder {
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
        Ok(project) => box_actix_rest_response(project),
        Err(e) => {
            let err = format!("create project failed,{}", e);
            box_error_actix_rest_response(err.clone(), "CREATE_PROJ_FAILED".to_owned(), err)
        }
    }
}

pub async fn create_project_by_tpl(
    form: actix_web_validator::Json<TexProjectTplReq>,
    login_user_info: LoginUserInfo,
) -> HttpResponse {
    let tpl = get_tempalte_by_id(&form.0.template_id);
    let projects = create_tpl_project(&tpl, &login_user_info).await;
    match projects {
        Ok(project) => {
            if project.is_some() {
                box_actix_rest_response(project.unwrap())
            } else {
                box_error_actix_rest_response(
                    "failed with none",
                    "CREATE_TPL_PROJ_FAILED".to_owned(),
                    "failed".to_owned(),
                )
            }
        }
        Err(e) => {
            error!("Error creating project,{}", e);
            box_error_actix_rest_response(
                "failed",
                "CREATE_TPL_PROJ_FAILED".to_owned(),
                "failed".to_owned(),
            )
        }
    }
}

pub async fn del_proj(
    form: web::Json<TexDelProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    del_project(&form.project_id.clone(), &login_user_info);
    box_actix_rest_response("ok")
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
    box_actix_rest_response(compile_result)
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

pub async fn update_compile_status(form: web::Json<TexCompileQueueStatus>) -> impl Responder {
    return compile_status_update(&form.0).await;
}

pub async fn get_latest_pdf(params: web::Query<GetProjParams>) -> impl Responder {
    let pdf_info = get_proj_latest_pdf(&params.0.project_id).await;
    box_actix_rest_response(pdf_info)
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

pub async fn get_proj_compile_log(form: web::Query<TexCompileQueueLog>) -> HttpResponse {
    let output = get_compiled_log(&form.0).await;
    return box_actix_rest_response(output);
}

pub async fn get_queue_status(form: web::Query<QueueStatusReq>) -> HttpResponse {
    let result = get_cached_queue_status(form.0.id).await;
    return box_actix_rest_response(result.unwrap_or_default());
}

async fn upload_proj_file(
    MultipartForm(form): MultipartForm<ProjUploadFile>,
    login_user_info: LoginUserInfo,
) -> HttpResponse {
    let files = save_proj_file(form, &login_user_info).await;
    box_actix_rest_response(files)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/list", web::get().to(get_projects))
            .route("/info", web::get().to(get_project))
            .route("/add", web::post().to(create_project))
            .route("/add-from-tpl", web::post().to(create_project_by_tpl))
            .route("/del", web::delete().to(del_proj))
            .route("/latest/pdf", web::get().to(get_latest_pdf))
            .route("/edit", web::put().to(edit_project))
            .route("/join", web::post().to(join_proj))
            .route("/file/upload", web::post().to(upload_proj_file))
            .route("/log/stream", web::get().to(sse_handler))
            .route("/temp/code", web::get().to(get_temp_auth_code))
            .route("/compile", web::put().to(compile_proj))
            .route("/queue/status", web::get().to(get_queue_status))
            .route(
                "/compile/log/stream",
                web::get().to(get_proj_compile_log_stream),
            )
            .route("/compile/log", web::get().to(get_proj_compile_log))
            .route("/compile/queue", web::post().to(add_compile_req_to_queue))
            .route("/compile/store", web::post().to(add_compile_req_to_db))
            .route("/compile/status", web::put().to(update_compile_status)),
    );
}
