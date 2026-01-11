use super::eden::external_app::clone_github_repo;
use super::eden::proj::create_files_into_db;
use super::eden::proj::create_proj;
use super::eden::proj::create_project_dyn_params;
use super::eden::proj::do_create_proj_dependencies;
use super::project_queue_service::get_latest_proj_queue;
use super::spec::proj_spec::ProjSpec;
use crate::common::interop::synctex::synctex_node_visible_h;
use crate::common::interop::synctex::synctex_node_visible_v;
use crate::common::interop::synctex::synctex_scanner_get_name;
use crate::common::interop::synctex::{
    synctex_display_query, synctex_edit_query, synctex_node_box_visible_depth,
    synctex_node_box_visible_h, synctex_node_box_visible_height, synctex_node_box_visible_v,
    synctex_node_box_visible_width, synctex_node_column, synctex_node_line, synctex_node_p,
    synctex_node_page, synctex_scanner_new_with_output_file, synctex_scanner_next_result,
};
use crate::common::interop::synctex::{synctex_node_tag, synctex_scanner_free};
use crate::common::utils::rest::http_client;
use crate::common::zip::compress::gen_zip;
use crate::common::zip::decompress::exact_upload_zip;
use crate::diesel::RunQueryDsl;
use crate::external::github::repo::repo_file::get_github_repo_size;
use crate::external::github::repo::repo_file::repo_file_exists;
use crate::model::app::tpl_params::ProjDynParams;
use crate::model::dict::proj_source_type::ProjSourceType;
use crate::model::dict::tex_file_compile_status::TeXFileCompileStatus;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::custom::file::search_file::SearchFile;
use crate::model::diesel::custom::project::folder::folder_add::FolderAdd;
use crate::model::diesel::custom::project::proj_type::ProjType;
use crate::model::diesel::custom::project::queue::compile_queue_add::CompileQueueAdd;
use crate::model::diesel::custom::project::tex_proj_editor_add::TexProjEditorAdd;
use crate::model::diesel::custom::project::tex_project_cache::TexProjectCache;
use crate::model::diesel::custom::project::upload::full_proj_upload::FullProjUpload;
use crate::model::diesel::custom::project::upload::github_proj_sync::GithubProjSync;
use crate::model::diesel::custom::project::upload::proj_pdf_upload_file::ProjPdfUploadFile;
use crate::model::diesel::custom::project::upload::proj_upload_file::ProjUploadFile;
use crate::model::diesel::tex::custom_tex_models::TexProjFolder;
use crate::model::diesel::tex::custom_tex_models::TexProjFolderMap;
use crate::model::diesel::tex::custom_tex_models::TexUserConfig;
use crate::model::diesel::tex::custom_tex_models::{
    TexCompQueue, TexProjEditor, TexProject, TexTemplate,
};
use crate::model::error::texhub_error::TexhubError;
use crate::model::request::project::add::copy_proj_req::CopyProjReq;
use crate::model::request::project::add::tex_folder_req::TexFolderReq;
use crate::model::request::project::add::tex_project_req::TexProjectReq;
use crate::model::request::project::del::del_folder_req::DelFolderReq;
use crate::model::request::project::edit::archive_proj_req::ArchiveProjReq;
use crate::model::request::project::edit::edit_proj_nickname::EditProjNickname;
use crate::model::request::project::edit::edit_proj_req::EditProjReq;
use crate::model::request::project::edit::rename_proj_folder::RenameProjFolder;
use crate::model::request::project::edit::trash_proj_req::TrashProjReq;
use crate::model::request::project::query::download_proj::DownloadProj;
use crate::model::request::project::query::folder_proj_params::FolderProjParams;
use crate::model::request::project::query::get_pdf_pos_params::GetPdfPosParams;
use crate::model::request::project::query::get_src_pos_params::GetSrcPosParams;
use crate::model::request::project::query::proj_list_query_params::ProjListQueryParams;
use crate::model::request::project::query::proj_query_params::ProjQueryParams;
use crate::model::request::project::query::search_proj_params::SearchProjParams;
use crate::model::request::project::queue::queue_req::QueueReq;
use crate::model::request::project::queue::queue_status_req::QueueStatusReq;
use crate::model::request::project::tex_compile_project_req::TexCompileProjectReq;
use crate::model::request::project::tex_compile_queue_log::TexCompileQueueLog;
use crate::model::request::project::tex_compile_queue_req::TexCompileQueueReq;
use crate::model::request::project::tex_compile_queue_status::TexCompileQueueStatus;
use crate::model::request::project::tex_join_project_req::TexJoinProjectReq;
use crate::model::response::project::compile_resp::CompileResp;
use crate::model::response::project::latest_compile::LatestCompile;
use crate::model::response::project::pdf_pos_resp::PdfPosResp;
use crate::model::response::project::src_pos_resp::SrcPosResp;
use crate::model::response::project::tex_proj_resp::TexProjResp;
use crate::net::render_client::{construct_headers, render_request};
use crate::net::y_websocket_client::initial_file_request;
use crate::service::config::user_config_service::get_user_config;
use crate::service::file::file_service::{
    get_cached_file_by_fid, get_file_tree, get_main_file_list,
};
use crate::service::global::proj::proj_util::get_purge_proj_base_dir;
use crate::service::global::proj::proj_util::{
    get_proj_base_dir, get_proj_base_dir_instant, get_proj_compile_req, get_proj_log_name,
};
use crate::service::project::project_editor_service::get_default_proj_ids;
use crate::service::project::project_queue_service::get_proj_working_queue_list;
use crate::{common::database::get_connection, model::diesel::tex::custom_tex_models::TexFile};
use actix_web::HttpResponse;
use actix_web::Responder;
use diesel::result::Error;
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
};
use futures_util::{StreamExt, TryStreamExt};
use log::info;
use log::{error, warn};
use meilisearch_sdk::search::*;
use reqwest::Client;
use rust_wheel::common::infra::user::rd_user::get_user_info;
use rust_wheel::common::util::model_convert::map_entity;
use rust_wheel::common::util::net::sse_message::SSEMessage;
use rust_wheel::common::util::rd_file_util::copy_dir_recursive;
use rust_wheel::common::util::rd_file_util::{
    create_directory_if_not_exists, get_filename_without_ext, join_paths,
};
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::common::wrapper::actix_http_resp::box_err_actix_rest_response;
use rust_wheel::common::wrapper::actix_http_resp::{
    box_actix_rest_response, box_error_actix_rest_response,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::config::cache::redis_util::{
    del_redis_key, get_str_default, push_to_stream, set_value,
};
use rust_wheel::model::error::infra_error::InfraError;
use rust_wheel::model::response::api_response::ApiResponse;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use rust_wheel::model::user::rd_user_info::RdUserInfo;
use rust_wheel::texhub::proj::compile_result::CompileResult;
use rust_wheel::texhub::project::{get_proj_path, get_proj_relative_path};
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

pub struct TexProjectService {}

impl ProjSpec for TexProjectService {
    fn get_proj_count_by_uid(&self, uid: &i64) -> i64 {
        use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
        let cr: Result<i64, Error> = tex_project
            .filter(user_id.eq(uid))
            .count()
            .get_result(&mut get_connection());
        cr.unwrap()
    }

    fn get_proj_list(
        &self,
        query_params: &ProjListQueryParams,
        login_user_info: &LoginUserInfo,
    ) -> Vec<TexProject> {
        use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
        let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
        query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
        if query_params.proj_source_type.is_some() {
            query = query
                .filter(cv_work_table::proj_source_type.eq(query_params.proj_source_type.unwrap()));
        }
        if query_params.name.is_some() {
            query = query.filter(cv_work_table::proj_name.eq(query_params.name.as_ref().unwrap()))
        }
        let cvs = query.load::<TexProject>(&mut get_connection());
        if let Err(e) = cvs {
            error!("get proj list failed: {}", e);
            return Vec::new();
        }
        return cvs.unwrap();
    }

    fn get_proj_by_type(
        &self,
        query_params: &ProjQueryParams,
        login_user_info: &LoginUserInfo,
        default_folder: Option<&TexProjFolder>,
    ) -> Vec<TexProjResp> {
        use crate::model::diesel::tex::tex_schema::tex_proj_editor as proj_editor_table;
        let mut query = proj_editor_table::table.into_boxed::<diesel::pg::Pg>();
        if query_params.role_id.is_some() {
            let rid = query_params.role_id.unwrap();
            query = query.filter(proj_editor_table::role_id.eq(rid));
        }
        query = query.filter(proj_editor_table::proj_status.eq(query_params.proj_status));
        query = query.filter(proj_editor_table::user_id.eq(login_user_info.userId));
        let editors: Vec<TexProjEditor> = query
            .load::<TexProjEditor>(&mut get_connection())
            .expect("get project editor failed");
        if editors.len() == 0 {
            return Vec::new();
        }
        let proj_ids: Vec<String> = editors.iter().map(|item| item.project_id.clone()).collect();
        use crate::model::diesel::tex::tex_schema::tex_project as tex_project_table;
        let mut proj_query = tex_project_table::table.into_boxed::<diesel::pg::Pg>();
        proj_query = proj_query.filter(tex_project_table::project_id.eq_any(proj_ids));
        if default_folder.is_some() {
            let folder_proj_ids =
                get_default_folder_proj_ids(query_params, default_folder.unwrap(), login_user_info);
            proj_query = proj_query.filter(tex_project_table::project_id.eq_any(folder_proj_ids));
        }
        if query_params.proj_source_type.is_some() {
            proj_query = proj_query.filter(
                tex_project_table::proj_source_type.eq(query_params.proj_source_type.unwrap()),
            );
        }
        let projects: Vec<TexProject> = proj_query
            .load::<TexProject>(&mut get_connection())
            .expect("get project editor failed");
        let mut proj_resp: Vec<TexProjResp> = map_entity(projects);
        proj_resp.iter_mut().for_each(|item1| {
            if let Some(item2) = editors
                .iter()
                .find(|item2| item1.project_id == item2.project_id)
            {
                item1.role_id = item2.role_id.clone();
            }
        });
        return proj_resp;
    }
}

pub fn get_proj_folders(
    query_params: &ProjQueryParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexProjFolder> {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder as proj_folder_table;
    let mut query = proj_folder_table::table.into_boxed::<diesel::pg::Pg>();
    query = query
        .order_by(proj_folder_table::created_time.desc())
        .filter(proj_folder_table::user_id.eq(login_user_info.userId));
    query = query.filter(proj_folder_table::proj_type.eq(query_params.proj_status));
    let cvs = query.load::<TexProjFolder>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get proj folder failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_folder_project_impl(
    query_params: &FolderProjParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexProject> {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder_map as folder_map;
    let mut map_query = folder_map::table.into_boxed::<diesel::pg::Pg>();
    map_query = map_query.filter(folder_map::user_id.eq(login_user_info.userId));
    map_query = map_query.filter(folder_map::folder_id.eq(query_params.folder_id));
    let folder_maps = map_query
        .load::<TexProjFolderMap>(&mut get_connection())
        .expect("get map failed");
    let proj_ids: Vec<String> = folder_maps
        .iter()
        .map(|item| item.project_id.clone())
        .collect();
    if proj_ids.len() == 0 {
        return Vec::new();
    }
    let curr_tab_proj_ids = get_default_proj_ids(login_user_info.userId, &proj_ids);
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    query = query.filter(cv_work_table::project_id.eq_any(curr_tab_proj_ids));
    let cvs = query.load::<TexProject>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get projects failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_default_folder_proj_ids(
    query_params: &ProjQueryParams,
    default_folder: &TexProjFolder,
    login_user_info: &LoginUserInfo,
) -> Vec<String> {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder_map as proj_folder_map_table;
    let mut query = proj_folder_map_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(proj_folder_map_table::folder_id.eq(default_folder.id));
    query = query.filter(proj_folder_map_table::user_id.eq(login_user_info.userId));
    query = query.filter(proj_folder_map_table::proj_type.eq(query_params.proj_status));
    let editors: Vec<TexProjFolderMap> = query
        .load::<TexProjFolderMap>(&mut get_connection())
        .expect("get default project folder map failed");
    if editors.len() == 0 {
        return Vec::new();
    }
    let proj_ids: Vec<String> = editors.iter().map(|item| item.project_id.clone()).collect();
    return proj_ids;
}

pub fn get_prj_by_id(proj_id: &String) -> Option<TexProject> {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(proj_id));
    let cvs: Vec<TexProject> = query.load::<TexProject>(&mut get_connection()).unwrap();
    if cvs.len() > 1 {
        error!("wrong project count, project id: {}", proj_id);
        return None;
    }
    return if cvs.len() == 1 {
        Some(cvs[0].clone())
    } else {
        None
    };
}

pub fn edit_proj(edit_req: &EditProjReq) -> TexProject {
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_project::project_id
        .eq(edit_req.project_id.clone());
    let update_result = diesel::update(tex_project.filter(predicate))
        .set(proj_name.eq(&edit_req.proj_name))
        .get_result::<TexProject>(&mut get_connection())
        .expect("unable to update tex project");
    return update_result;
}

pub fn rename_proj_collection_folder(
    edit_req: &RenameProjFolder,
    login_user_info: &LoginUserInfo,
) -> TexProjFolder {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder as cv_work_table;
    use crate::model::diesel::tex::tex_schema::tex_proj_folder::dsl::*;
    let predicate = cv_work_table::id
        .eq(edit_req.folder_id.clone())
        .and(cv_work_table::user_id.eq(login_user_info.userId));
    let update_result = diesel::update(tex_proj_folder.filter(predicate))
        .set((
            folder_name.eq(&edit_req.folder_name),
            updated_time.eq(get_current_millisecond()),
        ))
        .get_result::<TexProjFolder>(&mut get_connection())
        .expect("unable to rename project folder name");
    return update_result;
}

pub fn del_proj_collection_folder(del_req: &DelFolderReq, login_user_info: &LoginUserInfo) {
    let mut connection = get_connection();
    let trans_result =
        connection.transaction(|_connection| do_folder_del(del_req, login_user_info));
    match trans_result {
        Ok(_) => {}
        Err(e) => {
            error!("delete project collection folder failed,error:{}", e);
        }
    }
}

pub async fn do_proj_copy(cp_req: &CopyProjReq, login_user_info: &LoginUserInfo) -> impl Responder {
    let ps = TexProjectService {};
    let project_count = ps.get_proj_count_by_uid(&login_user_info.userId);
    if project_count > 2 && login_user_info.vipExpireTime < get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::NonVipTooMuchProj);
    }
    if project_count > 50 && login_user_info.vipExpireTime > get_current_millisecond() {
        return box_err_actix_rest_response(TexhubError::VipTooMuchProj);
    }
    let project = create_cp_project(cp_req, login_user_info).await;
    match project {
        Ok(proj) => box_actix_rest_response(proj),
        Err(e) => {
            let err = format!("create copy project failed,{}", e);
            box_error_actix_rest_response(err.clone(), "CREATE_PROJ_FAILED".to_owned(), err)
        }
    }
}

pub fn do_folder_del(
    del_req: &DelFolderReq,
    login_user_info: &LoginUserInfo,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder as cv_work_table;
    // use crate::model::diesel::tex::tex_schema::tex_proj_folder::dsl::*;
    let predicate = cv_work_table::id
        .eq(del_req.folder_id.clone())
        .and(cv_work_table::user_id.eq(login_user_info.userId));
    let delete_result = diesel::delete(cv_work_table::dsl::tex_proj_folder.filter(predicate))
        .execute(&mut get_connection());

    // use crate::model::diesel::tex::tex_schema::tex_project as proj_table;

    //let update_proj_predicate = proj_table::folder_id
    //    .eq(del_req.folder_id.clone())
    //    .and(cv_work_table::user_id.eq(login_user_info.userId));
    //let update_result = diesel::update(tex_proj_folder.filter(predicate))
    //   .set(id.eq(&del_req.folder_id))
    //    .get_result::<TexProjFolder>(&mut get_connection())
    //   .expect("unable to rename project folder name");
    return delete_result;
}

pub async fn create_empty_project(
    proj_req: &TexProjectReq,
    login_user_info: &LoginUserInfo,
) -> Result<TexProject, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let trans_result = connection.transaction(|connection| {
        do_create_proj_trans(proj_req, &user_info, connection, login_user_info)
    });
    return trans_result;
}

pub async fn create_tpl_project(
    tex_tpl: &TexTemplate,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let trans_result = connection.transaction(|connection| {
        do_create_tpl_proj_trans(&tex_tpl, &user_info, connection, login_user_info)
    });
    return trans_result;
}

pub async fn create_cp_project(
    cp_req: &CopyProjReq,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let proj = get_cached_proj_info(&cp_req.project_id).unwrap();
    let copied_proj_name = format!("{}{}", proj.main.proj_name, "(Copy)");
    let proj_req: TexProjectReq = TexProjectReq {
        name: copied_proj_name,
        template_id: None,
        folder_id: Some(cp_req.folder_id),
        legacy_proj_id: Some(cp_req.project_id.clone()),
        proj_source_type: Some(ProjSourceType::Copied as i16),
        proj_source: Some(proj.main.project_id.clone()),
    };
    let main_name: String = proj.main_file.name.clone();
    let trans_result = connection.transaction(|connection| {
        do_copy_proj_trans(
            &main_name,
            &proj_req,
            &user_info,
            connection,
            login_user_info,
        )
    });
    return trans_result;
}

fn do_create_proj_trans(
    proj_req: &TexProjectReq,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    login_user_info: &LoginUserInfo,
) -> Result<TexProject, Error> {
    let uid: i64 = rd_user_info.id;
    let create_result = create_proj(proj_req, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    do_create_proj_dependencies(proj_req, rd_user_info, connection, &proj);
    let result = create_main_file(&proj.project_id, connection, &uid);
    match result {
        Ok(file) => {
            let file_create_proj = proj.clone();
            let u_copy = login_user_info.clone();
            task::spawn(async move {
                sync_file_to_yjs(&file_create_proj, &file.file_id, &u_copy).await;
            });
        }
        Err(e) => {
            error!("create main file failed,{}", e)
        }
    }
    return Ok(proj);
}

fn do_create_tpl_proj_trans(
    tpl: &TexTemplate,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let proj_req: TexProjectReq = TexProjectReq {
        name: tpl.name.clone(),
        template_id: Some(tpl.template_id),
        folder_id: None,
        legacy_proj_id: None,
        proj_source_type: Some(ProjSourceType::TeXHubTemplate as i16),
        proj_source: Some(tpl.id.to_string()),
    };
    let create_result = create_proj(&proj_req, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    do_create_proj_dependencies(&proj_req, rd_user_info, connection, &proj);
    do_create_proj_on_disk(&tpl, &proj, rd_user_info, login_user_info);
    return Ok(Some(proj));
}

fn do_copy_proj_trans(
    main_name: &String,
    cp_req: &TexProjectReq,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let proj_req: TexProjectReq = TexProjectReq {
        name: cp_req.name.clone(),
        template_id: None,
        folder_id: cp_req.folder_id,
        legacy_proj_id: cp_req.legacy_proj_id.clone(),
        proj_source_type: cp_req.proj_source_type,
        proj_source: cp_req.proj_source.clone(),
    };
    let create_result = create_proj(&proj_req, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create copy proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    do_create_proj_dependencies(&proj_req, rd_user_info, connection, &proj);
    let legacy_proj_id = cp_req.legacy_proj_id.as_ref().unwrap().clone();
    do_create_copied_proj_on_disk(
        &legacy_proj_id,
        &proj,
        main_name,
        rd_user_info,
        login_user_info,
    );
    return Ok(Some(proj));
}

pub fn do_create_copied_proj_on_disk(
    legacy_proj_id: &String,
    proj: &TexProject,
    main_name: &String,
    rd_user_info: &RdUserInfo,
    login_user_info: &LoginUserInfo,
) {
    let uid: i64 = rd_user_info.id;
    let create_res = create_copied_proj_files(
        legacy_proj_id,
        &proj.project_id,
        main_name,
        &uid,
        login_user_info,
    );
    if !create_res {
        error!("create project files failed, project: {:?}", proj);
        return;
    }
}

pub fn do_create_proj_on_disk(
    tpl: &TexTemplate,
    proj: &TexProject,
    rd_user_info: &RdUserInfo,
    login_user_info: &LoginUserInfo,
) {
    let create_res = create_proj_files(tpl, &proj.project_id, &rd_user_info.id, login_user_info);
    if !create_res {
        error!(
            "create project files failed,tpl: {:?}, project: {:?}",
            tpl, proj
        );
        return;
    }
}

pub fn create_copied_proj_files(
    legacy_proj_id: &String,
    proj_id: &String,
    main_name: &String,
    uid: &i64,
    login_user_info: &LoginUserInfo,
) -> bool {
    let legacy_proj_dir = get_proj_base_dir(legacy_proj_id);
    let proj_dir = get_proj_base_dir_instant(&proj_id);
    match create_directory_if_not_exists(&proj_dir) {
        Ok(()) => {}
        Err(e) => error!("create copied project directory failed,{}", e),
    }
    let result = copy_dir_recursive(&legacy_proj_dir.as_str(), &proj_dir);
    if let Err(e) = result {
        error!(
            "copy file failed,{}, legacy project dir: {}, project dir: {}",
            e, legacy_proj_dir, proj_dir
        );
        return false;
    }
    return create_files_into_db(&proj_dir, proj_id, uid, main_name, login_user_info);
}

pub fn create_proj_files(
    tpl: &TexTemplate,
    proj_id: &String,
    uid: &i64,
    login_user_info: &LoginUserInfo,
) -> bool {
    let tpl_base_files_dir = get_app_config("texhub.tpl_files_base_dir");
    let tpl_files_dir = join_paths(&[tpl_base_files_dir, tpl.template_id.to_string()]);
    let proj_dir = get_proj_base_dir_instant(&proj_id);
    match create_directory_if_not_exists(&proj_dir) {
        Ok(()) => {}
        Err(e) => error!("create project directory before tpl copy failed,{}", e),
    }
    let result = copy_dir_recursive(&tpl_files_dir.as_str(), &proj_dir);
    if let Err(e) = result {
        error!(
            "copy file failed,{}, tpl dir: {}, project dir: {}",
            e, tpl_files_dir, proj_dir
        );
        return false;
    }
    return create_files_into_db(
        &proj_dir,
        proj_id,
        uid,
        &tpl.main_file_name,
        login_user_info,
    );
}

pub fn support_sync(file_full_path: &String) -> bool {
    let path = Path::new(file_full_path);
    let extension = path.extension();
    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
    let name_without_ext = get_filename_without_ext(&file_name);
    if name_without_ext == "LICENSE"
        || name_without_ext == "Makefile"
        || name_without_ext == "README"
    {
        return true;
    }
    let sync_file_types = get_app_config("texhub.yjs_sync_file_type");
    let sync_type_array: Vec<&str> = sync_file_types.split(',').collect();
    if extension.is_some() {
        let ext_str = extension.unwrap().to_str().unwrap();
        if sync_type_array.contains(&ext_str) {
            return true;
        }
    }
    return false;
}

pub fn copy_files_in_db() -> bool {
    return true;
}

async fn sync_file_to_yjs(proj: &TexProject, file_id: &String, login_user_info: &LoginUserInfo) {
    let file_folder = get_proj_base_dir(&proj.project_id);
    match create_directory_if_not_exists(&file_folder) {
        Ok(()) => {}
        Err(e) => error!("create directory failed,{}", e),
    }
    let default_tex = get_app_config("texhub.default_tex_document");
    initial_file_request(&proj.project_id, file_id, &default_tex, login_user_info).await;
}

fn create_main_file(
    proj_id: &String,
    connection: &mut PgConnection,
    uid: &i64,
) -> Result<TexFile, diesel::result::Error> {
    let new_proj = TexFileAdd::gen_tex_main(proj_id, uid);
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let result = diesel::insert_into(tex_file)
        .values(&new_proj)
        .get_result::<TexFile>(connection);
    return result;
}

pub async fn save_proj_file(
    proj_upload: ProjUploadFile,
    login_user_info: &LoginUserInfo,
) -> HttpResponse {
    let proj_id = proj_upload.project_id.clone();
    let parent = proj_upload.parent.clone();
    for tmp_file in proj_upload.files {
        if tmp_file.size > 1024 * 1024 {
            return box_error_actix_rest_response(
                "",
                "001002P001".to_owned(),
                "exceed limit".to_owned(),
            );
        }
        let db_file = get_cached_file_by_fid(&proj_upload.parent).unwrap();
        let store_file_path = get_proj_base_dir(&proj_upload.project_id);
        let f_name = tmp_file.file_name;
        let file_path = join_paths(&[
            store_file_path,
            db_file.file_path.clone(),
            f_name.as_ref().unwrap().to_string(),
        ]);
        // https://stackoverflow.com/questions/77122286/failed-to-persist-temporary-file-cross-device-link-os-error-18
        let temp_path = format!("{}{}", "/tmp/", f_name.as_ref().unwrap().to_string());
        let save_result = tmp_file.file.persist(temp_path.as_str());
        if let Err(e) = save_result {
            error!(
                "Failed to save upload file to disk,{}, file path: {}",
                e, file_path
            );
        }
        let copy_result = fs::copy(&temp_path, &file_path.as_str());
        if let Err(e) = copy_result {
            error!("copy file failed, {}", e);
        } else {
            fs::remove_file(temp_path).expect("remove file failed");
        }
        let create_result = create_proj_file_impl(
            &f_name.unwrap().to_string(),
            login_user_info,
            &proj_id,
            &parent,
            &db_file.file_path,
        );
        if let Err(e) = create_result {
            error!("create project file failed,{}", e);
        }
        del_project_cache(&proj_id).await;
    }
    return box_actix_rest_response("ok");
}

pub async fn save_proj_output(proj_upload: ProjPdfUploadFile) -> HttpResponse {
    let proj_id = proj_upload.project_id.clone();
    for tmp_file in proj_upload.files {
        let f_name = tmp_file.file_name;
        // persist tempfile to /tmp to avoid cross-device link issues
        let temp_path = format!("{}{}", "/tmp/", f_name.as_ref().unwrap_or(&"".to_string()));
        if let Err(e) = tmp_file.file.persist(temp_path.as_str()) {
            error!(
                "Failed to save upload file to disk,{}, file path: {}",
                e, temp_path
            );
            continue;
        }
        let proj_base = get_proj_base_dir(&proj_id);
        let out_dir = join_paths(&[proj_base, "app-compile-output".to_owned()]);
        if let Err(e) = create_directory_if_not_exists(&out_dir) {
            error!("create app-compile-output dir failed,{}", e);
        }
        let dest = join_paths(&[out_dir, f_name.unwrap_or_default()]);
        if let Err(e) = fs::copy(&temp_path, &dest) {
            error!("copy file failed, {}, src: {}, dst: {}", e, temp_path, dest);
        }
        let _ = fs::remove_file(&temp_path);
    }
    return box_actix_rest_response("ok");
}

pub async fn save_full_proj(
    proj_upload: FullProjUpload,
    login_user_info: &LoginUserInfo,
) -> HttpResponse {
    for tmp_file in proj_upload.files {
        if tmp_file.size > 100 * 1024 * 1024 {
            return box_err_actix_rest_response(TexhubError::ExceedMaxUnzipSize);
        }
        let f_name = tmp_file.file_name;
        if let Some(ext) = Path::new(&f_name.clone().unwrap_or_default()).extension() {
            if ext != "zip" {
                return box_err_actix_rest_response(TexhubError::UnexpectFileType);
            }
        } else {
            return box_err_actix_rest_response(TexhubError::UnexpectFileType);
        }
        let ps = TexProjectService {};
        let project_count = ps.get_proj_count_by_uid(&login_user_info.userId);
        if project_count > 2 && login_user_info.vipExpireTime < get_current_millisecond() {
            return box_err_actix_rest_response(TexhubError::NonVipTooMuchProj);
        }
        if project_count > 50 && login_user_info.vipExpireTime > get_current_millisecond() {
            return box_err_actix_rest_response(TexhubError::VipTooMuchProj);
        }
        // https://stackoverflow.com/questions/77122286/failed-to-persist-temporary-file-cross-device-link-os-error-18
        let temp_folder_path = format!("{}{}", "/tmp/", login_user_info.userId);
        let temp_path = format!(
            "{}{}{}",
            temp_folder_path,
            "/",
            f_name.as_ref().unwrap().to_string()
        );
        let create_result = create_directory_if_not_exists(&temp_folder_path);
        if let Err(e) = create_result {
            error!("create temp project folder failed,{}", e);
            return actix_web::error::ErrorInternalServerError("handle upload failed").into();
        }
        let save_result = tmp_file.file.persist(temp_path.as_str());
        if let Err(e) = save_result {
            error!(
                "Failed to save upload project file to disk,{}, file path: {}",
                e, temp_path
            );
        }
        let exact_result = exact_upload_zip(&temp_path, &temp_folder_path.as_str());
        if let Err(e) = exact_result {
            error!("exact file failed, {}", e);
            return actix_web::error::ErrorInternalServerError("exact file failed").into();
        }
        // else {
        //     fs::remove_file(temp_path).expect("remove file failed");
        // }
        let path: PathBuf = PathBuf::from(temp_folder_path.clone());
        let proj_root_file_path = find_file_path(path, "main.tex");
        if proj_root_file_path.is_none() {
            error!("did not found main.tex, path:{}", &temp_folder_path);
            return actix_web::error::ErrorInternalServerError("exact file failed").into();
        }
        let fn_path = f_name.clone().unwrap_or_default();
        let f_name_without_ext = Path::new(&fn_path).file_stem().unwrap().to_string_lossy();
        let main_folder_path = proj_root_file_path
            .unwrap()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();
        // create project from exact result
        let tpl_params = ProjDynParams {
            tpl_id: -1,
            name: f_name_without_ext.to_string(),
            main_file_name: "main.tex".to_owned(),
            tpl_files_dir: main_folder_path,
            proj_source_type: ProjSourceType::LocalImport as i16,
            proj_source: fn_path,
        };
        let create_result = create_project_dyn_params(&tpl_params, &login_user_info).await;
        if let Err(e) = create_result {
            error!("create project failed,{},tpl params:{:?}", e, &tpl_params);
            return actix_web::error::ErrorInternalServerError("create project failed").into();
        }
    }
    return box_actix_rest_response("ok");
}

/**
 * to the next step create project
 * we found the path recursively that contains `main.tex` file as the project root path
 * if the path contains more than one `main.tex`, use the first one as the project root path
 */
fn find_file_path<P: AsRef<Path>>(start_path: P, file_name: &str) -> Option<PathBuf> {
    let start_path = start_path.as_ref();

    fn search(path: &Path, depth: usize, f_name: &str) -> Option<PathBuf> {
        const MAX_DEPTH: usize = 10;
        if depth > MAX_DEPTH {
            return None;
        }
        if path.is_dir() {
            for entry in fs::read_dir(path).expect("Unable to read directory") {
                let entry = entry.expect("Unable to access entry");
                let entry_path = entry.path();
                if entry_path.file_name().map_or(false, |name| name == f_name) {
                    return Some(entry_path);
                }
                if entry_path.is_dir() {
                    if let Some(found) = search(&entry_path, depth + 1, f_name) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    search(start_path, 1, file_name)
}

pub async fn import_from_github_impl(
    sync_info: &GithubProjSync,
    login_user_info: &LoginUserInfo,
) -> HttpResponse {
    if !sync_info.url.starts_with("https://github.com/") {
        return box_err_actix_rest_response(TexhubError::OnlyGithubRepoSupport);
    }
    // get user github token
    let configs: Option<Vec<TexUserConfig>> = get_user_config(&login_user_info.userId);
    if configs.is_none() {
        return box_err_actix_rest_response(TexhubError::UserConfigMissing);
    }
    let config_list = configs.unwrap();
    let github_token = config_list
        .iter()
        .find(|config| config.config_key == "GITHUB_TOKEN");
    if github_token.is_none() {
        error!("github token config is missing,{:?}", config_list);
        return box_err_actix_rest_response(TexhubError::GithubConfigMissing);
    }
    let token_val = github_token.unwrap().config_value.clone();
    let repo_info = get_github_repo_size(&sync_info.url, &token_val).await;
    if repo_info.is_none() {
        return box_err_actix_rest_response(TexhubError::FetchGithubRepoSizeFailed);
    }
    let repo = repo_info.unwrap();
    let max_repo_size = get_app_config("texhub.max_github_repo_size");
    if repo.size.unwrap() > max_repo_size.parse::<u32>().unwrap() {
        return box_err_actix_rest_response(TexhubError::ExceedeGithubRepoSize);
    }
    // check the legacy clone
    let tex_proj_service = TexProjectService {};
    let query_params = ProjListQueryParams {
        proj_source_type: Some(ProjSourceType::GitHubImport as i16),
        name: Some(repo.name.clone()),
    };
    let legacy_proj = tex_proj_service.get_proj_list(&query_params, login_user_info);
    if !legacy_proj.is_empty() {
        return box_err_actix_rest_response(TexhubError::AlreadyClonedThisRepo);
    }
    // check the main file exists in repo
    let main_file_name = sync_info.main_file.clone().unwrap_or("main.tex".to_owned());
    let exists = repo_file_exists(
        &sync_info.url,
        &github_token.unwrap().config_value,
        &main_file_name,
    )
    .await;
    if !exists {
        error!(" the input main file did not exits,{}", &main_file_name);
    }
    // clone project
    let main_folder_path = format!("{}{}/{}", "/tmp/", login_user_info.userId, repo.name);
    let clone_url = add_token_to_url(&sync_info.url, &github_token.unwrap().config_value);
    let clone_repo = clone_github_repo(&clone_url, &main_folder_path.to_owned());
    if clone_repo.is_none() {
        return box_err_actix_rest_response(TexhubError::CloneRepoFailed);
    }
    // check main.tex file
    let proj_root_file_path = find_file_path(&main_folder_path, &main_file_name);
    if proj_root_file_path.is_none() {
        error!("did not found main file, path:{}", &main_folder_path);
        return actix_web::error::ErrorInternalServerError("exact file failed").into();
    }
    // create project
    let tpl_params = ProjDynParams {
        tpl_id: -1,
        name: repo.name,
        main_file_name: main_file_name,
        tpl_files_dir: main_folder_path,
        proj_source_type: ProjSourceType::GitHubImport as i16,
        proj_source: sync_info.url.clone(),
    };
    let create_result = create_project_dyn_params(&tpl_params, &login_user_info).await;
    if let Err(e) = create_result {
        error!("create project failed,{},tpl params:{:?}", e, &tpl_params);
        return actix_web::error::ErrorInternalServerError("create project failed").into();
    }
    return box_actix_rest_response("ok");
}

fn add_token_to_url(url: &str, token: &str) -> String {
    let tokenized_url = url.replace("https://", &format!("https://{}@", token));
    tokenized_url
}

fn create_proj_file_impl(
    file_name: &String,
    login_user_info: &LoginUserInfo,
    proj_id: &String,
    parent_id: &String,
    relative_file_path: &String,
) -> Result<TexFile, Error> {
    let new_proj = TexFileAdd::gen_upload_tex_file(
        &file_name,
        login_user_info,
        proj_id,
        parent_id,
        relative_file_path,
    );
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let result = diesel::insert_into(tex_file)
        .values(&new_proj)
        .get_result::<TexFile>(&mut get_connection());
    return result;
}

pub async fn get_pdf_pos(params: &GetPdfPosParams) -> Vec<PdfPosResp> {
    let url_path = "/tex/project/pos/pdf";
    let url = format!("{}{}", get_app_config("texhub.render_api_url"), url_path);
    let proj: Option<TexProjectCache> = get_cached_proj_info(&params.project_id);

    let response = http_client()
        .get(&url)
        .headers(construct_headers())
        .query(&[
            ("project_id", &params.project_id),
            ("path", &params.path),
            ("file", &params.file),
            ("main_file", &params.main_file),
            ("line", &params.line.to_string()),
            ("column", &params.column.to_string()),
            ("created_time", &proj.unwrap().main.created_time.to_string()),
        ])
        .send()
        .await;

    match response {
        Ok(r) => {
            if !r.status().is_success() {
                error!(
                    "get pdf pos failed, status: {}, msg: {}",
                    r.status(),
                    r.text().await.unwrap_or_default()
                );
                return Vec::new();
            }
            let rtxt = r.text().await;
            info!(
                "pdf pos response content: {}",
                rtxt.as_ref().unwrap_or(&"".to_string())
            );
            let resp: Result<ApiResponse<Vec<PdfPosResp>>, serde_json::Error> =
                serde_json::from_str(rtxt.as_ref().unwrap_or(&"".to_string()));
            match resp {
                Ok(position_list) => {
                    return position_list.result;
                }
                Err(e) => {
                    error!("parse pdf pos response failed: {}", e);
                    return Vec::new();
                }
            }
        }
        Err(e) => {
            error!("request pdf pos error: {}", e);
            return Vec::new();
        }
    }
}

pub async fn get_src_pos(params: &GetSrcPosParams) -> Vec<SrcPosResp> {
    let url_path = "/tex/project/pos/src";
    let url = format!("{}{}", get_app_config("texhub.render_api_url"), url_path);
    let proj: Option<TexProjectCache> = get_cached_proj_info(&params.project_id);

    let response = http_client()
        .get(&url)
        .headers(construct_headers())
        .query(&[
            ("project_id", &params.project_id),
            ("main_file", &params.main_file),
            ("page", &params.page.to_string()),
            ("h", &params.h.to_string()),
            ("v", &params.v.to_string()),
            ("created_time", &proj.unwrap().main.created_time.to_string()),
        ])
        .send()
        .await;

    match response {
        Ok(r) => {
            if !r.status().is_success() {
                error!(
                    "get src pos failed, status: {}, msg: {}",
                    r.status(),
                    r.text().await.unwrap_or_default()
                );
                return Vec::new();
            }
            let resp: Result<ApiResponse<Vec<SrcPosResp>>, reqwest::Error> = r.json().await;
            match resp {
                Ok(position_list) => {
                    return position_list.result;
                }
                Err(e) => {
                    error!("parse src pos response failed: {}", e);
                    return Vec::new();
                }
            }
        }
        Err(e) => {
            error!("request src pos error: {}", e);
            return Vec::new();
        }
    }
}

pub async fn join_project(
    req: &TexJoinProjectReq,
    login_user_info: &LoginUserInfo,
) -> Result<TexProjEditor, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let new_proj_editor = TexProjEditorAdd::from_req(
        &req.project_id,
        &login_user_info.userId,
        2,
        &user_info.nickname,
    );
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let result = diesel::insert_into(tex_proj_editor)
        .values(&new_proj_editor)
        .get_result::<TexProjEditor>(&mut get_connection());
    return result;
}

pub async fn del_project_cache(del_project_id: &String) {
    let cache_key: String = format!(
        "{}:{}",
        get_app_config("texhub.proj_cache_key"),
        del_project_id
    );
    let del_result = del_redis_key(cache_key.as_str());
    if let Err(e) = del_result {
        error!("delete project cache failed,{},cached key:{}", e, cache_key);
    }
}

pub fn del_project_logic(
    del_project_id: &String,
    login_user_info: &LoginUserInfo,
) -> TexProjEditor {
    let delete_result = logic_del_project_impl(del_project_id, login_user_info);
    return delete_result;
}

pub fn del_project(del_project_id: &String, login_user_info: &LoginUserInfo) {
    let mut connection = get_connection();
    let result = connection.transaction(|connection| {
        let legacy_proj = get_prj_by_id(&del_project_id);
        let delete_result = del_project_impl(del_project_id, connection, login_user_info);
        match delete_result {
            Ok(rows) => {
                if rows == 0 {
                    warn!(
                        "the delete project effect {} rows, project id: {}",
                        rows, del_project_id
                    );
                }
                if rows == 1 {
                    del_project_file(del_project_id, &login_user_info.userId, connection);
                    let async_proj_id = del_project_id.clone();
                    task::spawn_blocking({
                        move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(del_project_disk_file(
                                &async_proj_id,
                                legacy_proj.unwrap().created_time,
                            ));
                        }
                    });
                }
                Ok("")
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    match result {
        Ok(_) => {}
        Err(e) => {
            error!(
                "transaction failed, project id: {},error:{}",
                del_project_id, e
            );
        }
    }
}

pub async fn del_project_disk_file(proj_id: &String, created_time: i64) {
    if proj_id.is_empty() {
        error!("delete project id is null");
        return;
    }
    let proj_dir = get_purge_proj_base_dir(proj_id, created_time);
    let result = tokio::fs::remove_dir_all(Path::new(&proj_dir)).await;
    match result {
        Ok(_) => {}
        Err(e) => {
            error!("delete project from disk failed,{}", e)
        }
    }
}

pub fn del_project_impl(
    del_project_id: &String,
    connection: &mut PgConnection,
    login_user_info: &LoginUserInfo,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_project as tex_project_table;
    let predicate = tex_project_table::project_id
        .eq(del_project_id)
        .and(tex_project_table::user_id.eq(login_user_info.userId));
    let delete_result =
        diesel::delete(tex_project_table::dsl::tex_project.filter(predicate)).execute(connection);
    return delete_result;
}

pub fn logic_del_project_impl(
    del_project_id: &String,
    login_user_info: &LoginUserInfo,
) -> TexProjEditor {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as tex_project_table;
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let predicate = tex_project_table::project_id
        .eq(del_project_id)
        .and(tex_project_table::user_id.eq(login_user_info.userId));
    let delete_result = diesel::update(tex_proj_editor.filter(predicate))
        .set(proj_status.eq(ProjType::Deleted as i32))
        .get_result::<TexProjEditor>(&mut get_connection())
        .expect("update project status facing error");
    return delete_result;
}

pub fn del_project_file(del_project_id: &String, uid: &i64, connection: &mut PgConnection) {
    let del_command = format!(
        "WITH RECURSIVE x AS (
            SELECT file_id
            FROM   tex_file
            WHERE parent = '{}'
            AND user_id = '{}'
        
            UNION  ALL
            SELECT a.file_id
            FROM   x
            JOIN   tex_file a ON a.parent = x.file_id
            )
         DELETE FROM tex_file a
         USING  x
         WHERE a.file_id = x.file_id",
        del_project_id, uid
    );
    let cte_menus = sql_query(&del_command).load::<TexFile>(connection);
    match cte_menus {
        Ok(_) => {}
        Err(e) => {
            error!(
                "delete project file failed, project id: {}, command:{},error info:{}",
                del_project_id, del_command, e
            );
        }
    }
}

pub async fn compile_project(params: &TexCompileProjectReq) -> Option<serde_json::Value> {
    return render_request(params).await;
}

pub async fn compile_status_update(params: &TexCompileQueueStatus) -> HttpResponse {
    use crate::model::diesel::tex::tex_schema::tex_comp_queue::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_comp_queue::id.eq(params.id.clone());
    let completed = match params.comp_status {
        2 => get_current_millisecond(),
        _ => 0,
    };
    let update_result = diesel::update(tex_comp_queue.filter(predicate))
        .set((
            comp_status.eq(params.comp_status),
            comp_result.eq(params.comp_result),
            complete_time.eq(completed),
        ))
        .get_result::<TexCompQueue>(&mut get_connection());
    if let Err(e) = update_result {
        let err_msg = format!(
            "update compile queue failed, error info:{},params:{:?}",
            e, params
        );
        error!("{}", err_msg);
        return box_error_actix_rest_response("", "UPDATE_QUEUE_FAILED".to_owned(), err_msg);
    }
    let q = update_result.unwrap();
    if let Some(resp) = cache_queue(&q) {
        return resp;
    }
    return box_actix_rest_response(q);
}

pub async fn add_compile_to_queue(
    params: &TexCompileQueueReq,
    login_user_info: &LoginUserInfo,
) -> HttpResponse {
    let mut connection = get_connection();
    let queue_req = QueueReq {
        comp_status: vec![
            TeXFileCompileStatus::Waiting as i32,
            TeXFileCompileStatus::Compiling as i32,
        ],
        project_id: params.project_id.clone(),
    };
    let queue_list = get_proj_working_queue_list(&queue_req, login_user_info);
    if !queue_list.is_empty() {
        return box_err_actix_rest_response(TexhubError::CompilingPocessing);
    }
    let new_compile = CompileQueueAdd::from_req(&params.project_id, &login_user_info.userId);
    use crate::model::diesel::tex::tex_schema::tex_comp_queue::dsl::*;
    let queue_result = diesel::insert_into(tex_comp_queue)
        .values(&new_compile)
        .get_result::<TexCompQueue>(&mut connection);
    if let Err(e) = queue_result {
        error!("add compile queue failed, error info:{}", e);
        return box_error_actix_rest_response(
            "",
            "QUEUE_ADD_FAILED".to_string(),
            "queue add failed".to_string(),
        );
    }
    let proj_cache = get_cached_proj_info(&params.project_id);
    let main_name = proj_cache.clone().unwrap().main_file.name;
    let log_file_name = format!("{}{}", get_filename_without_ext(&main_name), ".log");
    let compile_base_dir = get_app_config("texhub.compile_base_dir");
    let proj_base_dir = get_proj_path(
        &compile_base_dir,
        proj_cache.clone().unwrap().main.created_time,
    );
    let stream_key = get_app_config("texhub.compile_stream_redis_key");
    let file_path = join_paths(&[
        proj_base_dir.clone(),
        params.project_id.clone(),
        main_name.clone(),
    ]);
    let out_path = join_paths(&[proj_base_dir, params.project_id.clone()]);
    let rt = get_current_millisecond().to_string();
    let qid = queue_result.as_ref().unwrap().id.to_string();
    let proj_created_time = proj_cache.clone().unwrap().main.created_time;
    let created_time_str = proj_created_time.to_string();
    let user_id_str = login_user_info.userId.to_string();
    let s_params = [
        ("file_path", file_path.as_str()),
        ("out_path", out_path.as_str()),
        ("project_id", params.project_id.as_str()),
        ("req_time", rt.as_str()),
        ("qid", qid.as_str()),
        (
            "version_no",
            queue_result.as_ref().unwrap().version_no.as_str(),
        ),
        ("log_file_name", log_file_name.as_str()),
        ("proj_created_time", created_time_str.as_str()),
        ("user_id", &user_id_str.as_str()),
    ];
    let p_result = push_to_stream(&stream_key.as_str(), &s_params);
    if let Err(pe) = p_result {
        error!("push to stream failed,{}", pe);
        return box_error_actix_rest_response(
            "push stream failed",
            "QUEUE_ADD_FAILED".to_string(),
            "queue add failed".to_string(),
        );
    }
    if let Some(resp) = cache_queue(queue_result.as_ref().unwrap()) {
        return resp;
    }
    return box_actix_rest_response(queue_result.unwrap());
}

pub fn cache_queue(queue_result: &TexCompQueue) -> Option<HttpResponse> {
    let queue_status_key = get_app_config("texhub.compile_status_cached_key");
    let full_cached_key = format!("{}:{}", queue_status_key, queue_result.id);
    let queue_str = serde_json::to_string(queue_result);
    let cached_result = set_value(&full_cached_key, queue_str.unwrap().as_str(), 86400);
    if let Err(ce) = cached_result {
        error!("set queue value failed,{}", ce);
        return Some(box_error_actix_rest_response(
            "cached queue failed",
            "QUEUE_CACHED_FAILED".to_string(),
            "queue cached failed".to_string(),
        ));
    }
    return None;
}

pub async fn get_compiled_log(req: &TexCompileQueueLog) -> String {
    let log_full_path = get_proj_log_name(&req.project_id).await;
    let mut file = match File::open(&log_full_path) {
        Ok(file) => file,
        Err(error) => {
            error!(
                "Error opening project log file: {:?}, full path: {}",
                error, log_full_path
            );
            return "".to_string();
        }
    };
    let mut contents = String::new();
    if let Err(error) = file.read_to_string(&mut contents) {
        error!("Error reading project log file: {:?}", error);
        return "".to_string();
    }
    return contents;
}

pub async fn get_proj_latest_pdf(proj_id: &String, uid: &i64) -> Result<LatestCompile, InfraError> {
    let proj_info = get_cached_proj_info(proj_id).unwrap();
    let main_file = proj_info.main_file;
    let mut req = Vec::new();
    req.push(CompileResult::Success as i32);
    let newest_queue = get_latest_proj_queue(&req, uid, proj_id);
    let ver_no = if newest_queue.is_some() {
        newest_queue.unwrap().version_no
    } else {
        get_current_millisecond().to_string()
    };
    let pdf_name = format!(
        "{}{}{}",
        get_filename_without_ext(&main_file.name),
        ".pdf?v=",
        ver_no
    );
    let proj_relative_path = get_proj_relative_path(proj_id, proj_info.main.created_time);
    let pdf_result: LatestCompile = LatestCompile {
        path: join_paths(&[proj_relative_path, pdf_name.to_string()]),
        project_id: proj_id.clone(),
        file_name: main_file.name,
    };
    return Ok(pdf_result);
}

pub async fn get_project_pdf(proj_id: &String) -> String {
    let proj_dir = get_proj_base_dir(proj_id);
    if !fs::metadata(&proj_dir).is_ok() {
        error!("folder did not exists, dir: {}", proj_dir);
        return "".to_owned();
    }
    let subdirectories = fs::read_dir(proj_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    // https://stackoverflow.com/questions/76946130/creation-time-is-not-available-on-this-platform-currently
    let latest_directory = subdirectories
        .iter()
        .max_by_key(|&dir| dir.metadata().unwrap().modified().unwrap());

    match latest_directory {
        Some(directory) => {
            let name = directory.file_name().unwrap_or_default().to_str().unwrap();
            return name.to_string();
        }
        None => {
            return "".to_owned();
        }
    }
}

pub async fn send_render_req(
    params: &TexCompileProjectReq,
    tx: UnboundedSender<SSEMessage<String>>,
) -> Result<String, reqwest::Error> {
    let client = Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let url_path = format!("{}", "/render/compile/v1/project/sse");
    let url = format!("{}{}", get_app_config("texhub.render_api_url"), url_path);
    let json_data = get_proj_compile_req(&params.project_id, &params.file_name);
    let resp = client
        .get(url)
        .headers(construct_headers())
        .query(&json_data)
        .send()
        .await?
        .bytes_stream()
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) });
    let mut compile_params: HashMap<String, String> = HashMap::new();
    compile_params.insert("project_id".to_owned(), params.project_id.to_string());
    compile_params.insert("req_time".to_owned(), get_current_millisecond().to_string());
    compile_params.insert(
        "file_path".to_owned(),
        json_data.get("file_path").unwrap().to_string(),
    );
    compile_params.insert(
        "out_path".to_owned(),
        json_data.get("out_path").unwrap().to_string(),
    );
    let mut resp = Box::pin(resp);
    while let Some(item) = resp.next().await {
        let data = item.unwrap();
        let string_content = std::str::from_utf8(&data).unwrap().to_owned();
        let sse_mesg = serde_json::from_str(&string_content);
        if let Err(parse_err) = sse_mesg {
            error!("parse json failed,{}, text:{}", parse_err, string_content);
            continue;
        }
        let send_result = tx.send(sse_mesg.unwrap());
        match send_result {
            Ok(_) => {}
            Err(e) => {
                error!("send xelatex compile log error: {}", e);
            }
        }
    }
    Ok(String::new())
}

pub async fn get_cached_queue_status(qid: i64) -> Option<TexCompQueue> {
    let queue_status_key = get_app_config("texhub.compile_status_cached_key");
    let full_cached_key = format!("{}:{}", queue_status_key, qid);
    let cached_queue_result = get_str_default(&full_cached_key.as_str());
    if let Err(e) = cached_queue_result {
        error!("get cached queue failed,{}", e);
        return None;
    }
    if cached_queue_result.as_ref().unwrap().is_none() {
        return None;
    }
    let queue: Result<TexCompQueue, serde_json::Error> =
        serde_json::from_str(cached_queue_result.unwrap().unwrap().as_str());
    return Some(queue.unwrap());
}

pub async fn save_upload_file(req: &QueueStatusReq) -> Option<TexCompQueue> {
    let queue_status_key = get_app_config("texhub.compile_status_cached_key");
    let full_cached_key = format!("{}:{}", queue_status_key, req.id);
    let cached_queue_result = get_str_default(&full_cached_key.as_str());
    if let Err(e) = cached_queue_result {
        error!("get cached queue failed,{}", e);
        return None;
    }
    if cached_queue_result.as_ref().unwrap().is_none() {
        return None;
    }
    let queue: Result<TexCompQueue, serde_json::Error> =
        serde_json::from_str(cached_queue_result.unwrap().unwrap().as_str());
    return Some(queue.unwrap());
}

pub fn get_cached_proj_info(proj_id: &String) -> Option<TexProjectCache> {
    let cache_key = format!("{}:{}", get_app_config("texhub.proj_cache_key"), proj_id);
    let proj_info_result = get_str_default(&cache_key.as_str());
    if let Err(e) = proj_info_result.as_ref() {
        error!("get cached project info failed,{}", e);
        return None;
    }
    let cached_proj_info = proj_info_result.unwrap();
    if cached_proj_info.is_none() {
        let proj = get_prj_by_id(proj_id);
        if proj.is_none() {
            error!("do not found project info: {}", proj_id);
            return None;
        }
        let file = get_main_file_list(proj_id);
        let file_tree = get_file_tree(proj_id);
        let proj_info = TexProjectCache::from_db(&proj.unwrap(), file.unwrap(), file_tree);
        let proj_cached_json = serde_json::to_string(&proj_info).unwrap();
        let cache_result = set_value(&cache_key.as_str(), &proj_cached_json.as_str(), 86400);
        if let Err(e) = cache_result {
            error!("set cached project info failed,{}", e);
            return None;
        }
        return Some(proj_info);
    }
    let cached_proj = serde_json::from_str(cached_proj_info.as_ref().unwrap().as_str());
    if let Err(e) = cached_proj {
        error!(
            "parse cached project info failed,{},cached project info: {}, pid: {}",
            e,
            cached_proj_info.unwrap(),
            proj_id
        );
        return None;
    }
    return Some(cached_proj.unwrap());
}

pub async fn proj_search_impl(params: &SearchProjParams) -> Option<SearchResults<SearchFile>> {
    let url = get_app_config("texhub.meilisearch_url");
    let api_key = env::var("MEILI_MASTER_KEY");
    if let Err(e) = api_key {
        error!("get meilisearch api key failed {}", e);
        return None;
    }
    let client = meilisearch_sdk::Client::new(url, Some(api_key.unwrap()));
    let movies = client.index("files");
    let proj_search_filter = format!("project_id = {}", &params.project_id);
    let query_word = &params.keyword;
    let query: SearchQuery = SearchQuery::new(&movies)
        .with_query(query_word)
        .with_filter(proj_search_filter.as_str())
        .with_attributes_to_crop(Selectors::Some(&[("content", None)]))
        .with_show_matches_position(true)
        .with_crop_length(12)
        .build();
    let results = client.index("files").execute_query(&query).await;
    match results {
        Ok(r) => {
            return Some(r);
        }
        Err(e) => {
            error!("search failed,{}, params: {:?}", e, params);
            return None;
        }
    }
}

pub async fn handle_update_nickname(edit_nickname: &EditProjNickname) {
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_project::user_id
        .eq(edit_nickname.user_id.clone());
    diesel::update(tex_project.filter(predicate))
        .set(nickname.eq(&edit_nickname.nickname))
        .get_result::<TexProject>(&mut get_connection())
        .expect("unable to update tex project");
}

pub fn handle_archive_proj(req: &ArchiveProjReq, login_user_info: &LoginUserInfo) -> TexProjEditor {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as tex_project_table;
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let predicate = tex_project_table::user_id
        .eq(login_user_info.userId.clone())
        .and(tex_project_table::project_id.eq(req.project_id.clone()));
    let update_result = diesel::update(tex_proj_editor.filter(predicate))
        .set(archive_status.eq(req.archive_status))
        .get_result::<TexProjEditor>(&mut get_connection())
        .expect("unable to update tex project archive status");
    return update_result;
}

pub fn handle_folder_create(req: &TexFolderReq, login_user_info: &LoginUserInfo) -> TexProjFolder {
    use crate::model::diesel::tex::tex_schema::tex_proj_folder::dsl::*;
    let new_folder = FolderAdd::from_req(req, &login_user_info.userId);
    let result = diesel::insert_into(tex_proj_folder)
        .values(&new_folder)
        .get_result::<TexProjFolder>(&mut get_connection())
        .expect("unable to create folder");
    return result;
}

pub fn handle_trash_proj(req: &TrashProjReq, login_user_info: &LoginUserInfo) -> TexProjEditor {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as tex_project_table;
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let predicate = tex_project_table::user_id
        .eq(login_user_info.userId.clone())
        .and(tex_project_table::project_id.eq(req.project_id.clone()));
    let update_result = diesel::update(tex_proj_editor.filter(predicate))
        .set(proj_status.eq(ProjType::Trash as i32))
        .get_result::<TexProjEditor>(&mut get_connection())
        .expect("unable to update tex project archive status");
    return update_result;
}

pub fn handle_compress_proj(req: &DownloadProj) -> String {
    let archive_path = gen_zip(&req.project_id);
    return archive_path;
}
