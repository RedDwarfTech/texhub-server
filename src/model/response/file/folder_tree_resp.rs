use crate::model::diesel::tex::custom_tex_models::TexFolderTree;
use rust_wheel::common::util::convert_to_tree_generic::IntoTree;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct FolderTreeResp {
    pub id: i64,
    pub name: String,
    pub project_id: String,
    pub file_type: i32,
    pub file_path: String,
    pub file_id: String,
    pub parent: String,
    pub children: Vec<FolderTreeResp>,
}

impl Default for FolderTreeResp {
    fn default() -> Self {
        FolderTreeResp {
            id: 0,
            children: vec![],
            name: "".to_owned(),
            project_id: "".to_owned(),
            file_type: 1,
            file_id: "".to_owned(),
            parent: "".to_owned(),
            file_path: "".to_owned(),
        }
    }
}

impl From<&TexFolderTree> for FolderTreeResp {
    fn from(p: &TexFolderTree) -> Self {
        Self {
            children: vec![],
            id: p.id,
            name: p.name.clone(),
            project_id: p.project_id.clone(),
            file_type: p.file_type,
            file_path: p.file_path.clone(),
            file_id: p.file_id.clone(),
            parent: p.parent.clone(),
        }
    }
}

impl IntoTree<String> for &FolderTreeResp {
    type Output = FolderTreeResp;

    fn get_id(&self) -> String {
        self.file_id.clone()
    }

    fn get_parent_id(&self) -> String {
        self.parent.clone()
    }

    fn convert(&self, children: Vec<Self::Output>) -> Self::Output {
        FolderTreeResp {
            id: self.id,
            name: self.name.clone(),
            project_id: self.project_id.clone(),
            file_path: self.file_path.clone(),
            file_type: self.file_type,
            file_id: self.file_id.clone(),
            parent: self.parent.clone(),
            children,
        }
    }
}
