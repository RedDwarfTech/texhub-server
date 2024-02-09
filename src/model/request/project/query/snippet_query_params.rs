use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct SnippetQueryParams {
    pub snippet: Option<String>
}
