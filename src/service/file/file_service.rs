use std::env;
use std::fs::{self, File, Metadata};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::common::database::get_connection;
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::custom::file::search_file::SearchFile;
use crate::model::diesel::tex::custom_tex_models::{TexFile, TexFileVersion, TexFolderTree};
use crate::model::request::file::add::file_add_req::TexFileAddReq;
use crate::model::request::file::del::file_del::TexFileDelReq;
use crate::model::request::file::edit::move_file_req::MoveFileReq;
use crate::model::request::file::file_rename::TexFileRenameReq;
use crate::model::request::file::query::file_code_params::FileCodeParams;
use crate::model::request::project::query::get_proj_history_page::GetProjPageHistory;
use crate::model::response::file::file_tree_resp::FileTreeResp;
use crate::model::response::file::folder_tree_resp::FolderTreeResp;
use crate::model::response::project::latest_compile::LatestCompile;
use crate::service::global::proj::proj_util::get_proj_base_dir;
use crate::service::project::project_service::{del_project_cache, del_project_file};
use actix_files::NamedFile;
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::{CacheControl, CacheDirective, ContentDisposition, DispositionType};
use actix_web::{HttpRequest, HttpResponse};
use chrono::Duration;
use diesel::result::Error::{self, NotFound};
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
    QueryResult,
};
use log::error;
use mime::Mime;
use reqwest::header::HeaderValue;
use rust_wheel::common::query::pagination::Paginate;
use rust_wheel::common::util::convert_to_tree_generic::convert_to_tree;
use rust_wheel::common::util::model_convert::{map_entity, map_pagination_res};
use rust_wheel::common::util::rd_file_util::{
    create_folder_not_exists, get_filename_without_ext, join_paths,
};
use rust_wheel::common::wrapper::actix_http_resp::{
    box_actix_rest_response, box_error_actix_rest_response,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::config::cache::redis_util::{del_redis_key, set_value, sync_get_str};
use rust_wheel::model::response::pagination_response::PaginationResponse;
use rust_wheel::model::response::pagination::Pagination;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use rust_wheel::texhub::th_file_type::ThFileType;
use tokio::task;

use super::spec::file_spec::FileSpec;

pub struct TexFileService {}

impl FileSpec for TexFileService {
    fn get_proj_file_count(&self, proj_id: &str) -> i64 {
        use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
        let count = cv_work_table::table
            .filter(cv_work_table::project_id.eq(proj_id))
            .count()
            .get_result(&mut get_connection())
            .unwrap();
        return count;
    }

    fn get_file_by_id(&self, fid: &str) -> Option<TexFile> {
        use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
        let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
        query = query.filter(cv_work_table::file_id.eq(fid));
        let cvs: Result<TexFile, Error> = query.first::<TexFile>(&mut get_connection());
        match cvs {
            Ok(result) => {
                return Some(result);
            }
            Err(err) => {
                error!("get files failed, {}", err);
                return None;
            }
        }
    }
}

pub fn get_cached_file_by_fid(filter_id: &String) -> Option<TexFile> {
    let file_cached_key_prev: String = get_app_config("texhub.fileinfo_redis_key");
    let file_cached_key = format!("{}:{}", file_cached_key_prev, &filter_id);
    let cached_file = sync_get_str(&file_cached_key);
    if cached_file.is_some() {
        let tf = serde_json::from_str(&cached_file.unwrap());
        if let Err(e) = tf {
            error!("parse cached file facing issue,{}", e);
        } else {
            return Some(tf.unwrap());
        }
    }
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(filter_id));
    let files = query.load::<TexFile>(&mut get_connection()).unwrap();
    if files.len() == 0 {
        return None;
    }
    let file = &files[0];
    let file_json = serde_json::to_string(file).unwrap();
    let one_day = Duration::try_days(1);
    let seconds_in_one_day = one_day.unwrap().num_seconds();
    set_value(&file_cached_key, &file_json, seconds_in_one_day as u64).unwrap();
    return Some(file.to_owned());
}

