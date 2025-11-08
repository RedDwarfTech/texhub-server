#![allow(unused)]
#![allow(clippy::all)]

use crate::model::dict::proj_source_type::ProjSourceType;
use crate::model::diesel::tex::tex_schema::*;
use crate::model::request::project::add::tex_project_req::TexProjectReq;
use rust_wheel::common::util::time_util::get_current_millisecond;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

#[derive(Insertable, Queryable, QueryableByName, Debug, Serialize, Deserialize, Default, Clone)]
#[diesel(table_name = tex_project)]
pub struct TexProjectAdd {
    pub proj_name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub proj_status: i32,
    pub template_id: i64,
    pub project_id: String,
    pub nickname: String,
    pub proj_source_type: i16,
    pub proj_source: String,
}

impl TexProjectAdd {
    pub(crate) fn from_req(prj_req: &TexProjectReq, user_id: &i64, nickname: &String) -> Self {
        let uuid = Uuid::new_v4();
        let uuid_string = uuid.to_string().replace("-", "");
        Self {
            proj_name: prj_req.name.to_string(),
            created_time: get_current_millisecond(),
            updated_time: get_current_millisecond(),
            user_id: user_id.to_owned(),
            proj_status: 1,
            template_id: 1,
            project_id: uuid_string,
            nickname: nickname.to_string(),
            proj_source_type: prj_req
                .proj_source_type
                .clone()
                .unwrap_or(ProjSourceType::Default as i16),
            proj_source: prj_req.proj_source.clone().unwrap_or("default".to_owned()),
        }
    }
}
