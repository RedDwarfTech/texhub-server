use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFolderReq {
    #[validate(length(min = 1))]
    pub folder_name: String,
    pub proj_type: i32
}