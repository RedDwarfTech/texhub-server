use validator::Validate;

#[derive(serde::Deserialize, Validate, Clone)]
pub struct SnippetReq {
    pub id: i64,
    #[validate(length(min = 1))]
    pub name: String
}