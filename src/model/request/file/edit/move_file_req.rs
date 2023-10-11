use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct MoveFileReq {
    pub project_id: String,
    pub file_id: String,
    pub parent_id: String,
    pub file_type: i32,
    pub src_path: String,
    pub dist_path: String,
    pub file_name: String,
}