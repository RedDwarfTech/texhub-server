#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::project::add::tex_folder_req::TexFolderReq;
use crate::model::request::project::edit::edit_proj_folder::EditProjFolder;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_proj_folder_map)]
pub struct FolderMapAdd {
    pub folder_id: i64,
    pub created_time: i64,
    pub updated_time: i64,
    pub project_id: String,
    pub user_id: i64,
    pub proj_type: i32,
}

impl FolderMapAdd {
    pub(crate) fn from_req(folder_req: &EditProjFolder, user_id: &i64) ->Self {
        Self {
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            folder_id: 0,
            project_id: folder_req.project_id.clone(),
            proj_type: folder_req.proj_type.clone(),
        }
    }
}