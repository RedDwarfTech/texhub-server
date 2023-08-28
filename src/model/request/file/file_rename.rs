use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFileRenameReq {
    pub file_id: String,
    #[validate(length(min = 1))]
    pub name: String,
}