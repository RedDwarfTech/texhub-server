use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct PdfPosResp {
    pub page: i32,
    pub h: f32,
    pub v: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl From<(i32,f32,f32,f32,f32,f32,f32)> for PdfPosResp {
    fn from(items:(i32,f32,f32,f32,f32,f32,f32)) -> Self {
        Self {
            page: items.0,
            h: items.1,
            v: items.2,
            width: items.3,
            height: items.4,
            x: items.5,
            y: items.6,
        }
    }
}
