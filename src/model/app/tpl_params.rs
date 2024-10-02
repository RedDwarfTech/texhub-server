use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct TplParams {
    pub tpl_id: i64,
    pub name: i64,
    pub main_file_name: String,
}