#[derive(serde::Deserialize,serde::Serialize,Clone)]
pub struct TexCompileQueueLog {
    pub project_id: String,
    pub version_no: String,
    pub file_name: String,
    pub qid: i64
}