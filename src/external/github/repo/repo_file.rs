use log::error;
use octocrab::{models::Repository, Octocrab};

use crate::common::utils::url_parse::parse_github_url;

pub async fn repo_file_exists(url: &str, github_token: &str, file_name: &str) -> bool {
   let parsed = parse_github_url(url);
    let octocrab = Octocrab::builder()
        .personal_token(github_token)
        .build()
        .unwrap();
    let owner = parsed[0];
    let repo = parsed[1];
    let file_path = "/";
    let r = octocrab
        .repos(owner, repo)
        .get_content()
        .path(file_path)
        .r#ref(file_name)
        .send()
        .await;
    match r {
        Ok(_) => {
            return true;
        }
        Err(_e) => {
            return false;
        }
    }
}

pub async fn get_github_repo_size(url: &str, github_token: &str) -> Option<Repository> {
    let trimmed = &url["https://github.com/".len()..];
    let trimmed = trimmed.trim_end_matches(".git");
    let parts: Vec<&str> = trimmed.split('/').collect();
    if parts.len() == 2 {
        let octocrab = Octocrab::builder()
            .personal_token(github_token)
            .build()
            .unwrap();
        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        let repo_info: Result<Repository, octocrab::Error> =
            octocrab.repos(owner.clone(), repo.clone()).get().await;
        if let Err(err) = repo_info.as_ref() {
            error!(
                " get repo info failed: {:?},owner:{},repo:{}",
                err, owner, repo
            );
            return None;
        }
        return Some(repo_info.unwrap());
    } else {
        error!(" get repo parts failed: {}", url);
        return None;
    }
}
