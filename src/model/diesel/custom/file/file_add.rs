#![allow(unused)]
#![allow(clippy::all)]

use std::ffi::OsString;

use crate::model::diesel::tex::custom_tex_models::TexTemplate;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::file::add::file_add_req::TexFileAddReq;
use crate::service::infra::infra_service::get_snowflake_id;
use crate::service::infra::infra_service::get_uniq_id;
use actix_multipart::form::tempfile::TempFile;
use futures::executor;
use log::warn;
use rust_wheel::common::util::time_util::get_current_millisecond;
use rust_wheel::model::user::login_user_info::LoginUserInfo;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Insertable, Queryable, QueryableByName, Debug, Serialize, Deserialize, Default, Clone)]
#[diesel(table_name = tex_file)]
pub struct TexFileAdd {
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
    pub sort: i32,
    pub yjs_initial: i16,
    pub file_path: String,
}

impl TexFileAdd {
    pub(crate) fn gen_tex_main(prj_id: &String, uid: &i64) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        let uniq_id = get_uniq_id();
        Self {
            id: uniq_id.unwrap(),
            name: "main.tex".to_owned(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: uid.to_owned(),
            doc_status: 1,
            project_id: prj_id.to_string(),
            file_type: 1,
            file_id: uuid_string,
            parent: prj_id.to_string(),
            main_flag: 1,
            file_path: "/".to_owned(),
            sort: 0,
            yjs_initial: 0,
        }
    }

    pub(crate) fn gen_tex_file(
        add_file: &TexFileAddReq,
        login_user_info: &LoginUserInfo,
        f_path: &String,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        let uniq_id = get_uniq_id();
        Self {
            name: add_file.name.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: login_user_info.userId,
            doc_status: 1,
            project_id: add_file.project_id.clone(),
            file_type: add_file.file_type,
            file_id: uuid_string,
            parent: add_file.parent.clone(),
            main_flag: 0,
            file_path: f_path.to_string(),
            sort: 0,
            yjs_initial: 0,
            id: uniq_id.unwrap(),
        }
    }

    pub(crate) fn gen_tex_file_from_disk(
        stored_path: String,
        uid: &i64,
        proj_id: &String,
        file_name: &OsString,
        main_name: &String,
        parent_id: &str,
        file_type: i32,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        let f_name = file_name.to_string_lossy().into_owned();
        let is_main_file =
            f_name == main_name.to_owned() && (stored_path == "/" || stored_path.is_empty());
            let uniq_id = get_uniq_id();
        Self {
            name: file_name.to_string_lossy().into_owned(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: uid.to_owned(),
            doc_status: 1,
            project_id: proj_id.to_string(),
            file_type: file_type,
            file_id: uuid_string,
            parent: parent_id.to_string(),
            main_flag: if is_main_file { 1 } else { 0 },
            yjs_initial: 0,
            file_path: if stored_path.is_empty() {
                "/".to_string()
            } else {
                stored_path
            },
            sort: 0,
            id: uniq_id.unwrap(),
        }
    }

    pub(crate) fn gen_tex_folder(
        add_file: &TexFileAddReq,
        login_user_info: &LoginUserInfo,
        f_path: &String,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        let uniq_id = get_uniq_id();
        Self {
            name: add_file.name.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: login_user_info.userId,
            doc_status: 1,
            project_id: add_file.project_id.clone(),
            file_type: add_file.file_type,
            file_id: uuid_string,
            parent: add_file.parent.clone(),
            main_flag: 0,
            file_path: f_path.to_string(),
            sort: 0,
            yjs_initial: 0,
            id: uniq_id.unwrap(),
        }
    }

    pub(crate) fn gen_upload_tex_file(
        file_name: &String,
        login_user_info: &LoginUserInfo,
        proj_id: &String,
        parent: &String,
        f_path: &String,
    ) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        let uniq_id = get_uniq_id();
        Self {
            name: file_name.clone(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: login_user_info.userId,
            doc_status: 1,
            project_id: proj_id.to_string(),
            file_type: 1,
            file_id: uuid_string,
            parent: parent.to_string(),
            main_flag: 0,
            file_path: f_path.to_string(),
            sort: 0,
            yjs_initial: 0,
            id: uniq_id.unwrap(),
        }
    }
}
