use rust_wheel::{common::util::{time_util::get_current_millisecond, rd_file_util::{join_paths, get_filename_without_ext}}, config::app::app_conf_reader::get_app_config, texhub::project::get_proj_path};
use serde_json::Value;

use service::project::project_service::get_cached_proj_info;

pub fn get_proj_compile_req(proj_id: &String, file_name: &String) -> Value {
    let file_path = format!(
        "/opt/data/project/{}/{}",
        proj_id, file_name
    );
    let out_path = format!("/opt/data/project/{}", &proj_id);
    let json_data = serde_json::json!({
        "file_path": file_path,
        "out_path": out_path,
        "req_time": get_current_millisecond(),
        "project_id": proj_id
    });
    return json_data;
}

pub fn get_proj_base_dir(proj_id: &String) -> String{
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let proj_info = get_cached_proj_info(&proj_id).unwrap();
    let ct = proj_info.main.created_time;
    let proj_base_dir = get_proj_path(&base_compile_dir, ct);
    let proj_dir = join_paths(&[proj_base_dir, proj_id.to_owned()]);
    return proj_dir;
}

// because we create the project in transaction
// when get the project from database, the transaction is not commmitted
// so we have to get the folder instantly, it only works with create the project
// this method only works for the invoke moment
// do not use it to get the history project work directory
pub fn get_proj_base_dir_instant(proj_id: &String) -> String {
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let ct = get_current_millisecond();
    let proj_base_dir = get_proj_path(&base_compile_dir, ct);
    let proj_dir = join_paths(&[proj_base_dir, proj_id.to_owned()]);
    return proj_dir;
}

pub async fn get_proj_log_name(proj_id: &String) -> String{
    let base_compile_dir: String = get_app_config("texhub.compile_base_dir");
    let proj_info = get_cached_proj_info(&proj_id).unwrap();
    let ct = proj_info.main.created_time;
    let proj_base_dir = get_proj_path(&base_compile_dir, ct);
    let log_name = format!("{}.log", get_filename_without_ext(&proj_info.main_file.name));
    let proj_dir = join_paths(&[proj_base_dir, proj_id.to_owned(), log_name]);
    return proj_dir;
}