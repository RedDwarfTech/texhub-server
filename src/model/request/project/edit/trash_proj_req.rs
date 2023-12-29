#[derive(serde::Deserialize)]
pub struct TrashProjReq {
    pub project_id: String,
    pub trash: i32
}