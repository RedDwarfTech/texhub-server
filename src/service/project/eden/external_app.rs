use git2::Repository;
use std::fs;

use crate::{
    model::diesel::custom::file::file_add::TexFileAdd,
    net::y_websocket_client::initial_file_request,
    service::{
        global::proj::proj_util::get_proj_base_dir_instant, project::project_service::support_sync,
    },
};
use log::error;
use rust_wheel::{
    common::util::rd_file_util::join_paths, model::user::login_user_info::LoginUserInfo,
};

pub async fn init_project_into_yjs(files: &Vec<TexFileAdd>, login_user_info: &LoginUserInfo) {
    for file in files {
        let proj_base_dir = get_proj_base_dir_instant(&file.project_id);
        let file_full_path = join_paths(&[
            proj_base_dir,
            file.file_path.to_owned(),
            file.name.to_owned(),
        ]);
        if file.file_type == 1 && support_sync(&file_full_path) {
            let file_content = fs::read_to_string(&file_full_path);
            if let Err(e) = file_content {
                error!(
                    "Failed to read file when initial yjs,{}, file full path: {}",
                    e, file_full_path
                );
                return;
            }
            initial_file_request(
                &file.project_id,
                &file.file_id,
                &file_content.unwrap(),
                login_user_info,
            )
            .await;
        }
    }
}

pub fn clone_github_repo(url: &str) {
    let _repo = match Repository::clone(url, "/temp/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
}
