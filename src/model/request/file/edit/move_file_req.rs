use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct MoveFileReq {
    #[validate(length(min = 1, max = 64))]
    pub project_id: String,
    #[validate(length(min = 1, max = 64))]
    pub file_id: String,
    #[validate(length(min = 1, max = 64))]
    pub dist_file_id: String,
}