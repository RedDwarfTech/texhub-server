use crate::common::proj::proj_util::{get_proj_base_dir, get_proj_compile_req, get_proj_log_name, get_proj_base_dir_instant};
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::custom::project::queue::compile_queue_add::CompileQueueAdd;
use crate::model::diesel::custom::project::tex_proj_editor_add::TexProjEditorAdd;
use crate::model::diesel::custom::project::tex_project_add::TexProjectAdd;
use crate::model::diesel::custom::project::tex_project_cache::TexProjectCache;
use crate::model::diesel::custom::project::upload::proj_upload_file::ProjUploadFile;
use crate::model::diesel::tex::custom_tex_models::{
    TexCompQueue, TexProjEditor, TexProject, TexTemplate,
};
use crate::model::request::project::edit::edit_proj_req::EditProjReq;
use crate::model::request::project::query::proj_query_params::ProjQueryParams;
use crate::model::request::project::queue::queue_req::QueueReq;
use crate::model::request::project::queue::queue_status_req::QueueStatusReq;
use crate::model::request::project::tex_compile_project_req::TexCompileProjectReq;
use crate::model::request::project::tex_compile_queue_log::TexCompileQueueLog;
use crate::model::request::project::tex_compile_queue_req::TexCompileQueueReq;
use crate::model::request::project::tex_compile_queue_status::TexCompileQueueStatus;
use crate::model::request::project::tex_join_project_req::TexJoinProjectReq;
use crate::model::response::project::compile_resp::CompileResp;
use crate::model::response::project::latest_compile::LatestCompile;
use crate::model::response::project::tex_proj_resp::TexProjResp;
use crate::net::render_client::{construct_headers, render_request};
use crate::net::y_websocket_client::initial_file_request;
use crate::service::file::file_service::{get_file_by_fid, get_file_tree, get_main_file_list};
use crate::service::project::project_queue_service::get_proj_queue_list;
use crate::{common::database::get_connection, model::diesel::tex::custom_tex_models::TexFile};
use actix_web::HttpResponse;
use diesel::result::Error;
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection, QueryDsl,
};
use futures_util::{StreamExt, TryStreamExt};
use log::{error, warn};
use reqwest::Client;
use rust_wheel::common::infra::user::rd_user::get_user_info;
use rust_wheel::common::util::model_convert::map_entity;
use rust_wheel::common::util::net::sse_message::SSEMessage;
use rust_wheel::common::util::rd_file_util::{
    create_directory_if_not_exists, get_filename_without_ext, join_paths,
};
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::common::wrapper::actix_http_resp::{
    box_actix_rest_response, box_error_actix_rest_response,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::config::cache::redis_util::{
    del_redis_key, get_str_default, push_to_stream, set_value,
};
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use rust_wheel::model::user::rd_user_info::RdUserInfo;
use rust_wheel::texhub::compile_status::CompileStatus;
use rust_wheel::texhub::project::{get_proj_path, get_proj_relative_path};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;
use std::process::{ChildStdout, Command, Stdio};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;
use uuid::Uuid;

pub fn get_prj_list(_tag: &String, login_user_info: &LoginUserInfo) -> Vec<TexProject> {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::user_id.eq(login_user_info.userId));
    let cvs = query.load::<TexProject>(&mut get_connection());
    match cvs {
        Ok(result) => {
            return result;
        }
        Err(err) => {
            error!("get docs failed, {}", err);
            return Vec::new();
        }
    }
}

pub fn get_proj_by_type(
    query_params: &ProjQueryParams,
    login_user_info: &LoginUserInfo,
) -> Vec<TexProjResp> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as proj_editor_table;
    let mut query = proj_editor_table::table.into_boxed::<diesel::pg::Pg>();
    if query_params.role_id.is_some() {
        let rid = query_params.role_id.unwrap();
        query = query.filter(proj_editor_table::role_id.eq(rid));
    }
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

pub async fn create_empty_project(
    proj_name: &String,
    login_user_info: &LoginUserInfo,
) -> Result<TexProject, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let trans_result = connection
        .transaction(|connection| do_create_proj_trans(proj_name, &user_info, connection));
    return trans_result;
}