pub fn get_path_content_by_fid(dl_file: &TexFile) -> Option<String> {
    let download_dir = get_proj_base_dir(&dl_file.project_id);
    let archive_file_path = join_paths(&[
        download_dir,
        dl_file.file_path.clone(),
        dl_file.name.clone(),
    ]);
    return Some(archive_file_path);
}

pub fn get_file_by_ids(ids: &Vec<String>) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq_any(ids));
    let cvs: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_file_by_int_ids(ids: &Vec<i64>) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::id.eq_any(ids));
    let cvs: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get files by int id failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_file_list(parent_id: &String) -> Vec<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_main_file_list(project_id: &String) -> Option<TexFile> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(
        cv_work_table::project_id
            .eq(project_id)
            .and(cv_work_table::main_flag.eq(1)),
    );
    let tex_files: Result<Vec<TexFile>, Error> = query.load::<TexFile>(&mut get_connection());
    match tex_files {
        Ok(files) => {
            let result = if files.len() > 0 {
                Some(files[0].to_owned())
            } else {
                error!("get main files null, pid: {}", project_id);
                None
            };
            return result;
        }
        Err(err) => {
            error!("get main files failed, {}", err);
            return None;
        }
    }
}

pub fn get_text_file_code(filter_file_id: &String) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::file_id.eq(filter_file_id));
    let cvs: Result<Vec<_>, Error> = query.load::<TexFile>(&mut get_connection());
    let tex_files: Vec<TexFile> = cvs.unwrap();
    let tex = tex_files[0].clone();
    let file_folder = format!("{}/{}", base_compile_dir, tex.project_id);
    let file_full_path = format!("{}/{}", file_folder, tex.name);
    let mut file = match File::open(file_full_path) {
        Ok(file) => file,
        Err(error) => {
            error!("Error opening file: {:?}", error);
            return "".to_string();
        }
    };
    let mut contents = String::new();
    if let Err(error) = file.read_to_string(&mut contents) {
        error!("Error reading file: {:?}", error);
        return "".to_string();
    }
    return contents;
}

pub fn get_proj_history_page_impl(
    params: &GetProjPageHistory,
) -> PaginationResponse<Vec<TexFileVersion>> {
    use crate::model::diesel::tex::tex_schema::tex_file_version as file_version_table;
    let mut query = file_version_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(file_version_table::project_id.eq(params.project_id.clone()));
    let query = query
        .order_by(file_version_table::created_time.desc())
        .paginate(params.page_num.unwrap_or(1).clone())
        .per_page(params.page_size.unwrap_or(9).clone());
    let page_result: QueryResult<(Vec<TexFileVersion>, i64, i64)> =
        query.load_and_count_pages_total::<TexFileVersion>(&mut get_connection());
    let page_map_result = map_pagination_res(
        page_result,
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(10),
    );
    return page_map_result;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HistoryItem {
    pub created_time: String,
    pub name: String,
    pub diff: String,
    pub id: String,
    pub doc_int_id: i64
}

pub async fn get_proj_history_page_impl_v1(
    params: &GetProjPageHistory,
) -> PaginationResponse<Vec<HistoryItem>> {
    let client = reqwest::Client::new();
    let base_url = get_app_config("texhub.y_websocket_api_url");
    let url = format!(
        "{}/doc/version/proj/scroll?projId={}&page_num={}&page_size={}",
        base_url.trim_end_matches('/'),
        params.project_id,
        params.page_num.unwrap_or(1),
        params.page_size.unwrap_or(10)
    );

    let resp = client.get(&url).send().await;
    let r = match resp {
        Ok(r) => r,
        Err(e) => {
            error!("get_proj_history_page_impl_v1: http request failed, error: {:?}", e);
            return PaginationResponse {
                pagination: Pagination {
                    pageNum: params.page_num.unwrap_or(1),
                    pageSize: params.page_size.unwrap_or(10),
                    total: 0,
                },
                data: vec![],
            };
        }
    };

    let s = match r.text().await {
        Ok(s) => s,
        Err(e) => {
            error!("get_proj_history_page_impl_v1: failed to get response text, error: {:?}", e);
            return PaginationResponse {
                pagination: Pagination {
                    pageNum: params.page_num.unwrap_or(1),
                    pageSize: params.page_size.unwrap_or(10),
                    total: 0,
                },
                data: vec![],
            };
        }
    };

    let json = match serde_json::from_str::<serde_json::Value>(&s) {
        Ok(json) => json,
        Err(e) => {
            error!("get_proj_history_page_impl_v1: json parse error: {:?}, raw: {}", e, s);
            return PaginationResponse {
                pagination: Pagination {
                    pageNum: params.page_num.unwrap_or(1),
                    pageSize: params.page_size.unwrap_or(10),
                    total: 0,
                },
                data: vec![],
            };
        }
    };

    let arr = match json.get("items").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => {
            error!("get_proj_history_page_impl_v1: 'items' field missing or not array, json: {:?}", json);
            return PaginationResponse {
                pagination: Pagination {
                    pageNum: params.page_num.unwrap_or(1),
                    pageSize: params.page_size.unwrap_or(10),
                    total: 0,
                },
                data: vec![],
            };
        }
    };

    let data: Vec<HistoryItem> = arr
        .iter()
        .filter_map(|item| {
            let doc_name = item.get("doc_name")?.as_str()?.to_string();
            let created_time = item.get("created_time")?.as_str()?.to_string();
            let diff = item.get("diff")?.as_str()?.to_string();
            let id = item.get("id")?.as_str()?.to_string();
            let doc_int_id = item.get("doc_int_id")
                .and_then(|v| v.as_i64().or_else(|| v.as_str()?.parse::<i64>().ok()));
            if doc_int_id.is_none() {
                error!("get_proj_history_page_impl_v1: doc_int_id is empty or invalid for item: {:?}", item);
            }
            Some(HistoryItem {
                created_time,
                name: doc_name,
                diff,
                id,
                doc_int_id: doc_int_id.unwrap_or(0)
            })
        })
        .collect();
    let rep_name_datas =  append_file_name(data);
    let total = json.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let pagination = Pagination {
        pageNum: params.page_num.unwrap_or(1),
        pageSize: params.page_size.unwrap_or(10),
        total,
    };
    PaginationResponse {
        pagination,
        data: rep_name_datas,
    }
}

