use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFileVerAddReq {
    pub name: String,
    #[validate(length(min = 1))]
    pub file_id: String,
    #[validate(length(min = 1))]
    pub project_id: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub snapshot: Vec<u8>
}