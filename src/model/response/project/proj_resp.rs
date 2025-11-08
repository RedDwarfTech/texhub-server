use crate::model::diesel::tex::custom_tex_models::TexProjFolder;
use serde::{Deserialize, Serialize};
use super::tex_proj_resp::TexProjResp;

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct ProjResp {
    pub folders: Vec<TexProjFolder>,
    pub projects: Vec<TexProjResp>,
}

impl ProjResp {
    pub(crate) fn from_req(folder: Vec<TexProjFolder>, proj: Vec<TexProjResp>) ->Self {
        Self {
            folders: folder,
            projects: proj,
        }
    }
}
