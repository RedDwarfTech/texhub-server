#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::project::add::tex_folder_req::TexFolderReq;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_proj_folder)]
pub struct FolderAdd {
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub sort: i32,
    pub folder_name: String,
    pub proj_type: i32,
    pub default_folder: i32
}

impl FolderAdd {
    pub(crate) fn from_req(folder_req: &TexFolderReq, user_id: &i64) ->Self {
        Self {
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            sort: 0,
            folder_name: folder_req.folder_name.clone(),
            proj_type: folder_req.proj_type.clone(),
            default_folder: folder_req.default_folder,
        }
    }
}