fn append_file_name(data: Vec<HistoryItem>) -> Vec<HistoryItem> {
    let file_ids: Vec<i64> = data
        .iter()
        .map(|item| item.doc_int_id.clone())
        .collect();
    let files = get_file_by_int_ids(&file_ids);
    let file_name_map: std::collections::HashMap<i64, String> = files
        .into_iter()
        .map(|file| (file.id, file.name))
        .collect();
    data.into_iter()
        .map(|mut item| {
            if let Some(file_name) = file_name_map.get(&item.doc_int_id) {
                item.name = file_name.clone();
            }
            item
        })
        .collect()
}

pub async fn create_file(add_req: &TexFileAddReq, login_user_info: &LoginUserInfo) -> HttpResponse {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(
        cv_work_table::parent
            .eq(add_req.parent.clone())
            .and(cv_work_table::name.eq(add_req.name.clone()))
            .and(cv_work_table::file_type.eq(add_req.file_type.clone())),
    );
    let cvs = query.load::<TexFile>(&mut get_connection()).unwrap();
    if !cvs.is_empty() {
        return box_error_actix_rest_response(
            "already exists",
            "RESOURCE_ALREADY_EXISTS".to_owned(),
            "file/folder already exists".to_owned(),
        );
    }
    let gen_file_path = get_file_path(add_req);
    let new_file = TexFileAdd::gen_tex_file(add_req, login_user_info, &gen_file_path);
    let err_msg = format!(
        "failed to add new tex file or folder,{}",
        serde_json::to_string(&new_file).unwrap()
    );
    let result = diesel::insert_into(tex_file)
        .values(&new_file)
        .get_result::<TexFile>(&mut get_connection())
        .expect(&err_msg);
    create_file_on_disk(&result).await;
    del_project_cache(&add_req.project_id).await;
    push_to_fulltext_search(&result, &"hello world".to_string()).await;
    let resp = box_actix_rest_response(result);
    return resp;
}

