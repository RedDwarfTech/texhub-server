use crate::model::diesel::tex::custom_tex_models::{TexCompQueue, TexProjEditor};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct TexProjShareResp {
    pub collar_status: i32,
    pub queue: TexCompQueue,
}

impl From<(TexProjEditor, TexCompQueue)> for TexProjShareResp {
    fn from(items: (TexProjEditor, TexCompQueue)) -> Self {
        Self {
            collar_status: 1,
            queue: items.1,
        }
    }
}
