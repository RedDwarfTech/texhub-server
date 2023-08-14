#[derive(serde::Deserialize)]
pub struct TexCompileProjectReq {
    pub project_id: String,
    pub req_time: i64
}