pub async fn create_tpl_project(
    tex_tpl: &TexTemplate,
    login_user_info: &LoginUserInfo,
) -> Result<Option<TexProject>, Error> {
    let user_info: RdUserInfo = get_user_info(&login_user_info.userId).await.unwrap();
    let mut connection = get_connection();
    let trans_result = connection
        .transaction(|connection| do_create_tpl_proj_trans(&tex_tpl, &user_info, connection));
    return trans_result;
}

fn do_create_proj_trans(
    proj_name: &String,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
) -> Result<TexProject, Error> {
    let create_result = create_proj(proj_name, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    let uid: i64 = rd_user_info.id.parse().unwrap();
    let result = create_main_file(&proj.project_id, connection, &uid);
    match result {
        Ok(file) => {
            let file_create_proj = proj.clone();
            task::spawn(async move {
                sync_file_to_yjs(&file_create_proj, &file.file_id).await;
            });
            let editor_result = create_proj_editor(&proj.project_id, rd_user_info, 1);
            match editor_result {
                Ok(_) => {}
                Err(error) => {
                    error!("create editor error: {}", error);
                }
            }
        }
        Err(e) => {
            error!("create file failed,{}", e)
        }
    }
    return Ok(proj);
}

fn do_create_tpl_proj_trans(
    tpl: &TexTemplate,
    rd_user_info: &RdUserInfo,
    connection: &mut PgConnection,
) -> Result<Option<TexProject>, Error> {
    let create_result = create_proj(&tpl.name, connection, rd_user_info);
    if let Err(ce) = create_result {
        error!("Failed to create proj: {}", ce);
        return Err(ce);
    }
    let proj = create_result.unwrap();
    do_create_proj_on_disk(&tpl, &proj, rd_user_info);
    return Ok(Some(proj));
}

pub fn do_create_proj_on_disk(tpl: &TexTemplate, proj: &TexProject, rd_user_info: &RdUserInfo) {
    let uid: i64 = rd_user_info.id.parse().unwrap();
    let create_res = create_proj_files(tpl, &proj.project_id, &uid);
    if !create_res {
        error!(
            "create project files failed,tpl: {:?}, project: {:?}",
            tpl, proj
        );
        return;
    }
    let editor_result = create_proj_editor(&proj.project_id, rd_user_info, 1);
    match editor_result {
        Ok(_) => {}
        Err(error) => {
            error!("create editor error: {}", error);
        }
    }
}

pub fn create_proj_files(tpl: &TexTemplate, proj_id: &String, uid: &i64) -> bool {
    let tpl_base_files_dir = get_app_config("texhub.tpl_files_base_dir");
    let tpl_files_dir = join_paths(&[tpl_base_files_dir, tpl.template_id.to_string()]);
    let proj_dir = get_proj_base_dir_instant(&proj_id);
    let result = copy_dir_recursive(&tpl_files_dir.as_str(), &proj_dir);
    if let Err(e) = result {
        error!(
            "copy file failed,{}, tpl dir: {}, project dir: {}",
            e, tpl_files_dir, proj_dir
        );
        return false;
    }
    return create_files_into_db(&proj_dir, proj_id, uid, tpl);
}

fn copy_dir_recursive(src: &str, dst: &str) -> io::Result<()> {
    if !fs::metadata(src)?.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} is not a directory", src),
        ));
    }

    if fs::metadata(dst).is_err() {
        fs::create_dir(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();

        if path.is_file() {
            let dst_file = format!("{}/{}", dst, file_name.to_str().unwrap());
            fs::copy(&path, &dst_file)?;
        } else if path.is_dir() {
            let dst_dir = format!("{}/{}", dst, file_name.to_str().unwrap());
            copy_dir_recursive(&path.to_str().unwrap(), &dst_dir)?;
        }
    }

    Ok(())
}

