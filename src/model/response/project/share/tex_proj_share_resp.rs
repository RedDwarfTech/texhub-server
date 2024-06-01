use crate::model::diesel::tex::custom_tex_models::TexProjEditor;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct TexProjShareResp {
    pub collar_status: i32,
    pub nickname: String,
    pub created_time: i64,
    pub role_id: i32,
}

impl From<&TexProjEditor> for TexProjShareResp {
    fn from(item: &TexProjEditor) -> Self {
        Self {
            collar_status: item.collar_status,
            nickname: item.nickname.clone(),
            created_time: item.created_time,
            role_id: item.role_id
        }
    }
}
