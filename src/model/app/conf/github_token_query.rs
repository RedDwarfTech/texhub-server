use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct GithubTokenQuery {
    pub key: String,
    pub user_id: i64,
}