use crate::common::database::get_connection;
use crate::handle_multipart_error;
use crate::model::dict::collar_status::CollarStatus;
use crate::model::diesel::custom::project::upload::full_proj_upload::FullProjUpload;
use crate::model::diesel::custom::project::upload::github_proj_sync::GithubProjSync;
use crate::model::diesel::tex::custom_tex_models::TexProjFolder;
use crate::model::error::texhub_error::TexhubError;
use crate::model::request::project::add::copy_proj_req::CopyProjReq;
use crate::model::request::project::add::tex_file_idx_req::TexFileIdxReq;
use crate::model::request::project::add::tex_folder_req::TexFolderReq;
use crate::model::request::project::add::tex_project_req::TexProjectReq;
use crate::model::request::project::del::del_folder_req::DelFolderReq;
use crate::model::request::project::edit::archive_proj_req::ArchiveProjReq;
use crate::model::request::project::edit::edit_proj_folder::EditProjFolder;
use crate::model::request::project::edit::rename_proj_folder::RenameProjFolder;
use crate::model::request::project::edit::trash_proj_req::TrashProjReq;
use crate::model::request::project::query::download_proj::DownloadProj;
use crate::model::request::project::query::folder_proj_params::FolderProjParams;
use crate::model::request::project::query::get_proj_history_page::GetProjPageHistory;
use crate::model::request::project::query::search_proj_params::SearchProjParams;
use crate::model::request::project::share::collar_query_params::CollarQueryParams;
use crate::model::response::project::proj_resp::ProjResp;
use crate::model::response::project::tex_proj_resp::TexProjResp;
use crate::service::file::file_service::{
    get_cached_file_by_fid, get_proj_history_page_impl, push_to_fulltext_search,
};
use crate::service::project::project_folder_map_service::move_proj_folder;
use crate::service::project::project_service::{
    del_proj_collection_folder, del_project, del_project_logic, do_proj_copy,
    get_folder_project_impl, get_proj_folders, handle_archive_proj, handle_compress_proj,
    handle_folder_create, handle_trash_proj, import_from_github_impl, proj_search_impl,
    rename_proj_collection_folder, save_full_proj, TexProjectService,
};
use crate::service::project::share::share_service::get_collar_relation;
use crate::service::project::spec::proj_spec::ProjSpec;
use crate::{
    model::{
        diesel::custom::{
            file::{search_file::SearchFile, search_file_resp::SearchFileResp},
            project::upload::proj_upload_file::ProjUploadFile,
        },
        request::project::{
            add::tex_project_tpl_req::TexProjectTplReq,
            edit::edit_proj_req::EditProjReq,
            query::{
                get_pdf_pos_params::GetPdfPosParams, get_proj_params::GetProjParams,
                get_src_pos_params::GetSrcPosParams, proj_query_params::ProjQueryParams,
            },
            queue::queue_status_req::QueueStatusReq,
            tex_compile_project_req::TexCompileProjectReq,
            tex_compile_queue_log::TexCompileQueueLog,
            tex_compile_queue_req::TexCompileQueueReq,
            tex_compile_queue_status::TexCompileQueueStatus,
            tex_del_project_req::TexDelProjectReq,
            tex_join_project_req::TexJoinProjectReq,
        },
    },
    service::{
        project::project_service::{
            add_compile_to_queue, compile_project, compile_status_update, create_empty_project,
            create_tpl_project, edit_proj, get_cached_proj_info, get_cached_queue_status,
            get_comp_log_stream, get_compiled_log, get_pdf_pos, get_proj_latest_pdf, get_src_pos,
            join_project, save_proj_file, send_render_req,
        },
        tpl::template_service::get_tempalte_by_id,
    },
};
use actix_files::NamedFile;
use actix_multipart::form::{MultipartForm, MultipartFormConfig};
use actix_web::HttpRequest;
use actix_web::{
    http::header::{CacheControl, CacheDirective},
    web, HttpResponse, Responder,
};
use log::error;
use meilisearch_sdk::SearchResult;
use mime::Mime;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::common::wrapper::actix_http_resp::box_err_actix_rest_response;
use rust_wheel::model::error::infra_error::InfraError;
use rust_wheel::{
    common::{
        util::net::{sse_message::SSEMessage, sse_stream::SseStream},
        wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    },
    model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo},
};
use serde_json::{Map, Value};
use tokio::{
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task,
};

pub async fn get_projects(
    params: web::Query<ProjQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let folders: Vec<TexProjFolder> = get_proj_folders(&params.0, &login_user_info);
    let default_folder = folders.iter().find(|folder| folder.default_folder == 1);
    let ps = TexProjectService {};
    let projects: Vec<TexProjResp> =
        ps.get_proj_by_type(&params.0, &login_user_info, default_folder);
    let resp: ProjResp = ProjResp::from_req(folders, projects);
    let res = ApiResponse {
        result: resp,
        ..Default::default()
    };
    HttpResponse::Ok().json(res)
}

