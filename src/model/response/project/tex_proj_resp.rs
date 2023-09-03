use serde::{Deserialize, Serialize};

use crate::model::diesel::tex::custom_tex_models::TexProject;

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct TexProjResp {
    pub id: i64,
    pub proj_name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub user_id: i64,
    pub proj_status: i32,
    pub template_id: i64,
    pub project_id: String,
    pub role_id: i32,
}

impl From<&TexProject> for TexProjResp {
    fn from(proj: &TexProject) -> Self {
        Self {
            id: proj.id,
            proj_name: proj.proj_name.clone(),
            created_time: proj.created_time,
            updated_time: proj.updated_time,
            user_id: proj.user_id,
            proj_status: proj.proj_status,
            template_id: proj.template_id,
            project_id: proj.project_id.clone(),
            role_id: 0
        }
    }
}
