use crate::{
    model::{
        request::{file::{
            add::file_add_req::TexFileAddReq, file_del::TexFileDelReq, file_rename::TexFileRenameReq,
            query::file_query_params::FileQueryParams, edit::move_file_req::MoveFileReq, add::file_add_ver_req::TexFileVerAddReq,
        }, project::query::get_proj_history::GetProjHistory},
        response::file::ws_file_detail::WsFileDetail,
    },
    service::{
        file::file_service::{
            create_file, delete_file_recursive, file_init_complete, get_file_by_fid, get_file_list,
            get_file_tree, get_main_file_list, get_text_file_code, mv_file_impl, rename_trans, create_file_ver, get_proj_history,
        },
        project::project_service::{get_cached_proj_info, del_project_cache},
    },
};
use actix_web::{web, HttpResponse, Responder};
use log::error;
use rust_wheel::{
    common::wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    model::{response::api_response::ApiResponse, user::login_user_info::LoginUserInfo},
};

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

pub async fn get_file(params: web::Query<FileQueryParams>) -> impl Responder {
    let docs = get_file_by_fid(&params.file_id).unwrap();
    box_actix_rest_response(docs)
}

pub async fn get_y_websocket_file(params: web::Query<FileQueryParams>) -> impl Responder {
    let docs = get_file_by_fid(&params.file_id).unwrap();
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

pub async fn get_files(params: web::Query<AppParams>) -> impl Responder {
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

pub async fn get_files_tree(params: web::Query<AppParams>) -> impl Responder {
    let docs = get_file_tree(&params.parent);
    box_actix_rest_response(docs)
}

pub async fn add_file(
    form: actix_web_validator::Json<TexFileAddReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    return create_file(&form.0, &login_user_info).await;
}

pub async fn add_file_version(
    form: actix_web_validator::Json<TexFileVerAddReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let tex_file_version = create_file_ver(&form.0, &login_user_info);
    box_actix_rest_response(tex_file_version)
}

pub async fn update_file_init(form: web::Json<FileCodeParams>) -> impl Responder {
    let new_file = file_init_complete(&form.0);
    box_actix_rest_response(new_file)
}

pub async fn del_file(form: web::Json<TexFileDelReq>) -> impl Responder {
    let db_file = get_file_by_fid(&form.file_id).unwrap();
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

pub async fn move_node(form: actix_web_validator::Json<MoveFileReq>,login_user_info: LoginUserInfo) -> impl Responder {
    let move_result = mv_file_impl(&form.0, &login_user_info).await;
    if let Err(err) = &move_result {
        error!("move file failed,{}", err);
        box_error_actix_rest_response("failed", "MOVE_FILE_FAILED".to_owned(), "".to_owned());
    }
    let db_file = move_result.unwrap();
    if db_file.is_none() {
        return box_error_actix_rest_response("no texfile", "MOVE_FILE_FAILED".to_owned(), "".to_owned());
    }
    del_project_cache(&db_file.clone().unwrap().project_id).await;
    let proj_file_tree = get_file_tree(&db_file.unwrap().project_id);
    box_actix_rest_response(proj_file_tree)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/file")
            .route("/list", web::get().to(get_files))
            .route("/add", web::post().to(add_file))
            .route("/tree", web::get().to(get_files_tree))
            .route("/ver/add", web::post().to(add_file_version))
            .route("/del", web::delete().to(del_file))
            .route("/main", web::get().to(get_main_file))
            .route("/code", web::get().to(get_file_code))
            .route("/mv", web::patch().to(move_node))
            .route("/inited", web::put().to(update_file_init))
            .route("/rename", web::patch().to(rename_file))
            .route("/detail", web::get().to(get_file))
            .route("/y-websocket/detail", web::get().to(get_y_websocket_file)),
    );
}
