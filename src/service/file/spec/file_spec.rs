use crate::model::diesel::tex::custom_tex_models::TexFile;

pub trait FileSpec {
    fn get_proj_file_count(&self, proj_id: &str) -> i64;
    fn get_file_by_id(&self, file_id: &str) -> Option<TexFile>;
}
