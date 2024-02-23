#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_proj_editor)]
pub struct TexProjEditorAdd {
    pub role_id: i32,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub collar_status: i32,
    pub sort: i32,
    pub project_id: String,
    pub nickname: String,
}

impl TexProjEditorAdd {
    pub(crate) fn from_req(prj_id: &String, user_id: &i64, rid: i32, nickname: &String) ->Self {
        Self {
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            project_id: prj_id.to_string(),
            role_id: rid,
            collar_status: 1,
            sort: 0,
            nickname: nickname.to_owned()
        }
    }
}