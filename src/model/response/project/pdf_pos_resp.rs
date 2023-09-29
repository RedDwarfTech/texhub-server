use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct PdfPosResp {
    pub page: i32,
    pub h: f64,
    pub v: f64,
    pub width: f64,
    pub height: f64,
}