pub async fn push_to_fulltext_search(tex_file: &TexFile, content: &String) {
    let url = get_app_config("texhub.meilisearch_url");
    // https://www.meilisearch.com/docs/reference/api/documents
    // let full_url = join_paths(&[url, "/indexes/files/documents".to_string()]);
    let api_key = env::var("MEILI_MASTER_KEY").expect("MEILI_MASTER_KEY must be set");
    let client = meilisearch_sdk::Client::new(url, Some(api_key));
    let movies = client.index("files");
    let new_doc = SearchFile::new_file(tex_file, content);
    let add_result = movies.add_documents(&[new_doc], Some("file_id")).await;
    match add_result {
        Ok(_) => {}
        Err(e) => {
            error!(
                "add fulltext search facing error,{}, text file: {:?}",
                e, tex_file
            );
        }
    }
    let set_result = movies
        .set_filterable_attributes(["name", "project_id"])
        .await;
    match set_result {
        Ok(_) => {}
        Err(se) => {
            error!(
                "set fulltext search filter error,{}, text file: {:?}",
                se, tex_file
            );
        }
    }
}

pub async fn create_file_on_disk(file: &TexFile) {
    let base_compile_dir: String = get_proj_base_dir(&file.project_id);
    if file.file_type == (ThFileType::Folder as i32) {
        let split_path = &[base_compile_dir, file.file_path.clone()];
        let file_full_path = join_paths(split_path);
        create_folder_not_exists(&file_full_path);
    } else {
        let split_path = &[base_compile_dir, file.file_path.clone(), file.name.clone()];
        let file_full_path = join_paths(split_path);
        let create_result = create_disk_file(&file_full_path);
        if let Err(e) = create_result {
            error!("create file on disk failed, {}", e);
        }
    }
}

fn create_disk_file(file_path: &str) -> std::io::Result<()> {
    if !std::path::Path::new(file_path).exists() {
        let mut file = File::create(file_path)?;
        file.write_all(b"")?;
    }
    Ok(())
}

pub fn get_file_path(add_req: &TexFileAddReq) -> String {
    match get_cached_file_by_fid(&add_req.parent) {
        None => {
            if add_req.file_type == 0 {
                format!("/{}", add_req.name)
            } else {
                "/".to_owned()
            }
        }
        Some(finfo) => {
            if add_req.file_type == 0 {
                join_paths(&[finfo.file_path, add_req.name.to_string()])
            } else {
                finfo.file_path.clone()
            }
        }
    }
}

pub fn file_init_complete(edit_req: &FileCodeParams) -> TexFile {
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let predicate =
        crate::model::diesel::tex::tex_schema::tex_file::file_id.eq(edit_req.file_id.clone());
    let update_result = diesel::update(tex_file.filter(predicate))
        .set(yjs_initial.eq(1))
        .get_result::<TexFile>(&mut get_connection())
        .expect("unable to update tex file");
    return update_result;
}

pub async fn rename_trans(
    edit_req: &TexFileRenameReq,
    login_user_info: &LoginUserInfo,
) -> Option<TexFile> {
    let edit_req_copy = edit_req.clone();
    let mut rename_connection = get_connection();
    let trans_result: Result<Option<TexFile>, Error> =
        rename_connection.transaction(|connection| {
            return rename_file_impl(&edit_req_copy, login_user_info, connection);
        });
    if let Err(e) = trans_result {
        error!(
            "rename file failed,{}, edit req: {:?}, login user: {:?}",
            e, edit_req_copy, login_user_info
        );
        return None;
    }
    let renamed_file = trans_result.unwrap();
    if renamed_file.is_some() {
        let proj_id = renamed_file.clone().unwrap().project_id;
        del_project_cache(&proj_id).await;
    }
    return renamed_file;
}

