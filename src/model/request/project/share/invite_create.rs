/// 签发（或复用）项目邀请凭证。
///
/// 刻意不接受调用方指定 role_id：目为只有 Collaborator=2 这一个可授予的角色，
/// 让为端传只会多一个能被利用来提权的入口。
#[derive(serde::Deserialize)]
pub struct TexInviteCreateReq {
    pub project_id: String,
    /// 有效期（天）。缺省或 0 表示永不过期。
    pub expire_days: Option<i64>,
    /// 最大使用次数。缺省或 0 表示不限次。
    pub max_uses: Option<i32>,
}

/// 预览邀请链接的入参。
///
/// 单独一个 query DTO（而不是复用 TexInviteCreateReq）：预览只要 token，
/// 那些签发用的可选参数在这里语义上毫无意义，留着只会让人误以为它们生效。
#[derive(serde::Deserialize, Debug)]
pub struct InvitePreviewParams {
    pub token: String,
}

/// 读机当为邀请凭证的入参（只读，不签发）。
#[derive(serde::Deserialize, Debug)]
pub struct InviteQueryParams {
    pub project_id: String,
}
