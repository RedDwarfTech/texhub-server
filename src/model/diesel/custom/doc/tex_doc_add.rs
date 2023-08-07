#![allow(unused)]
#![allow(clippy::all)]

use serde::Serialize;
use serde::Deserialize;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_project)]
pub struct TexProjectAdd {
    pub doc_name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub doc_status: i32,
    pub template_id: i64,
    pub project_id: String,
}