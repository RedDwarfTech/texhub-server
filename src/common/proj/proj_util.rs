use rust_wheel::common::util::time_util::get_current_millisecond;
use serde_json::Value;

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
