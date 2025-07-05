use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct FileVersionParamsV1 {
    pub id: i64
}
