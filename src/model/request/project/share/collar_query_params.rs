use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct CollarQueryParams {
    #[validate(length(min = 1))]
    pub project_id: String,
    pub user_id: i64
}
