use serde::{Deserialize, Serialize};

use crate::model::diesel::tex::custom_tex_models::TexFile;
use crate::common::utils::url_parse::json_as_string;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct TexFileResp {
    #[serde(serialize_with = "json_as_string")]
    pub id: i64,
    pub name: String,
    #[serde(serialize_with = "json_as_string")]
    pub created_time: i64,
    #[serde(serialize_with = "json_as_string")]
    pub updated_time: i64,
    #[serde(serialize_with = "json_as_string")]
    pub user_id: i64,
    pub doc_status: i32,
    pub project_id: String,
    pub file_type: i32,
    pub file_id: String,
    pub parent: String,
    pub main_flag: i16,
    pub sort: i32,
    pub yjs_initial: i16,
    pub file_path: String
}

impl Default for TexFileResp {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".to_owned(),
            created_time: 0,
            updated_time: 0,
            user_id: 0,
            doc_status: 1,
            project_id: "".to_owned(),
            file_type: 1,
            file_id: "".to_owned(),
            parent: "".to_owned(),
            file_path: "".to_owned(),
            main_flag: 0,
            yjs_initial: 0,
            sort: 0,
        }
    }
}

impl From<&TexFile> for TexFileResp {
    fn from(p: &TexFile) -> Self {
        Self {
            id: p.id,
            name: p.name.clone(),
            created_time: p.created_time,
            updated_time: p.updated_time,
            user_id: p.user_id,
            doc_status: p.doc_status,
            project_id: p.project_id.clone(),
            file_type: p.file_type,
            file_path: p.file_path.clone(),
            file_id: p.file_id.clone(),
            parent: p.parent.clone(),
            main_flag: p.main_flag,
            yjs_initial: p.yjs_initial,
            sort: p.sort
        }
    }
}