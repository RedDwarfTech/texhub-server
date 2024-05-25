use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct ShareQueryParams {
    pub project_id: String
}
