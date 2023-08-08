#[derive(serde::Deserialize)]
pub struct TexFileAddReq {
    pub name: String,
    pub parent: String,
    pub project_id: String,
    pub file_type: i32,
}