use crate::{
    model::{
        dict::collar_status::CollarStatus,
        diesel::tex::custom_tex_models::TexFile,
        request::{
            file::{
                add::{file_add_req::TexFileAddReq, file_add_ver_req::TexFileVerAddReq},
                del::file_del::TexFileDelReq,
                edit::move_file_req::MoveFileReq,
                file_rename::TexFileRenameReq,
                query::{
                    download_file_query::DownloadFileQuery, file_code_params::FileCodeParams,
                    file_query_params::FileQueryParams, main_file_params::MainFileParams,
                    pdf_partial::PdfPartial, sub_file_query_params::SubFileQueryParams,
                },
            },
            project::share::collar_query_params::CollarQueryParams,
        },
        response::file::ws_file_detail::WsFileDetail,
    },
    service::{
        file::{
            file_service::{
                create_file, delete_file_recursive, file_init_complete, get_cached_file_by_fid,
                get_file_by_ids, get_file_list, get_file_tree, get_main_file_list, get_partial_pdf,
                get_path_content_by_fid, get_text_file_code, mv_file_impl, proj_folder_tree,
                rename_trans, TexFileService,
            },
            file_version_service::{
                create_file_ver, get_latest_file_version_by_fid, update_file_version,
                update_version_status,
            },
            spec::file_spec::FileSpec,
        },
        project::{
            project_service::{del_project_cache, get_cached_proj_info, get_proj_latest_pdf},
            share::share_service::get_collar_relation,
        },
    },
};
use actix_files::NamedFile;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use log::error;
use mime::Mime;
use rust_i18n::t;
use rust_wheel::{
    common::{
        util::time_util::get_current_millisecond,
        wrapper::actix_http_resp::{
            box_actix_rest_response, box_err_actix_rest_response, box_error_actix_rest_response,
        },
    },
    model::{
        error::infra_error::InfraError, response::api_response::ApiResponse,
        user::login_user_info::LoginUserInfo,
    },
};

pub async fn get_file(params: web::Query<FileQueryParams>) -> impl Responder {
    let docs = get_cached_file_by_fid(&params.file_id).unwrap();
    box_actix_rest_response(docs)
}

pub async fn download_file(
    req: HttpRequest,
    params: web::Query<DownloadFileQuery>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let tex_file: Option<TexFile> = get_cached_file_by_fid(&params.file_id);
    if tex_file.is_none() {
        return Err(actix_web::error::ErrorBadRequest("File not Found"));
    }
    let t_file = tex_file.unwrap();
    let collar_params: CollarQueryParams = CollarQueryParams {
        project_id: t_file.project_id.clone(),
        user_id: login_user_info.userId,
    };
    let collar_params = get_collar_relation(&collar_params).await;
    if collar_params.is_none() || collar_params.unwrap().is_empty() {
        return Err(actix_web::error::ErrorBadRequest("lack of privilleage"));
    }
    let path = get_path_content_by_fid(&t_file);
    if path.is_none() {
        return Err(actix_web::error::ErrorBadRequest("File not Found"));
    }
    match NamedFile::open(&path.clone().unwrap()) {
        Ok(file) => {
            let content_type: Mime = "application/octet-stream".parse().unwrap();
            Ok(NamedFile::set_content_type(file, content_type).into_response(&req))
        }
        Err(e) => {
            error!("Error open file {},{}", path.unwrap(), e);
            return Err(actix_web::error::ErrorBadRequest("File not Found"));
        }
    }
}

pub async fn get_y_websocket_file(params: web::Query<FileQueryParams>) -> impl Responder {
    let docs = get_cached_file_by_fid(&params.file_id).unwrap();
    let proj = get_cached_proj_info(&docs.project_id);
    let file_detail = WsFileDetail {
        file_path: docs.file_path,
        project_id: docs.project_id,
        created_time: docs.created_time,
        updated_time: docs.updated_time,
        file_id: docs.file_id,
        name: docs.name,
        project_created_time: proj.unwrap().main.created_time,
    };
    box_actix_rest_response(file_detail)
}

pub async fn get_files(params: web::Query<SubFileQueryParams>) -> impl Responder {
    let docs = get_file_list(&params.parent);
    box_actix_rest_response(docs)
}

pub async fn get_main_file(params: web::Query<MainFileParams>) -> impl Responder {
    let docs = get_main_file_list(&params.project_id);
    box_actix_rest_response(docs)
}

pub async fn get_file_code(params: web::Query<FileCodeParams>) -> impl Responder {
    let file_text = get_text_file_code(&params.file_id);
    box_actix_rest_response(file_text)
}

pub async fn get_files_tree(params: web::Query<SubFileQueryParams>) -> impl Responder {
    let docs = get_file_tree(&params.parent);
    box_actix_rest_response(docs)
}

pub async fn get_proj_folder_tree(params: web::Query<SubFileQueryParams>) -> impl Responder {
    let docs = proj_folder_tree(&params.parent);
    box_actix_rest_response(docs)
}

pub async fn add_file(
    form: actix_web_validator::Json<TexFileAddReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let fs = TexFileService {};
    let file_count = fs.get_proj_file_count(&form.0.project_id);
    if file_count > 1000 {
        return box_error_actix_rest_response(
            "",
            "001002D001".to_owned(),
            "exceed the file limit".to_owned(),
        );
    }
    return create_file(&form.0, &login_user_info).await;
}

