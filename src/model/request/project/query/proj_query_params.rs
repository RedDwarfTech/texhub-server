#[derive(serde::Deserialize)]
pub struct ProjQueryParams {
    pub tag: Option<String>,
    pub role_id: Option<i32>,
}