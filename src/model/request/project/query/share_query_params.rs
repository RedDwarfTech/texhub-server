use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct ShareQueryParams {
    #[validate(length(min = 1))]
    pub project_id: String
}
