pub trait ProjSpec {
    fn get_proj_count_by_uid(&self, uid: &i64) -> i64;
}