pub fn rename_file_impl(
    edit_req: &TexFileRenameReq,
    login_user_info: &LoginUserInfo,
    connection: &mut PgConnection,
) -> Result<Option<TexFile>, Error> {
    use crate::model::diesel::tex::tex_schema::tex_file as tex_file_table;
    use tex_file_table::dsl::*;
    let predicate = tex_file_table::file_id
        .eq(edit_req.file_id.clone())
        .and(tex_file_table::user_id.eq(login_user_info.userId));
    let update_msg = format!(
        "unable to update tex file name, user id: {}, file_id: {}",
        login_user_info.userId,
        edit_req.file_id.clone()
    );
    let fs = TexFileService {};
    let legacy_file = fs.get_file_by_id(&edit_req.file_id);
    if legacy_file.is_none() {
        error!("could not found file, {:?}", &edit_req);
        return Err(NotFound);
    }
    let update_result = diesel::update(tex_file.filter(predicate))
        .set((name.eq(edit_req.name.clone()),))
        .get_result::<TexFile>(connection)
        .expect(&update_msg);
    let proj_dir = get_proj_base_dir(&update_result.project_id);
    if update_result.file_type == ThFileType::Folder as i32 {
        let rename_result = handle_folder_rename(proj_dir, &legacy_file.unwrap(), &update_result);
        if let Err(err) = rename_result {
            error!("folder file err,{}", err);
            return Err(NotFound);
        }
    } else {
        let rename_result = handle_file_rename(proj_dir, &update_result, edit_req);
        if let Err(err) = rename_result {
            error!("rename file on disk facing err,{}", err);
            return Err(NotFound);
        }
    }
    let file_cached_key_prev: String = get_app_config("texhub.fileinfo_redis_key");
    let file_cached_key = format!("{}:{}", file_cached_key_prev, &edit_req.file_id.clone());
    let del_cache_result = del_redis_key(&file_cached_key);
    if let Err(e) = del_cache_result {
        error!("failed to delete file cache,{}", e);
    }
    return Ok(Some(update_result));
}

fn handle_folder_rename(
    proj_dir: String,
    legacy_file: &TexFile,
    new_file: &TexFile,
) -> Result<(), std::io::Error> {
    let legacy_path = join_paths(&[proj_dir.clone(), legacy_file.file_path.to_string()]);

    let new_path = join_paths(&[proj_dir, new_file.file_path.to_string()]);
    return fs::rename(legacy_path.clone(), new_path.clone());
}

fn handle_file_rename(
    proj_dir: String,
    update_result: &TexFile,
    edit_req: &TexFileRenameReq,
) -> Result<(), std::io::Error> {
    let legacy_path = join_paths(&[
        proj_dir.clone(),
        update_result.file_path.clone(),
        edit_req.legacy_name.clone(),
    ]);
    let new_path = join_paths(&[
        proj_dir,
        update_result.file_path.clone(),
        edit_req.name.clone(),
    ]);
    let rename_result = fs::rename(legacy_path.clone(), new_path.clone());
    if let Err(err) = rename_result.as_ref() {
        error!(
            "rename file on disk failed, err:{}, legacy path: {}, new path:{},update result: {:?}",
            err, legacy_path, new_path, update_result
        );
    }
    rename_result
}

fn move_directory(src_path: &str, dest_path: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(dest_path)?;
    for entry in fs::read_dir(src_path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let entry_path = entry.path();
        if file_type.is_dir() {
            let new_dest_path = format!("{}/{}", dest_path, entry.file_name().to_string_lossy());
            move_directory(&entry_path.to_string_lossy(), &new_dest_path)?;
        } else if file_type.is_file() {
            let new_dest_path = format!("{}/{}", dest_path, entry.file_name().to_string_lossy());
            fs::rename(&entry_path, &new_dest_path)?;
        }
    }
    fs::remove_dir_all(src_path)?;
    Ok(())
}

