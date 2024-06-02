use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct ShareDel {
    #[validate(range(min = 1))]
    pub id: i64,
    #[validate(length(min = 1))]
    pub project_id: String
}