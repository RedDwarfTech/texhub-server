#[derive(serde::Deserialize)]
pub struct SearchProjParams {
    pub project_id: String,
    pub keyword: String,
}