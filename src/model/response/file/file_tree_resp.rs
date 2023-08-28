use rust_wheel::common::util::convert_to_tree_generic::IntoTree;
use serde::{Serialize, Deserialize};

use crate::model::diesel::tex::custom_tex_models::TexFile;

#[derive(Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct FileTreeResp {
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
    pub yjs_initial: i16,
    pub children: Vec<FileTreeResp>
}

impl Default for FileTreeResp {
    fn default() -> Self {
        FileTreeResp {
            id: 0,
            children: vec![],
            name: "".to_owned(),
            created_time: 0,
            updated_time: 0,
            user_id: 0,
            doc_status: 1,
            project_id: "".to_owned(),
            file_type: 1,
            file_id: "".to_owned(),
            parent: "".to_owned(),
            main_flag: 0,
        }
    }
}

impl From<&TexFile> for FileTreeResp {
    fn from(p: &TexFile) -> Self {
        Self {
            children: vec![],
            id: p.id,
            name: p.name.clone(),
            created_time: p.created_time,
            updated_time: p.updated_time,
            user_id: p.user_id,
            doc_status: p.doc_status,
            project_id: p.project_id.clone(),
            file_type: p.file_type,
            file_id: p.file_id.clone(),
            parent: p.parent.clone(),
            main_flag: p.main_flag,
        }
    }
}

impl IntoTree<String> for &FileTreeResp {
    type Output = FileTreeResp;

    fn get_id(&self) -> String {
        self.file_id.clone()
    }

    fn get_parent_id(&self) -> String {
        self.parent.clone()
    }

    fn convert(&self, children: Vec<Self::Output>) -> Self::Output {
        FileTreeResp {
            id: self.id,
            name: self.name.clone(),
            created_time: self.created_time,
            updated_time: self.updated_time,
            user_id: self.user_id,
            doc_status: self.doc_status,
            project_id: self.project_id.clone(),
            file_type: self.file_type,
            file_id: self.file_id.clone(),
            parent: self.parent.clone(),
            main_flag: self.main_flag,
            children,
        }
    }
}