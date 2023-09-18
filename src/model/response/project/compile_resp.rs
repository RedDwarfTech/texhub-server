use serde::{Serialize, Deserialize};
use crate::model::diesel::tex::custom_tex_models::TexCompQueue;
use super::latest_compile::LatestCompile;

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct CompileResp {
    pub comp: LatestCompile,
    pub queue: TexCompQueue,
}

impl From<(LatestCompile,TexCompQueue)> for CompileResp {
    fn from(items:(LatestCompile,TexCompQueue)) -> Self {
        Self {
            comp: items.0,
            queue: items.1,
        }
    }
}
