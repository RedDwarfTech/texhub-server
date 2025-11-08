use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct RenameProjFolder {
    pub folder_name: String,
    pub folder_id: i64
}