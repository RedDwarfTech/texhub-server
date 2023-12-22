#[derive(serde::Deserialize)]
pub struct ArchiveProjReq {
    pub project_id: String,
    pub archive_status: i32
}