pub fn mv_file_impl(
    move_req: &MoveFileReq,
    login_user_info: &LoginUserInfo,
    src_file: &TexFile,
    dist_file: &TexFile,
) -> Result<Option<TexFile>, Error> {
    use crate::model::diesel::tex::tex_schema::tex_file as tex_file_table;
    use tex_file_table::dsl::*;
    let mut connection = get_connection();
    let trans_result: Result<Option<TexFile>, Error> = connection.transaction(|connection| {
        let proj_dir = get_proj_base_dir(&move_req.project_id);
        if src_file.file_type == ThFileType::Folder as i32 {
            let src_dir = join_paths(&[proj_dir.clone(), src_file.file_path.clone()]);
            let dist_dir = join_paths(&[proj_dir.clone(), dist_file.file_path.clone()]);
            let m_result = move_directory(&src_dir, &dist_dir);
            if let Err(err) = m_result {
                error!(
                    "move dir failed, {}, src dir: {}, dist dir: {}",
                    err, src_dir, dist_dir
                );
                return Ok(None);
            }
        } else {
            let src_path = join_paths(&[
                proj_dir.clone(),
                src_file.file_path.clone(),
                src_file.name.clone(),
            ]);
            // https://stackoverflow.com/questions/78251705/is-a-directory-os-error-21-when-using-rust-to-move-a-file
            let dist_path = join_paths(&[
                proj_dir.clone(),
                dist_file.file_path.clone(),
                "/".to_owned(),
                src_file.name.clone(),
            ]);
            let fm = fs::rename(&src_path, &dist_path);
            if let Err(err) = fm {
                error!(
                    "move file failed, {} ,src path: {}, dist path: {}",
                    err, src_path, dist_path
                );
                return Ok(None);
            }
        }
        let predicate = tex_file_table::file_id
            .eq(move_req.file_id.clone())
            .and(tex_file_table::user_id.eq(login_user_info.userId))
            .and(tex_file_table::project_id.eq(move_req.project_id.clone()));
        let new_relative_path = if src_file.file_type == ThFileType::Folder as i32 {
            join_paths(&[dist_file.file_path.clone(), dist_file.name.clone()])
        } else {
            dist_file.file_path.clone()
        };
        let update_result = diesel::update(tex_file.filter(predicate))
            .set((
                parent.eq(dist_file.file_id.clone()),
                file_path.eq(new_relative_path),
            ))
            .get_result::<TexFile>(connection)
            .expect("unable to move tex file");
        return Ok(Some(update_result));
    });
    return trans_result;
}

pub fn delete_file_recursive(
    del_req: &TexFileDelReq,
    tex_file: &TexFile,
    uid: i64,
) -> Result<usize, Error> {
    let mut connection = get_connection();
    let trans_result = connection.transaction(|connection| {
        let delete_result = del_single_file(&del_req.file_id, connection);
        match delete_result {
            Ok(proj) => {
                del_project_file(&del_req.file_id, &uid, connection);
                task::spawn_blocking({
                    let del_tex_file = tex_file.clone();
                    move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(del_disk_file(&del_tex_file));
                    }
                });
                return Ok(proj);
            }
            Err(e) => diesel::result::QueryResult::Err(e),
        }
    });
    return trans_result;
}

pub async fn del_disk_file(tex_file: &TexFile) {
    let proj_base_dir = get_proj_base_dir(&tex_file.project_id);
    if tex_file.file_type == (ThFileType::Folder as i32) {
        let folder_path = join_paths(&[proj_base_dir, tex_file.file_path.clone()]);
        let del_result = fs::remove_dir_all(&folder_path);
        if let Err(e) = del_result {
            error!("delete folder failed, {}, path: {}", e, folder_path);
        }
    } else {
        let proj_base_dir = get_proj_base_dir(&tex_file.project_id);
        let file_path = join_paths(&[
            proj_base_dir,
            tex_file.file_path.clone(),
            tex_file.name.clone(),
        ]);
        let del_result = fs::remove_file(&file_path);
        if let Err(e) = del_result {
            error!("delete file failed, e:{}, path: {}", e, file_path)
        }
    }
}

pub fn del_single_file(
    del_file_id: &String,
    connection: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_file::dsl::*;
    let predicate = crate::model::diesel::tex::tex_schema::tex_file::file_id.eq(del_file_id);
    let delete_result = diesel::delete(tex_file.filter(predicate)).execute(connection);
    return delete_result;
}

