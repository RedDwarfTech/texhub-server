#[derive(serde::Deserialize)]
pub struct GetProjPageHistory {
    pub project_id: String,
    pub page_num: Option<i64>,
    pub page_size: Option<i64>,
}