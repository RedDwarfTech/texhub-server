#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_comp_queue)]
pub struct CompileQueueAdd {
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub comp_status: i32,
    pub project_id: String,
}

impl CompileQueueAdd {
    pub(crate) fn from_req(proj_id: &String, user_id: &i64) ->Self {
        Self {
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            project_id: proj_id.to_string(),
            comp_status: 0,
        }
    }
}