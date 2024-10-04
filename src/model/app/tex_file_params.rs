use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct TexFileParams {
    pub proj_id: String,
    pub main_name: String,
    pub user_id: i64
}