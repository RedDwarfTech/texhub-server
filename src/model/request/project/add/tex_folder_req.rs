use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFolderReq {
    pub folder_name: String,
    pub proj_type: i32
}