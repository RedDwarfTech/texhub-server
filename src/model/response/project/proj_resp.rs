use serde::{Deserialize, Serialize};
use crate::model::diesel::tex::custom_tex_models::TexProject;

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ProjResp {
    pub folders: Vec<String>,
    pub projects: Vec<TexProjResp>
}