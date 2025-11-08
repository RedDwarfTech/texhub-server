#![allow(unused)]
#![allow(clippy::all)]
use crate::model::diesel::tex::custom_tex_models::TexTemplate;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::file::add::file_add_req::TexFileAddReq;
use crate::model::request::file::add::file_add_ver_req::TexFileVerAddReq;
use actix_multipart::form::tempfile::TempFile;
use diesel::sql_types::Bytea;
use openssl::sha::Sha256;
use ring::digest;
use rust_wheel::common::util::security_util::get_sha;
use rust_wheel::common::util::security_util::get_str_sha;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use serde::Deserialize;
use serde::Serialize;
use std::ffi::OsString;
use uuid::Uuid;

#[derive(Insertable, Queryable, QueryableByName, Debug, Serialize, Deserialize, Default, Clone)]
#[diesel(table_name = tex_user_config)]
pub struct TexUserConfigAdd {
    pub config_key: String,
    pub remark: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub config_value: String,
    pub user_id: i64,
}

impl TexUserConfigAdd {
    pub(crate) fn gen_user_config(uid: &i64, config_value: &String, config_key: &String) -> Self {
        Self {
            config_key: config_key.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: uid.to_owned(),
            remark: "".to_owned(),
            config_value: config_value.to_owned(),
        }
    }
}