pub async fn init_project_into_yjs(files: &Vec<TexFileAdd>) {
    for file in files {
        let proj_base_dir = get_proj_base_dir_instant(&file.project_id);
        let file_full_path = join_paths(&[
            proj_base_dir,
            file.file_path.to_owned(),
            file.name.to_owned(),
        ]);
        if file.file_type == 1 && support_sync(&file_full_path) {
            let file_content = fs::read_to_string(&file_full_path);
            if let Err(e) = file_content {
                error!(
                    "Failed to read file,{}, file full path: {}",
                    e, file_full_path
                );
                return;
            }
            initial_file_request(&file.project_id, &file.file_id, &file_content.unwrap()).await;
        }
    }
}

pub fn support_sync(file_full_path: &String) -> bool {
    let path = Path::new(file_full_path);
    let extension = path.extension();
    if let Some(ext) = extension {
        match ext.to_str().unwrap() {
            "tex" => {
                return true;
            }
            "cls" => {
                return true;
            }
            "bib" => {
                return true;
            }
            "bbl" => {
                return true;
            }
            _ => {
                return false;
            }
        }
    } else {
        return false;
    }
}

pub fn create_files_into_db(
    project_path: &String,
    proj_id: &String,
    uid: &i64,
    tpl: &TexTemplate,
) -> bool {
    let mut files: Vec<TexFileAdd> = Vec::new();
    let read_result = read_directory(project_path, proj_id, &mut files, uid, proj_id, tpl);
    if let Err(err) = read_result {
        error!("read directory failed,{}", err);
        return false;
    }
    use crate::model::diesel::tex::tex_schema::tex_file as files_table;
    if files.len() == 0 {
        error!(
            "read 0 files from disk, project path: {}, template: {:?}",
            project_path, tpl
        );
        return false;
    }
    let result = diesel::insert_into(files_table::dsl::tex_file)
        .values(&files)
        .get_result::<TexFile>(&mut get_connection());
    if let Err(err) = result {
        error!("write files into db facing issue,{}", err);
        return false;
    }
    task::spawn_blocking({
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(init_project_into_yjs(&files));
        }
    });
    return true;
}

fn read_directory(
    dir_path: &str,
    parent: &str,
    files: &mut Vec<TexFileAdd>,
    uid: &i64,
    proj_id: &String,
    tpl: &TexTemplate,
) -> io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let relative_path = path.parent().unwrap().strip_prefix(dir_path);
        let stored_path = relative_path.unwrap().to_string_lossy().into_owned();

        if path.is_file() {
            let uuid = Uuid::new_v4();
            let uuid_string = uuid.to_string().replace("-", "");
            let tex_file = TexFileAdd {
                name: file_name.to_string_lossy().into_owned(),
                created_time: get_current_millisecond(),
                updated_time: get_current_millisecond(),
                user_id: uid.to_owned(),
                doc_status: 1,
                project_id: proj_id.to_string(),
                file_type: 1,
                file_id: uuid_string,
                parent: parent.to_string(),
                main_flag: if file_name.to_string_lossy().into_owned() == tpl.main_file_name {
                    1
                } else {
                    0
                },
                yjs_initial: 0,
                file_path: if stored_path.is_empty() {
                    "/".to_string()
                } else {
                    stored_path
                },
                sort: 0,
            };
            files.push(tex_file)
        } else if path.is_dir() {
            let dir_name = file_name.to_string_lossy().into_owned();
            let next_parent = format!("{}/{}", parent, dir_name);
            read_directory(
                path.to_str().unwrap(),
                &next_parent,
                files,
                uid,
                proj_id,
                tpl,
            )?;
        }
    }

    Ok(())
}

fn create_proj_editor(
    proj_id: &String,
    rd_user_info: &RdUserInfo,
    rid: i32,
) -> Result<TexProjEditor, diesel::result::Error> {
    use crate::model::diesel::tex::tex_schema::tex_proj_editor as proj_editor_table;
    let uid: i64 = rd_user_info.id.parse().unwrap();
    let proj_editor = TexProjEditorAdd::from_req(proj_id, &uid, rid);
    let result = diesel::insert_into(proj_editor_table::dsl::tex_proj_editor)
        .values(&proj_editor)
        .get_result::<TexProjEditor>(&mut get_connection());
    return result;
}

