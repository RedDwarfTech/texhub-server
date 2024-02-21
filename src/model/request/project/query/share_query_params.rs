use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct ShareQueryParams {
    pub proj_id: String
}
