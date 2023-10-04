use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct SrcPosResp {
    pub file: String,
    pub line: i32,
    pub column: i32,
}

impl From<(String, i32, i32)> for SrcPosResp {
    fn from(items: (String, i32, i32)) -> Self {
        Self {
            file: items.0,
            line: items.1,
            column: items.2,
        }
    }
}
