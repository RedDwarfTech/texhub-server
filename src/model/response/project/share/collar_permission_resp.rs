use serde::{Deserialize, Serialize};

/**
 * 协作权限查询结果，供 texhub-broadcast 在 WS 建连时做 membership 校验。
 *
 * 只暴露判定所需的最小字段：是否成员 + 角色。绝不下发协作者名单，
 * 避免这个内部接口变成用户枚举工具。
 */
#[derive(Deserialize, Serialize, Default)]
#[allow(non_snake_case)]
pub struct CollarPermissionResp {
    pub project_id: String,
    pub user_id: i64,
    /// 1 = Owner, 2 = Collaborator, 0 = 非成员
    pub role_id: i32,
    pub is_member: bool,
    /// 当前是否存在可写权限（Owner / Collaborator 均可写）
    pub can_write: bool,
}
