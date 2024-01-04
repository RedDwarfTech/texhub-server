#[derive(serde::Deserialize)]
pub struct DownloadProj {
    pub project_id: String,
    pub version: String
}