use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct CollarQueryParams {
    #[validate(length(min = 1))]
    pub project_id: String,
    pub user_id: i64
}
