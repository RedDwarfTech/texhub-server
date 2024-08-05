use validator::Validate;

#[derive(serde::Deserialize, Validate, Clone, Debug)]
pub struct TexFileRenameReq {
    pub file_id: String,
    #[validate(length(min = 1))]
    pub name: String,
    #[validate(length(min = 1))]
    pub legacy_name: String,
}