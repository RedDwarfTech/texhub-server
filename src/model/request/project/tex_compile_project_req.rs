#[derive(serde::Deserialize,serde::Serialize)]
pub struct TexCompileProjectReq {
    pub project_id: String,
    pub file_name: String,
}