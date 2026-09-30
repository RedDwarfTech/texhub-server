/// 撤销项目当为的邀请凭证。
#[derive(serde::Deserialize)]
pub struct TexInviteRevokeReq {
    pub project_id: String,
}