async fn sync_file_to_yjs(proj: &TexProject, file_id: &String) {
    let file_folder = get_proj_base_dir(&proj.project_id);
    match create_directory_if_not_exists(&file_folder) {
        Ok(()) => {}
        Err(e) => error!("create directory failed,{}", e),
    }
    let default_tex = get_app_config("texhub.default_tex_document");
    initial_file_request(&proj.project_id, file_id, &default_tex).await;
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
) -> Vec<TexFile> {
    let proj_id = proj_upload.project_id.clone();
    let parent = proj_upload.parent.clone();
    let mut files: Vec<TexFile> = Vec::new();
    for tmp_file in proj_upload.files {
        let db_file = get_file_by_fid(&proj_upload.parent).unwrap();
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
            return files;
        }
        let copy_result = fs::copy(&temp_path, &file_path.as_str());
        if let Err(e) = copy_result {
            error!("copy file failed, {}", e);
            return files;
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
            return files;
        }
        del_project_cache(&proj_id).await;
        files.push(create_result.unwrap());
    }
    return files;
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

fn create_proj(
    name: &String,
    connection: &mut PgConnection,
    rd_user_info: &RdUserInfo,
) -> Result<TexProject, diesel::result::Error> {
    let uid: i64 = rd_user_info.id.parse().unwrap();
    let new_proj = TexProjectAdd::from_req(name, &uid, &rd_user_info.nickname);
    use crate::model::diesel::tex::tex_schema::tex_project::dsl::*;
    let result = diesel::insert_into(tex_project)
        .values(&new_proj)
        .get_result::<TexProject>(connection);
    return result;
}

pub fn join_project(
    req: &TexJoinProjectReq,
    login_user_info: &LoginUserInfo,
) -> Result<TexProjEditor, Error> {
    let new_proj_editor = TexProjEditorAdd::from_req(&req.project_id, &login_user_info.userId, 2);
    use crate::model::diesel::tex::tex_schema::tex_proj_editor::dsl::*;
    let result = diesel::insert_into(tex_proj_editor)
        .values(&new_proj_editor)
        .get_result::<TexProjEditor>(&mut get_connection());
    return result;
}

pub async fn del_project_cache(del_project_id: &String) {
    let cache_key = format!(
        "{}:{}",
        get_app_config("texhub.proj_cache_key"),
        del_project_id
    );
    let del_result = del_redis_key(cache_key.as_str()).await;
    if let Err(e) = del_result {
        error!("delete project cache failed,{},cached key:{}", e, cache_key);
    }
}

