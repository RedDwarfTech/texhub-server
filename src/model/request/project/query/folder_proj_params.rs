#[derive(serde::Deserialize)]
pub struct FolderProjParams {
    pub folder_id: i64,
    pub proj_type: i32,
    /// Fuzzy, case-insensitive match against the project name.
    pub keyword: Option<String>
}