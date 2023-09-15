use serde::{Serialize, Deserialize};

use crate::model::diesel::tex::custom_tex_models::TexFile;

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct WsFileDetail {
    pub file_path: String,
    pub project_id: String,
    pub name: String,
    pub project_created_time: i64
}


impl From<(&TexFile,i64)> for WsFileDetail {
    fn from(items:(&TexFile,i64)) -> Self {
        Self {
            file_path: items.0.file_path.clone(),
            project_id: items.0.project_id.clone(),
            name: items.0.name.clone(),
            project_created_time: items.1,
        }
    }
}