pub async fn get_folder_projects(
    params: web::Query<FolderProjParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let projects = get_folder_project_impl(&params.0, &login_user_info);
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
    let ps = TexProjectService {};
    let project_count = ps.get_proj_count_by_uid(&login_user_info.userId);
    if project_count > 2 && login_user_info.vipExpireTime < get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::NonVipTooMuchProj);
    }
    if project_count > 50 && login_user_info.vipExpireTime > get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::VipTooMuchProj);
    }
    let projects = create_empty_project(&form.0, &login_user_info).await;
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

pub async fn _logic_del_proj(
    form: web::Json<TexDelProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let proj_editor_result = del_project_logic(&form.project_id.clone(), &login_user_info);
    box_actix_rest_response(proj_editor_result)
}

pub async fn join_proj(
    form: web::Json<TexJoinProjectReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let result = join_project(&form.0, &login_user_info).await;
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

pub async fn get_latest_pdf(
    params: web::Query<GetProjParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let collar_query = CollarQueryParams {
        project_id: params.project_id.clone(),
        user_id: login_user_info.userId,
    };
    let relation = get_collar_relation(&collar_query).await;
    if relation.is_none() {
        return box_err_actix_rest_response(InfraError::AccessResourceDenied);
    }
    if relation.unwrap()[0].collar_status == CollarStatus::Exit as i32 {
        return box_err_actix_rest_response(InfraError::AccessResourceDenied);
    }
    let pdf_info = get_proj_latest_pdf(&params.0.project_id, &login_user_info.userId).await;
    match pdf_info {
        Ok(pdf) => box_actix_rest_response(pdf),
        Err(e) => box_err_actix_rest_response(e),
    }
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

pub async fn get_proj_compile_log_stream(
    form: web::Query<TexCompileQueueLog>,
    login_user_info: LoginUserInfo,
) -> HttpResponse {
    let (tx, rx): (
        UnboundedSender<SSEMessage<String>>,
        UnboundedReceiver<SSEMessage<String>>,
    ) = tokio::sync::mpsc::unbounded_channel();
    task::spawn(async move {
        let output = get_comp_log_stream(&form.0, tx, &login_user_info).await;
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
    return save_proj_file(form, &login_user_info).await;
}

async fn get_pdf_position(form: web::Query<GetPdfPosParams>) -> HttpResponse {
    let pos = get_pdf_pos(&form.0);
    box_actix_rest_response(pos)
}

async fn get_src_position(form: web::Query<GetSrcPosParams>) -> HttpResponse {
    let pos = get_src_pos(&form.0);
    box_actix_rest_response(pos)
}

async fn proj_search(form: actix_web_validator::Query<SearchProjParams>) -> HttpResponse {
    let pos = proj_search_impl(&form.0).await;
    if pos.is_some() {
        let searched_files = pos.unwrap().clone();
        let ftr: Vec<SearchFileResp> = get_fulltext_result(searched_files.hits);
        box_actix_rest_response(ftr)
    } else {
        box_actix_rest_response("")
    }
}

fn get_fulltext_result(inputs: Vec<SearchResult<SearchFile>>) -> Vec<SearchFileResp> {
    let mut files = Vec::new();
    for item in inputs {
        let formmatted_result: Option<Map<String, Value>> = item.formatted_result;
        if formmatted_result.is_some() {
            let unwrap_result: Map<String, Value> = formmatted_result.unwrap();
            let sfr = SearchFileResp::new_file(unwrap_result);
            files.push(sfr);
        }
    }
    return files;
}

async fn update_idx(form: web::Json<TexFileIdxReq>) -> HttpResponse {
    let tex_file = get_cached_file_by_fid(&form.0.file_id);
    let pos = push_to_fulltext_search(&tex_file.unwrap(), &form.0.content).await;
    box_actix_rest_response(pos)
}

async fn update_proj_nickname(form: web::Json<TexFileIdxReq>) -> HttpResponse {
    let tex_file = get_cached_file_by_fid(&form.0.file_id);
    let pos = push_to_fulltext_search(&tex_file.unwrap(), &form.0.content).await;
    box_actix_rest_response(pos)
}

pub async fn get_proj_his_page(params: web::Query<GetProjPageHistory>) -> impl Responder {
    let proj_history = get_proj_history_page_impl(&params.0);
    box_actix_rest_response(proj_history)
}

pub async fn archive_project(
    form: web::Json<ArchiveProjReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let proj_history = handle_archive_proj(&form.0, &login_user_info);
    box_actix_rest_response(proj_history)
}

pub async fn trash_project(
    form: web::Json<TrashProjReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let trash_result = handle_trash_proj(&form.0, &login_user_info);
    box_actix_rest_response(trash_result)
}

/**
* facing content type error
* https://stackoverflow.com/questions/77738477/content-type-error-when-using-rust-actix-download-file
* curl http://localhost:8000/tex/project/download?project_id=1&version=1
* curl http://localhost:8000/tex/project/compress?project_id=1&version=1
* curl -H "Content-Type: application/json" -X PUT -d '{"project_id": "1","version": "1"}' -o filename.zip http://localhost:8000/tex/project/download?project_id=1&version=1
* curl -H "Content-Type: application/json" -X PUT -d '{"project_id": "5ef2057551c24b5aa4d0e2cdadcbc524","version": "1"}'  -H "Authorization: Bearer eyJhbGciOiJIxxx" -o filename1.zip https://tex.poemhub.top/tex/project/download

*/
pub async fn download_project(
    req: HttpRequest,
    form: web::Json<DownloadProj>,
) -> actix_web::Result<impl actix_web::Responder> {
    let path = handle_compress_proj(&form.0);
    match NamedFile::open(&path) {
        Ok(file) => {
            let content_type: Mime = "application/zip".parse().unwrap();
            Ok(NamedFile::set_content_type(file, content_type).into_response(&req))
        }
        Err(_) => Err(actix_web::error::ErrorBadRequest("File not Found")),
    }
}

pub async fn compress_project(form: web::Json<DownloadProj>) -> impl Responder {
    let path = handle_compress_proj(&form.0);
    box_actix_rest_response(path)
}

pub async fn new_folder(
    form: actix_web_validator::Json<TexFolderReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let folder = handle_folder_create(&form.0, &login_user_info);
    box_actix_rest_response(folder)
}

pub async fn mv_proj_folder(
    form: actix_web_validator::Json<EditProjFolder>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    move_proj_folder(&form.0, &login_user_info.userId, &mut get_connection());
    box_actix_rest_response("ok")
}

pub async fn rename_collect_folder(
    form: actix_web_validator::Json<RenameProjFolder>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let folder = rename_proj_collection_folder(&form.0, &login_user_info);
    box_actix_rest_response(folder)
}

pub async fn del_collect_folder(
    form: actix_web_validator::Json<DelFolderReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let folder = del_proj_collection_folder(&form.0, &login_user_info);
    box_actix_rest_response(folder)
}

pub async fn cp_proj(
    form: actix_web_validator::Json<CopyProjReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    return do_proj_copy(&form.0, &login_user_info).await;
}

async fn upload_full_proj(
    MultipartForm(form): MultipartForm<FullProjUpload>,
    login_user_info: LoginUserInfo,
) -> HttpResponse {
    return save_full_proj(form, &login_user_info).await;
}

async fn import_from_github(
    form: actix_web_validator::Json<GithubProjSync>,
    login_user_info: LoginUserInfo,
) -> HttpResponse {
    let ps = TexProjectService {};
    let project_count = ps.get_proj_count_by_uid(&login_user_info.userId);
    if project_count > 2 && login_user_info.vipExpireTime < get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::NonVipTooMuchProj);
    }
    if project_count > 50 && login_user_info.vipExpireTime > get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::VipTooMuchProj);
    }
    return import_from_github_impl(&form.0, &login_user_info).await;
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/project")
            .route("/list", web::get().to(get_projects))
            .route("/info", web::get().to(get_project))
            .route("/history/page", web::get().to(get_proj_his_page))
            .route("/add", web::post().to(create_project))
            .route("/add-from-tpl", web::post().to(create_project_by_tpl))
            .route("/", web::delete().to(del_proj))
            .route("/latest/pdf", web::get().to(get_latest_pdf))
            .route("/pos/pdf", web::get().to(get_pdf_position))
            .route("/pos/src", web::get().to(get_src_position))
            .route("/edit", web::patch().to(edit_project))
            .route("/join", web::post().to(join_proj))
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
            .route("/compile/status", web::put().to(update_compile_status))
            .route("/search", web::get().to(proj_search))
            .route("/flush/idx", web::put().to(update_idx))
            .route("/nickname", web::put().to(update_proj_nickname))
            .route("/archive", web::put().to(archive_project))
            .route("/trash", web::put().to(trash_project))
            .route("/download", web::put().to(download_project))
            .route("/compress", web::put().to(compress_project))
            .route("/folder", web::post().to(new_folder))
            .route("/move", web::patch().to(mv_proj_folder))
            .route("/perfolder", web::get().to(get_folder_projects))
            .route("/folder/rename", web::patch().to(rename_collect_folder))
            .route("/folder/del", web::delete().to(del_collect_folder))
            .route("/copy", web::post().to(cp_proj))
            .route("/github/import", web::post().to(import_from_github)),
    );
    // https://stackoverflow.com/questions/71714621/actix-web-limit-upload-file-size
    // https://stackoverflow.com/questions/79046623/how-to-fix-the-second-actix-web-service-configuration-404-not-found-error
    let file_upload_config = MultipartFormConfig::default()
        .total_limit(1048576) // 1 MB = 1024 * 1024
        .memory_limit(2097152) // 2 MB = 2 * 1024 * 1024
        .error_handler(handle_multipart_error);
    let proj_upload_config = MultipartFormConfig::default()
        .total_limit(104857600) // 100 MB = 1024 * 1024
        .memory_limit(209715200) // 200 MB = 200 * 1024 * 1024
        .error_handler(handle_multipart_error);
    cfg.service(
        web::scope("/tex/ul/proj")
            .route("/upload", web::post().to(upload_full_proj))
            .app_data(proj_upload_config),
    );
    cfg.service(
        web::scope("/tex/ul/file")
            .route("/upload", web::post().to(upload_proj_file))
            .app_data(file_upload_config),
    );
}
