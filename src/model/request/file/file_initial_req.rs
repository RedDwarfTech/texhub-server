#[derive(serde::Deserialize,serde::Serialize)]
pub struct FileInitialReq {
    pub project_id: String,
    pub doc_id: String,
    pub file_content: String,
}