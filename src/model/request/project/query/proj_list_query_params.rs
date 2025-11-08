#[derive(serde::Deserialize)]
pub struct ProjListQueryParams {
    pub proj_source_type: Option<i16>,
    pub name: Option<String>,
}