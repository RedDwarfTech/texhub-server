use validator::Validate;

#[derive(serde::Deserialize, Validate)]
pub struct EditProjFolder {
    pub project_id: String,
    pub folder_id: i64,
    pub proj_type: i32
}