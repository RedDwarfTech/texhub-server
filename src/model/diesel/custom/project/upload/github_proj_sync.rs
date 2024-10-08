use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GithubProjSync {
    pub url: String,
    pub main_file: Option<String>,
}