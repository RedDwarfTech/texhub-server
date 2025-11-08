use validator::Validate;

#[derive(serde::Deserialize, Validate, Debug)]
pub struct ProjDynParams {
    pub tpl_id: i64,
    pub name: String,
    pub main_file_name: String,
    pub tpl_files_dir: String,
    pub proj_source_type: i16,
    pub proj_source: String,
}