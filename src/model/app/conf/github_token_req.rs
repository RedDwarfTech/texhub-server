use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct GithubTokenReq {
    pub token: String,
}