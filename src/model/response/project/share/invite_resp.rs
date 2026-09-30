use serde::{Deserialize, Serialize};

/// 邀请凭证的对外视图。
///
/// 刻意**不含** token_cipher：那是内部存储形态，对外一律只给可读的 token 明文，
/// 且只有签发它的 Owner 能拿到。完整链接由为端用 token 拼装，不在后端下发，
/// 避免把"站点地址"这个可变配置硬编码进后端响应。
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TexInviteResp {
    pub project_id: String,
    /// 邀请码明文，仅在签发 / 查询时返回给 Owner。
    pub token: String,
    pub role_id: i32,
    /// 0 表示永不过期。
    pub expire_at: i64,
    /// 0 表示不限次。
    pub max_uses: i32,
    pub used_count: i32,
    pub created_time: i64,
}
