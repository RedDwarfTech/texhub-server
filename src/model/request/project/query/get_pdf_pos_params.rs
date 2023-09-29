#[derive(serde::Deserialize)]
pub struct GetPdfPosParams {
    pub project_id: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
}