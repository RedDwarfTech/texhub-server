pub trait FileSpec {
    fn get_proj_file_count(&self, proj_id: &str) -> i64;
}
