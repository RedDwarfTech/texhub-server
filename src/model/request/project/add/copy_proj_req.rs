use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CopyProjReq {
    pub project_id: String,
    pub folder_id: i64
}