use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CopyProjReq {
    pub project_id: String
}