use serde::{Serialize, Deserialize};

use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::common::utils::url_parse::json_as_string;

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct WsFileDetail {
    #[serde(serialize_with = "json_as_string")]
    pub id: i64,
    pub file_path: String,
    pub project_id: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub file_id: String,
    pub name: String,
    pub project_created_time: i64
}

impl From<(&TexFile,i64)> for WsFileDetail {
    fn from(items:(&TexFile,i64)) -> Self {
        Self {
            id: items.0.id,
            file_path: items.0.file_path.clone(),
            project_id: items.0.project_id.clone(),
            name: items.0.name.clone(),
            project_created_time: items.1,
            created_time: items.0.created_time,
            updated_time: items.0.updated_time,
            file_id: items.0.file_id.clone(),
        }
    }
}