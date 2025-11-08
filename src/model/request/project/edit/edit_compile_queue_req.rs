#[derive(serde::Deserialize)]
pub struct EditCompileQueueReq {
    pub expire_time: i64,
}