use crate::common::proj::proj_util::get_proj_compile_req;
use crate::controller::project::project_controller::{EditPrjReq, GetPrjParams, ProjQueryParams};
use crate::diesel::RunQueryDsl;
use crate::model::diesel::custom::file::file_add::TexFileAdd;
use crate::model::diesel::custom::project::queue::compile_queue_add::CompileQueueAdd;
use crate::model::diesel::custom::project::tex_proj_editor_add::TexProjEditorAdd;
use crate::model::diesel::custom::project::tex_project_add::TexProjectAdd;
use crate::model::diesel::custom::project::tex_project_cache::TexProjectCache;
use crate::model::diesel::tex::custom_tex_models::{TexCompQueue, TexProjEditor, TexProject};
use crate::model::request::project::queue::queue_req::QueueReq;
use crate::model::request::project::queue::queue_status_req::QueueStatusReq;
use crate::model::request::project::tex_compile_project_req::TexCompileProjectReq;
use crate::model::request::project::tex_compile_queue_req::TexCompileQueueReq;
use crate::model::request::project::tex_compile_queue_status::TexCompileQueueStatus;
use crate::model::request::project::tex_join_project_req::TexJoinProjectReq;
use crate::model::response::project::tex_proj_resp::TexProjResp;
use crate::net::render_client::{construct_headers, render_request};
use crate::service::file::file_service::get_main_file_list;
use crate::service::project::project_queue_service::get_proj_queue_list;
use crate::{common::database::get_connection, model::diesel::tex::custom_tex_models::TexFile};
use actix_web::HttpResponse;
use diesel::result::Error;
use diesel::{
    sql_query, BoolExpressionMethods, Connection, ExpressionMethods, PgConnection,
    QueryDsl,
};
use futures_util::{StreamExt, TryStreamExt};
use log::{error, warn};
use reqwest::Client;
use rust_wheel::common::infra::user::rd_user::get_user_info;
use rust_wheel::common::util::model_convert::map_entity;
use rust_wheel::common::util::net::sse_message::SSEMessage;
use rust_wheel::common::util::rd_file_util::get_filename_without_ext;
use rust_wheel::common::util::rd_file_util::remove_dir_recursive;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::common::wrapper::actix_http_resp::{
    box_actix_rest_response, box_error_actix_rest_response,
};
use rust_wheel::config::app::app_conf_reader::get_app_config;
use rust_wheel::config::cache::redis_util::{get_str_default, push_to_stream, set_value};
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use rust_wheel::model::user::rd_user_info::RdUserInfo;
use rust_wheel::texhub::compile_status::CompileStatus;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::task;

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

pub fn get_prj_by_id(proj_id: &String) -> TexProject {
    use crate::model::diesel::tex::tex_schema::tex_project as cv_work_table;
    let mut query = cv_work_table::table.into_boxed::<diesel::pg::Pg>();
    query = query.filter(cv_work_table::project_id.eq(proj_id));
    let cvs = query.load::<TexProject>(&mut get_connection()).unwrap();
    return cvs[0].clone();
}

