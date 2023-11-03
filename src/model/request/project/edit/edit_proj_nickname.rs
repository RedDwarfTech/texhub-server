#[derive(serde::Deserialize)]
pub struct EditProjNickname {
    pub user_id: String,
    pub nickname: String,
}