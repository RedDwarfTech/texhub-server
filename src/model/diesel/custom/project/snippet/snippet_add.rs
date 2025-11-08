#![allow(unused)]
#![allow(clippy::all)]

use std::ffi::OsString;

use actix_multipart::form::tempfile::TempFile;
use log::warn;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use serde::Serialize;
use serde::Deserialize;
use uuid::Uuid;
use crate::model::diesel::tex::custom_tex_models::TexTemplate;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::file::add;
use crate::model::request::file::add::file_add_req::TexFileAddReq;
use crate::model::request::snippet::add::add_snippet_req::AddSnippetReq;

#[derive(Insertable,Queryable,QueryableByName,Debug,Serialize,Deserialize,Default,Clone)]
#[diesel(table_name = tex_snippet)]
pub struct SnippetAdd {
    pub title: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub sort: i32,
    pub snippet: String,
}

impl SnippetAdd {
    pub(crate) fn gen_snippet(add_file: &AddSnippetReq, login_user_info: &LoginUserInfo) ->Self {
        Self {
            title: add_file.title.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: login_user_info.userId,
            sort: 0,
            snippet: add_file.snippet.clone(),
        }
    }
}