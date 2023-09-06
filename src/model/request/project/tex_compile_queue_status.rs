#[derive(serde::Deserialize,serde::Serialize)]
pub struct TexCompileQueueStatus {
    pub id: i64,
    pub comp_status: i32
}