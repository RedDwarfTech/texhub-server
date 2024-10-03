use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct TplParams {
    pub tpl_id: i64,
    pub name: String,
    pub main_file_name: String,
    pub tpl_files_dir: String,
}