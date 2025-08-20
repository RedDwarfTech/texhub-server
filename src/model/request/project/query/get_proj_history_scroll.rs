#[derive(serde::Deserialize)]
pub struct GetProjHistoryScroll {
    pub project_id: String,
    pub file_id: Option<String>,
    pub offset: Option<i64>,
    pub page_size: Option<i32>
}