#[derive(serde::Deserialize,serde::Serialize)]
pub struct TexCompileQueueReq {
    pub project_id: String,
}