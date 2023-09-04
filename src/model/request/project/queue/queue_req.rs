#[derive(serde::Deserialize,serde::Serialize)]
pub struct QueueReq {
    pub comp_status: Vec<i32>
}