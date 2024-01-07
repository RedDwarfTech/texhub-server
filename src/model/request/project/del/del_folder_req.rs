use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct DelFolderReq {
    pub folder_id: i64
}