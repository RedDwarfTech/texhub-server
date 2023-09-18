#[derive(serde::Deserialize,serde::Serialize, Debug)]
pub struct TexCompileQueueStatus {
    pub id: i64,
    pub comp_status: i32,
    pub comp_result: i32,
}