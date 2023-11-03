#[derive(serde::Deserialize)]
pub struct EditProjNickname {
    pub user_id: i64,
    pub nickname: String,
}