use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct SnippetQueryParams {
    pub title: Option<String>
}
