use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct TexFileAddReq {
    pub name: String,
    #[validate(length(min = 1))]
    pub parent: String,
    #[validate(length(min = 1))]
    pub project_id: String,
    pub file_type: i32,
}