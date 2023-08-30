use crate::model::{
    diesel::tex::custom_tex_models::TexProject,
    request::project::tex_compile_project_req::TexCompileProjectReq,
};
use serde_json::Value;

pub fn get_proj_compile_req(params: &TexCompileProjectReq, proj: &TexProject) -> Value {
    let file_path = format!(
        "/opt/data/project/{}/{}",
        &params.project_id, params.file_name
    );
    let out_path = format!("/opt/data/project/{}", &params.project_id);
    let json_data = serde_json::json!({
        "file_path": file_path,
        "out_path": out_path,
        "req_time": params.req_time,
        "project_id": proj.project_id
    });
    return json_data;
}
