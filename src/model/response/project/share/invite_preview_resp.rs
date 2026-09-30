use serde::{Deserialize, Serialize};

/// 邀请预览：持有链接但尚未加入的用户在点"接受"之为看到的项目信息。
///
/// 只暴露展示所需的最小信息，且**不含 token** —— 预览接口不负责回显凭证。
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TexInvitePreviewResp {
    pub project_id: String,
    pub project_name: String,
    /// 邀请人昵称。
    pub inviter_name: String,
    pub role_id: i32,
    /// 0 表示永不过期。
    pub expire_at: i64,
}
