#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_project)]
pub struct TexProjectAdd {
    pub proj_name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub proj_status: i32,
    pub template_id: i64,
    pub project_id: String,
}

impl TexProjectAdd {
    pub(crate) fn from_req(prj_name: &String, user_id: &i64) ->Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        Self {
            proj_name: prj_name.to_string(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            proj_status: 1,
            template_id: 1,
            project_id: uuid_string
        }
    }
}