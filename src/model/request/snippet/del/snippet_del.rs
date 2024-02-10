use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct SnippetDel {
    pub id: i64
}