///
/// each version save the file full content
/// each version may take some disk space
/// generate file version in peroid time
/// not every time save
///
pub async fn add_file_version(
    form: actix_web_validator::Json<TexFileVerAddReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let legacy_version = get_latest_file_version_by_fid(&form.0.file_id);
    if legacy_version.is_some() {
        let unboxed_version = legacy_version.unwrap();
        let diff_time = get_current_millisecond() - unboxed_version.created_time;
        if diff_time < 60000 {
            // if last version less than 60s, replace it
            let result = update_file_version(&form.0, &unboxed_version.id);
            return box_actix_rest_response(result.unwrap());
        } else {
            // update and save the new draft version
            update_version_status(&unboxed_version.id);
            let tex_file_version = create_file_ver(&form.0, &login_user_info);
            return box_actix_rest_response(tex_file_version);
        }
    }
    let tex_file_version = create_file_ver(&form.0, &login_user_info);
    box_actix_rest_response(tex_file_version)
}

pub async fn update_file_init(form: web::Json<FileCodeParams>) -> impl Responder {
    let new_file = file_init_complete(&form.0);
    box_actix_rest_response(new_file)
}

pub async fn del_file(form: web::Json<TexFileDelReq>) -> impl Responder {
    let db_file = get_cached_file_by_fid(&form.file_id).unwrap();
    if db_file.main_flag == 1 {
        let res = ApiResponse {
            result: "main file could not be delete",
            resultCode: "delete_main_forbidden".to_owned(),
            ..Default::default()
        };
        return HttpResponse::Ok().json(res);
    }
    let new_file = delete_file_recursive(&form.0, &db_file).unwrap();
    box_actix_rest_response(new_file)
}

pub async fn rename_file(
    form: actix_web_validator::Json<TexFileRenameReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let db_file = rename_trans(&form.0, &login_user_info).await;
    box_actix_rest_response(db_file)
}

pub async fn move_node(
    form: actix_web_validator::Json<MoveFileReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let mut ids = Vec::new();
    ids.push(form.0.file_id.clone());
    ids.push(form.0.dist_file_id.clone());
    let db_files = get_file_by_ids(&ids);
    let src_file = db_files
        .clone()
        .into_iter()
        .find(|f| f.file_id.eq(&form.0.file_id.clone()));
    if src_file.is_none() {
        return box_error_actix_rest_response(
            "failed",
            "FILE_NOT_FOUND".to_owned(),
            "文件未找到".to_owned(),
        );
    }
    if src_file.clone().unwrap().main_flag == 1 {
        return box_error_actix_rest_response(
            "",
            "001001P001".to_owned(),
            t!("err_cannot_mv_main").to_string(),
        );
    }
    let dist_file = db_files
        .into_iter()
        .find(|f| f.file_id.eq(&form.0.dist_file_id.clone()));
    let move_result = mv_file_impl(
        &form.0,
        &login_user_info,
        &src_file.unwrap(),
        &dist_file.unwrap(),
    );
    if let Err(err) = &move_result {
        error!("move file failed,{}", err);
        return box_error_actix_rest_response(
            "failed",
            "MOVE_FILE_FAILED".to_owned(),
            "".to_owned(),
        );
    }
    let db_file = move_result.unwrap();
    if db_file.is_none() {
        return box_error_actix_rest_response(
            "no texfile",
            "MOVE_FILE_FAILED".to_owned(),
            "".to_owned(),
        );
    }
    del_project_cache(&db_file.clone().unwrap().project_id).await;
    box_actix_rest_response("ok")
}

/**
 * when the pdf become huge, loading the whole pdf everytime wasted too much resource
 * this api provide partial pdf loading to improve the performance and save system resource
 */
pub async fn load_partial(
    req: HttpRequest,
    params: actix_web_validator::Query<PdfPartial>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let range_header = req.headers().get("Range");
    if range_header.is_none() {
        return HttpResponse::BadRequest().finish();
    }
    let collar_query = CollarQueryParams {
        project_id: params.0.proj_id.clone(),
        user_id: login_user_info.userId,
    };
    let relation = get_collar_relation(&collar_query).await;
    if relation.is_none() {
        return box_err_actix_rest_response(InfraError::AccessResourceDenied);
    }
    if relation.unwrap()[0].collar_status == CollarStatus::Exit as i32 {
        return box_err_actix_rest_response(InfraError::AccessResourceDenied);
    }
    let pdf_info = get_proj_latest_pdf(&params.0.proj_id, &login_user_info.userId).await;
    if let Err(err) = pdf_info {
        return box_err_actix_rest_response(err);
    }
    return get_partial_pdf(&pdf_info.unwrap(), range_header);
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/file")
            .route("/list", web::get().to(get_files))
            .route("/add", web::post().to(add_file))
            .route("/tree", web::get().to(get_files_tree))
            .route("/folder/tree", web::get().to(get_proj_folder_tree))
            .route("/ver/add", web::post().to(add_file_version))
            .route("/del", web::delete().to(del_file))
            .route("/main", web::get().to(get_main_file))
            .route("/code", web::get().to(get_file_code))
            .route("/mv", web::patch().to(move_node))
            .route("/inited", web::put().to(update_file_init))
            .route("/rename", web::patch().to(rename_file))
            .route("/detail", web::get().to(get_file))
            .route("/download", web::get().to(download_file))
            .route("/pdf/partial", web::get().to(load_partial))
            .route("/y-websocket/detail", web::get().to(get_y_websocket_file)),
    );
}
