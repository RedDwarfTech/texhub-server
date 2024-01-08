use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFolderReq {
    #[validate(length(min = 1))]
    pub folder_name: String,
    pub proj_type: i32,
    #[serde(default = "default_folder")]
    pub default_folder: i32
}

fn default_folder() -> i32 {
    0
}