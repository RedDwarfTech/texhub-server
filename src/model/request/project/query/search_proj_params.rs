use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct SearchProjParams {
    #[validate(length(min = 1))]
    pub project_id: String,
    #[validate(length(min = 1, max = 50))]
    pub keyword: String,
}
