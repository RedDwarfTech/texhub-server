#[derive(serde::Deserialize)]
pub struct GetSrcPosParams {
    pub project_id: String,
    pub main_file: String,
    pub page: u32,
    pub h: f32,
    pub v: f32,
}