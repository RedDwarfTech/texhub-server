// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]

use serde::Serialize;
use serde::Deserialize;
use crate::model::diesel::tex::tex_schema::*;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_project)]
pub struct TexProject {
    pub id: i64,
    pub doc_name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub doc_status: i32,
    pub template_id: i64,
    pub project_id: String,
}

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_template)]
pub struct TexTemplate {
    pub id: i64,
    pub name: String,
    pub remark: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub template_status: i32,
    pub template_id: i64,
    pub preview_url: Option<String>,
    pub template_code: String,
    pub online_status: i32,
    pub source: Option<String>,
    pub font_size: Option<String>,
    pub main_color: Option<String>,
    pub theme: Option<String>,
    pub language: String,
    pub intro: String,
}

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_file)]
pub struct TexFile {
    pub id: i64,
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