pub fn edit_proj(edit_req: &EditPrjReq) -> TexProject {
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
        Ok(_) => {
            create_main_file_on_disk(&proj.project_id);
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

fn create_main_file_on_disk(project_id: &String) {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let file_folder = format!("{}/{}", base_compile_dir, project_id);
    match create_directory_if_not_exists(&file_folder) {
        Ok(()) => {}
        Err(e) => error!("create directory failed,{}", e),
    }
    if let Ok(mut file) = File::create(format!("{}/{}", &file_folder, "main.tex")) {
        if let Err(we) = file.write_all(
            b"\\documentclass{article}\n\n\\begin{document}\nHello, World!\n\\end{document}\n",
        ) {
            error!("write content failed, {}", we);
        }
    } else {
        error!("create file failed");
    }
}

fn create_directory_if_not_exists(path: &str) -> io::Result<()> {
    if !fs::metadata(path).is_ok() {
        fs::create_dir_all(path)?;
    }
    Ok(())
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
                    del_project_disk_file(del_project_id);
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

pub fn del_project_disk_file(proj_id: &String) {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let proj_dir = format!("{}/{}", base_compile_dir, proj_id);
    let result = remove_dir_recursive(Path::new(&proj_dir));
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
    let update_result = diesel::update(tex_comp_queue.filter(predicate))
        .set(comp_status.eq(params.comp_status))
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
    let new_proj = CompileQueueAdd::from_req(&params.project_id, &login_user_info.userId);
    use crate::model::diesel::tex::tex_schema::tex_comp_queue::dsl::*;
    let queue_result = diesel::insert_into(tex_comp_queue)
        .values(&new_proj)
        .get_result::<TexCompQueue>(&mut connection);
    if let Err(e) = queue_result {
        error!("add compile queue failed, error info:{}", e);
        return box_error_actix_rest_response(
            "",
            "QUEUE_ADD_FAILED".to_string(),
            "queue add failed".to_string(),
        );
    }
    let proj_cache = get_cached_proj_info(&params.project_id).await;
    let main_file_name = proj_cache.unwrap().main_file.name;
    let stream_key = get_app_config("texhub.compile_stream_redis_key");
    let file_path = format!(
        "/opt/data/project/{}/{}",
        &params.project_id, &main_file_name
    );
    let out_path = format!("/opt/data/project/{}", &params.project_id);
    let rt = get_current_millisecond().to_string();
    let qid = queue_result.as_ref().unwrap().id.to_string();
    let s_params = [
        ("file_path", file_path.as_str()),
        ("out_path", out_path.as_str()),
        ("project_id", params.project_id.as_str()),
        ("req_time", rt.as_str()),
        ("qid", qid.as_str()),
        ("version_no", queue_result.as_ref().unwrap().version_no.as_str())
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

pub async fn get_compiled_log(main_file: TexFile) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let file_folder = format!("{}/{}", base_compile_dir, main_file.project_id);
    let file_name_without_ext = get_filename_without_ext(&main_file.name);
    let log_full_path = format!("{}/{}.log", file_folder, file_name_without_ext);
    let mut file = match File::open(log_full_path) {
        Ok(file) => file,
        Err(error) => {
            error!("Error opening project log file: {:?}", error);
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

pub async fn get_project_pdf(params: &GetPrjParams) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let prj_dir = format!("{}/{}", base_compile_dir, params.project_id);
    if !fs::metadata(&prj_dir).is_ok() {
        error!("folder did not exists, dir: {}", prj_dir);
        return "".to_owned();
    }
    let subdirectories = fs::read_dir(prj_dir)
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
    params: &TexCompileProjectReq,
    tx: UnboundedSender<SSEMessage<String>>,
) -> Result<String, reqwest::Error> {
    let file_name_without_ext = get_filename_without_ext(&params.file_name);
    let file_path = format!(
        "/opt/data/project/{}/{}.log",
        params.project_id, file_name_without_ext
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
        let tx: UnboundedSender<SSEMessage<String>> = tx.clone();
        move || {
            let shared_tx = Arc::new(Mutex::new(tx));
            for line in reader.lines() {
                if let Ok(line) = line {
                    let msg_content = format!("{}\n", line.to_owned());
                    warn!("{}", msg_content);
                    let sse_msg: SSEMessage<String> =
                        SSEMessage::from_data(msg_content.to_string(), &"TEX_COMP_LOG".to_string());
                    let send_result = shared_tx.lock().unwrap().send(sse_msg);
                    if let Err(se) = send_result {
                        error!("send xelatex render compile log error: {}", se);
                    }
                }
            }
            _do_msg_send(&"end".to_string(), shared_tx, "TEX_COMP_END");
        }
    });
    Ok("".to_owned())
}

pub fn _do_msg_send(
    line: &String,
    tx: Arc<std::sync::Mutex<UnboundedSender<SSEMessage<String>>>>,
    msg_type: &str,
) {
    let sse_msg: SSEMessage<String> =
        SSEMessage::from_data(line.to_string(), &msg_type.to_string());
    let send_result = tx.lock().unwrap().send(sse_msg);
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
    compile_params.insert("req_time".to_owned(), params.req_time.to_string());
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

pub async fn get_cached_queue_status(req: &QueueStatusReq) -> Option<TexCompQueue> {
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

pub async fn get_cached_proj_info(proj_id: &String) -> Option<TexProjectCache> {
    let cache_key = format!("{}:{}", "texhub-server:proj:info", proj_id);
    let proj_info_result = get_str_default(&cache_key.as_str());
    if let Err(e) = proj_info_result.as_ref() {
        error!("get cached project info failed,{}", e);
        return None;
    }
    let cached_proj_info = proj_info_result.unwrap();
    if cached_proj_info.is_none() {
        let proj = get_prj_by_id(proj_id);
        let file = get_main_file_list(proj_id);
        let proj_info = TexProjectCache::from_db(&proj, file.unwrap());
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