pub fn get_file_tree(parent_id: &String) -> Vec<FileTreeResp> {
    use crate::model::diesel::tex::tex_schema::tex_file as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::parent.eq(parent_id));
    let cvs = query.load::<TexFile>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return find_sub_menu_cte_impl(&result, parent_id);
        }
        Err(err) => {
            error!("get files failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn proj_folder_tree(parent_id: &String) -> Option<FolderTreeResp> {
    let sub_tree = find_folder_sub_menu_cte_impl(parent_id);
    // add the virtual root node
    let virtual_root_node = FolderTreeResp::new_virtual_root(parent_id, sub_tree);
    return Some(virtual_root_node);
}

pub fn find_folder_sub_menu_cte_impl(root_id: &String) -> Vec<FolderTreeResp> {
    let mut connection = get_connection();
    let cte_query_sub_menus = format!(
        " with recursive sub_files as (
            select 
              id, 
              name, 
              sort,
              file_id, 
              project_id,
              file_type,
              parent,
              file_path 
            from 
              tex_file mr 
            where 
              parent = '{}'
              and file_type = 0
            union all 
            select 
              origin.id, 
              origin.name,
              origin.sort,
              origin.file_id,           
              origin.project_id,
              origin.file_type,
              origin.parent,
              origin.file_path 
            from 
              sub_files
            join tex_file origin 
            on origin.parent = sub_files.file_id
            where origin.file_type = 0
          ) 
          select 
            id, 
            name, 
            sort,
            file_id, 
            project_id,
            file_type,
            parent,
            file_path 
          from 
            sub_files 
          order by 
            sort asc;      
    ",
        root_id
    );
    let cte_menus = sql_query(cte_query_sub_menus)
        .load::<TexFolderTree>(&mut connection)
        .expect("Error find file");
    let menu_resource_resp: Vec<FolderTreeResp> = map_entity(cte_menus);
    return convert_folder_to_tree_impl(&menu_resource_resp, root_id);
}

pub fn find_sub_menu_cte_impl(_root_menus: &Vec<TexFile>, root_id: &String) -> Vec<FileTreeResp> {
    let mut connection = get_connection();
    let cte_query_sub_menus = format!(
        " with recursive sub_files as (
        select 
          id, 
          name, 
          file_id, 
          sort,
          created_time,
          updated_time,
          user_id,
          doc_status,
          project_id,
          file_type,
          parent,
          main_flag,
          yjs_initial,
          file_path 
        from 
          tex_file mr 
        where 
          parent = '{}'
        union all 
        select 
          origin.id, 
          origin.name, 
          origin.file_id, 
          origin.sort,
          origin.created_time,
          origin.updated_time,
          origin.user_id,
          origin.doc_status,
          origin.project_id,
          origin.file_type,
          origin.parent,
          origin.main_flag,
          origin.yjs_initial,
          origin.file_path 
        from 
          sub_files 
          join tex_file origin on origin.parent = sub_files.file_id
      ) 
      select 
        id, 
        name, 
        file_id, 
        sort,
        created_time,
        updated_time,
        user_id,
        doc_status,
        project_id,
        file_type,
        parent,
        main_flag,
        yjs_initial,
        file_path 
      from 
        sub_files 
      order by 
        sort asc;      
    ",
        root_id
    );
    let cte_menus = sql_query(cte_query_sub_menus)
        .load::<TexFile>(&mut connection)
        .expect("Error find file");
    let menu_resource_resp: Vec<FileTreeResp> = map_entity(cte_menus);
    return convert_to_tree_impl(&menu_resource_resp, root_id);
}

fn convert_to_tree_impl(contents: &Vec<FileTreeResp>, root_id: &str) -> Vec<FileTreeResp> {
    let root_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent == root_id)
        .collect();
    let sub_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent != root_id)
        .collect();
    let result = convert_to_tree(&root_element, &sub_element);
    return result;
}

fn convert_folder_to_tree_impl(
    contents: &Vec<FolderTreeResp>,
    root_id: &str,
) -> Vec<FolderTreeResp> {
    let root_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent == root_id)
        .collect();
    let sub_element: Vec<_> = contents
        .iter()
        .filter(|content| content.parent != root_id)
        .collect();
    let result = convert_to_tree(&root_element, &sub_element);
    return result;
}

