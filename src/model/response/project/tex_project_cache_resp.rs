#![allow(unused)]
#![allow(clippy::all)]

use crate::model::diesel::custom::project::tex_project_cache::TexProjectCache;
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
pub struct TexProjectCacheResp {
    pub main: TexProject,
    pub main_file: TexFileResp,
    pub tree: Vec<FileTreeResp>,
}

impl TexProjectCacheResp {
    pub(crate) fn from_db(main: &TexProject, main_file: TexFileResp, tree: Vec<FileTreeResp>) -> Self {
        Self {
            main: main.clone(),
            main_file,
            tree,
        }
    }
}

impl From<&TexProjectCache> for TexProjectCacheResp {
    fn from(cached_proj: &TexProjectCache) -> Self {
        Self {
            main: cached_proj.main.clone(),
            main_file: TexFileResp::from(&cached_proj.main_file),
            tree: cached_proj.tree.iter().map(|t| FileTreeResp::from(t)).collect(),
        }
    }
}