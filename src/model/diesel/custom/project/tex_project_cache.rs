#![allow(unused)]
#![allow(clippy::all)]

use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::diesel::tex::custom_tex_models::TexProject;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::response::file::file_tree_resp::FileTreeResp;
use crate::model::response::file::tex_file_resp::TexFileResp;
use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TexProjectCache {
    pub main: TexProject,
    pub main_file: TexFileResp,
    pub tree: Vec<FileTreeResp>,
}

impl TexProjectCache {
    pub(crate) fn from_db(main: &TexProject, main_file: TexFileResp, tree: Vec<FileTreeResp>) -> Self {
        Self {
            main: main.clone(),
            main_file,
            tree,
        }
    }
}
