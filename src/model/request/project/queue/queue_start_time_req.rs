#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct QueueStartTimeReq {
    pub id: i64,
    pub start_time: i64,
}
