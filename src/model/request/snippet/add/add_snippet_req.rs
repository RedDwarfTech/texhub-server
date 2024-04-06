use validator::Validate;

#[derive(serde::Deserialize, Validate, Clone)]
pub struct AddSnippetReq {
    #[validate(length(min = 1, max = 256))]
    pub title: String,
    #[validate(length(min = 1, max = 8192))]
    pub snippet: String
}