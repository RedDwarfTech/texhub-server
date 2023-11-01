use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFileIdxReq {
    pub file_id: String,
    pub content: String
}