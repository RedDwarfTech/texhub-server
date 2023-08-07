#![allow(unused)]
#![allow(clippy::all)]

use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_file)]
pub struct TexFileAdd {
    pub name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub doc_status: i32,
    pub project_id: String,
    pub file_type: i32,
    pub file_id: String,
    pub parent: String,
    pub main_flag: i16,
}

impl TexFileAdd {
    pub(crate) fn gen_tex_main(prj_id: &String) ->Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        Self {
            name: "main.tex".to_owned(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: 1,
            doc_status:1,
            project_id: prj_id.to_string(),
            file_type: 1,
            file_id: uuid_string,
            parent: prj_id.to_string(),
            main_flag: 1,
        }
    }
}