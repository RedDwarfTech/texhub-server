use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct SrcPosResp {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl From<(String, u32, u32)> for SrcPosResp {
    fn from(items: (String, u32, u32)) -> Self {
        Self {
            file: items.0,
            line: items.1,
            column: items.2,
        }
    }
}
