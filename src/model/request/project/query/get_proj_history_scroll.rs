#[derive(serde::Deserialize)]
pub struct GetProjHistoryScroll {
    pub project_id: String,
    pub offset: Option<i64>
}