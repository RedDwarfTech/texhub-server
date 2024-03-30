use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct MoveFileReq {
    pub project_id: String,
    pub file_id: String,
    pub dist_file_id: String,
}