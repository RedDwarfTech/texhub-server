#![allow(unused)]
#![allow(clippy::all)]
use std::ffi::OsString;
use actix_multipart::form::tempfile::TempFile;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::custom_tex_models::TexTemplate;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::file::add::file_add_req::TexFileAddReq;
use crate::model::request::file::add::file_add_ver_req::TexFileVerAddReq;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_file_version)]
pub struct TexFileVersionAdd {
    pub name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub project_id: String,
    pub file_id: String,
    pub content: String
}

impl TexFileVersionAdd {
    pub(crate) fn gen_tex_file(add_file: &TexFileVerAddReq, login_user_info: &LoginUserInfo) ->Self {
        Self {
            name: add_file.name.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: login_user_info.userId,
            project_id: add_file.project_id.clone(),
            file_id: add_file.file_id.clone(),
            content: add_file.content.clone()
        }
    }
}