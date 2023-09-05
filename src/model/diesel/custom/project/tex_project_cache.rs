#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::model::diesel::tex::custom_tex_models::TexProject;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Debug,Serialize,Deserialize,Default,Clone)]
pub struct TexProjectCache {
    pub main: TexProject,
    pub main_file: TexFile
}

impl TexProjectCache {
    pub(crate) fn from_db(main: &TexProject, main_file: TexFile) ->Self {
        Self {
            main: main.clone(),
            main_file,
        }
    }
}