use serde::{Deserialize, Serialize};
use crate::model::diesel::tex::custom_tex_models::TexProject;
use crate::common::utils::url_parse::json_as_string;

#[derive(Deserialize, Serialize, PartialEq, Eq, Debug, Default, Clone)]
#[allow(non_snake_case)]
pub struct TexProjResp {
    #[serde(serialize_with = "json_as_string")]
    pub id: i64,
    pub proj_name: String,
    #[serde(serialize_with = "json_as_string")]
    pub created_time: i64,
    #[serde(serialize_with = "json_as_string")]
    pub updated_time: i64,
    pub proj_status: i32,
    #[serde(serialize_with = "json_as_string")]
    pub template_id: i64,
    pub project_id: String,
    pub nickname: String,
    pub role_id: i32,
}

impl From<&TexProject> for TexProjResp {
    fn from(proj: &TexProject) -> Self {
        Self {
            id: proj.id,
            proj_name: proj.proj_name.clone(),
            created_time: proj.created_time,
            updated_time: proj.updated_time,
            proj_status: proj.proj_status,
            template_id: proj.template_id,
            project_id: proj.project_id.clone(),
            nickname: proj.nickname.clone(),
            role_id: 0,
        }
    }
}
