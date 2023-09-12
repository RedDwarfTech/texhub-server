#[derive(serde::Deserialize)]
pub struct EditProjReq {
    pub project_id: String,
    pub proj_name: String,
}