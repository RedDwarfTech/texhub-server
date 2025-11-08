#[derive(serde::Deserialize)]
pub struct DownloadFileQuery {
    pub file_id: String,
}