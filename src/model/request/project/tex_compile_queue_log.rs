#[derive(serde::Deserialize,serde::Serialize,Clone)]
pub struct TexCompileQueueLog {
    pub project_id: String,
    pub version_no: i64,
    pub file_name: String,
    pub qid: i64
}