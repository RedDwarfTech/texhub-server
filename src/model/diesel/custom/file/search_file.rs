use crate::model::diesel::tex::custom_tex_models::TexFile;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SearchFile {
    pub name: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub content: String,
    pub file_id: String,
}

impl SearchFile {
    pub(crate) fn new_file(file: &TexFile, content: &String) -> Self {
        Self {
            name: file.name.clone(),
            created_time: file.created_time,
            updated_time: file.updated_time,
            content: content.to_owned(),
            file_id: file.file_id.clone(),
        }
    }
}
