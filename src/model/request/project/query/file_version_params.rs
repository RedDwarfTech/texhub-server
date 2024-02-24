use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct FileVersionParams {
    pub id: i64
}