pub fn del_project(del_project_id: &String, login_user_info: &LoginUserInfo) {
    let mut connection = get_connection();
    let result = connection.transaction(|connection| {
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
                    del_project_file(del_project_id, connection);
                    let async_proj_id = del_project_id.clone();
                    task::spawn_blocking({
                        move || {
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(del_project_disk_file(&async_proj_id));
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

pub async fn del_project_disk_file(proj_id: &String) {
    if proj_id.is_empty() {
        error!("delete project id is null");
        return;
    }
    let proj_dir = get_proj_base_dir(proj_id);
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

pub fn del_project_file(del_project_id: &String, connection: &mut PgConnection) {
    let del_command = format!(
        "WITH RECURSIVE x AS (
            SELECT file_id
            FROM   tex_file
            WHERE parent = '{}'
        
            UNION  ALL
            SELECT a.file_id
            FROM   x
            JOIN   tex_file a ON a.parent = x.file_id
            )
         DELETE FROM tex_file a
         USING  x
         WHERE a.file_id = x.file_id",
        del_project_id
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
    warn!("get the update params: {:?}", params);
    let update_result = diesel::update(tex_comp_queue.filter(predicate))
        .set((
            comp_status.eq(params.comp_status),
            comp_result.eq(params.comp_result),
            complete_time.eq(completed),
        ))
        .get_result::<TexCompQueue>(&mut get_connection());
    if let Err(e) = update_result {
        error!("update compile queue failed, error info:{}", e);
        return box_error_actix_rest_response(
            "",
            "UPDATE_QUEUE_FAILED".to_owned(),
            "update queue failed".to_owned(),
        );
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
        comp_status: vec![CompileStatus::Complete as i32, CompileStatus::Queued as i32],
    };
    let queue_list = get_proj_queue_list(&queue_req, login_user_info);
    if !queue_list.is_empty() {
        // return box_error_actix_rest_response("", "QUEUE_BUSY".to_string(),"queue busy".to_string());
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
    let main_file_name = proj_cache.clone().unwrap().main_file.name;
    let log_file_name = format!("{}{}", get_filename_without_ext(&main_file_name), ".log");
    let compile_base_dir = get_app_config("texhub.compile_base_dir");
    let proj_base_dir = get_proj_path(
        &compile_base_dir,
        proj_cache.clone().unwrap().main.created_time,
    );
    let stream_key = get_app_config("texhub.compile_stream_redis_key");
    let file_path = join_paths(&[
        proj_base_dir.clone(),
        params.project_id.clone(),
        main_file_name.clone(),
    ]);
    let out_path = join_paths(&[proj_base_dir, params.project_id.clone()]);
    let rt = get_current_millisecond().to_string();
    let qid = queue_result.as_ref().unwrap().id.to_string();
    let proj_created_time = proj_cache.clone().unwrap().main.created_time;
    let created_time_str = proj_created_time.to_string();
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
    let log_full_path = get_proj_log_name(&req.project_id, &req.version_no).await;
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

pub async fn get_proj_latest_pdf(proj_id: &String) -> LatestCompile {
    let version_no = get_project_pdf(proj_id).await;
    let proj_info = get_cached_proj_info(proj_id).unwrap();
    let ct = proj_info.main.created_time;
    let main_file = proj_info.main_file;
    let pdf_name = format!("{}{}", get_filename_without_ext(&main_file.name), ".pdf");
    let relative_path = get_proj_relative_path(proj_id, ct, &version_no);
    let pdf_result: LatestCompile = LatestCompile {
        path: join_paths(&[relative_path, pdf_name.to_string()]),
        project_id: proj_id.clone(),
    };
    return pdf_result;
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

pub async fn get_comp_log_stream(
    params: &TexCompileQueueLog,
    tx: UnboundedSender<SSEMessage<String>>,
) -> Result<String, reqwest::Error> {
    let file_name_without_ext = get_filename_without_ext(&params.file_name);
    let base_compile_dir: String = get_proj_base_dir(&params.project_id);
    let file_path = format!(
        "{}/{}/{}.log",
        base_compile_dir, params.version_no, file_name_without_ext
    );
    let mut cmd = Command::new("tail")
        .arg("-n")
        .arg("+1")
        .arg("-f")
        .arg(file_path)
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let log_stdout = cmd.stdout.take().unwrap();
    let reader = std::io::BufReader::new(log_stdout);
    task::spawn_blocking({
        let queue_log_params = params.clone();
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(comp_log_file_read(reader, &tx, &queue_log_params));
        }
    });
    Ok("".to_owned())
}

pub async fn comp_log_file_read(
    reader: BufReader<ChildStdout>,
    tx: &UnboundedSender<SSEMessage<String>>,
    params: &TexCompileQueueLog,
) {
    for line in reader.lines() {
        if let Ok(line) = line {
            let msg_content = format!("{}\n", line.to_owned());
            if msg_content.contains("====END====") {
                let cr = get_proj_latest_pdf(&params.project_id).await;
                let queue = get_cached_queue_status(params.qid).await;
                let comp_resp = CompileResp::from((cr, queue.unwrap()));
                let end_json = serde_json::to_string(&comp_resp).unwrap();
                do_msg_send_sync(&end_json, &tx, &"TEX_COMP_END".to_string());
                break;
            } else {
                do_msg_send_sync(&msg_content.to_string(), &tx, &"TEX_COMP_LOG".to_string());
            }
        }
    }
    drop(tx);
}

pub fn do_msg_send_sync(line: &String, tx: &UnboundedSender<SSEMessage<String>>, msg_type: &str) {
    let sse_msg: SSEMessage<String> =
        SSEMessage::from_data(line.to_string(), &msg_type.to_string());
    let send_result = tx.send(sse_msg);
    match send_result {
        Ok(_) => {}
        Err(e) => {
            error!("send xelatex compile log error: {}", e);
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
            error!("parse json failed,{},text:{}", parse_err, string_content);
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