pub fn get_partial_pdf(lastest_pdf: &LatestCompile, range: Option<&HeaderValue>) -> HttpResponse {
    let proj_base_dir = get_proj_base_dir(&lastest_pdf.project_id);
    let pdf_name = format!(
        "{}{}",
        get_filename_without_ext(&lastest_pdf.file_name),
        ".pdf"
    );
    let pdf_file_path = join_paths(&[proj_base_dir, "app-compile-output".to_owned(), pdf_name]);
    if range.is_none() {
        let open_err = format!("Failed to open file, {}", &pdf_file_path);
        if !Path::new(&pdf_file_path).exists() {
            error!("file not exists,{}", &pdf_file_path);
            return HttpResponse::NotFound().body("no content");
        }
        let mut buf = Vec::new();
        let mut file = File::open(&pdf_file_path).expect(&open_err);
        let read_result = file.read_to_end(&mut buf);
        if let Err(err) = read_result {
            error!(
                "read file facing issue, path:{},err:{}",
                &pdf_file_path.clone(),
                err
            );
        }
        let metadata = file.metadata().expect("Failed to get metadata");
        let file_size = metadata.len();
        return HttpResponse::PartialContent()
            .insert_header(CacheControl(vec![CacheDirective::NoCache]))
            .append_header(("Accept-Ranges", "bytes"))
            .append_header(("Content-Length", file_size))
            .append_header((
                "Access-Control-Expose-Headers",
                "Accept-Ranges,Content-Range",
            ))
            .content_type("application/pdf")
            .body(buf);
    }
    let bytes_info: Vec<&str> = range.unwrap().to_str().unwrap().split("=").collect();
    let mut parts = bytes_info[1].split('-');
    let start = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
    let end = parts.next().unwrap_or("0").parse::<u64>().unwrap_or(0);
    let mut file = File::open(pdf_file_path).expect("Failed to open file");
    let metadata = file.metadata().expect("Failed to get metadata");
    let file_size = metadata.len();
    file.seek(SeekFrom::Start(start))
        .expect("Failed to seek file");
    let mut buf = vec![0; (end - start + 1) as usize];
    file.take(end - start + 1)
        .read_exact(&mut buf)
        .expect("Failed to read file");
    let content_range = format!("bytes {}-{}/{}", start, end, file_size);
    return HttpResponse::PartialContent()
        .insert_header(CacheControl(vec![CacheDirective::NoCache]))
        .append_header(("Content-Range", content_range))
        .append_header(("Accept-Ranges", "bytes"))
        .append_header(("Content-Length", file_size))
        .append_header((
            "Access-Control-Expose-Headers",
            "Accept-Ranges,Content-Range",
        ))
        .content_type("application/pdf")
        .body(buf);
}

pub fn get_full_pdf(lastest_pdf: &LatestCompile, req: HttpRequest) -> HttpResponse {
    let proj_base_dir = get_proj_base_dir(&lastest_pdf.project_id);
    let pdf_name = format!(
        "{}{}",
        get_filename_without_ext(&lastest_pdf.file_name),
        ".pdf"
    );
    let pdf_file_path = join_paths(&[proj_base_dir, "app-compile-output".to_owned(), pdf_name]);
    match NamedFile::open(&pdf_file_path.clone()) {
        Ok(file) => {
            let content_type: Mime = "application/pdf".parse().unwrap();
            return NamedFile::set_content_type(file, content_type)
                .set_content_disposition(ContentDisposition {
                    disposition: DispositionType::Inline,
                    parameters: vec![],
                })
                .into_response(&req);
        }
        Err(e) => {
            error!("Error open pdf file {},{}", pdf_file_path, e);
            return ErrorBadRequest("File not Found").into();
        }
    }
}

pub fn get_pdf_content_length(lastest_pdf: &LatestCompile) -> Option<Metadata> {
    let proj_base_dir = get_proj_base_dir(&lastest_pdf.project_id);
    let pdf_file_path = join_paths(&[proj_base_dir, lastest_pdf.file_name.clone()]);
    let file = File::open(pdf_file_path.clone());
    if let Err(err) = file {
        error!("open file failed: err:{}, path: {}", err, pdf_file_path);
        return None;
    }
    let file_metadata = file.unwrap().metadata().expect("Failed to get metadata");
    return Some(file_metadata);
}
