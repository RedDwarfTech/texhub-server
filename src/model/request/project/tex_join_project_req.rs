/// 通过邀请链接加入项目。
///
/// 刻意**只保留** `token`，删除原来的 `project_id`：加入哪个项目必须由服务端
/// 从 token 反查决定。曾经这里同时接受 `project_id`，等于只要知道任意项目 ID
/// 就能自助把自己插进 `tex_proj_editor`，是横向越权的直接来源。
#[derive(serde::Deserialize, Debug)]
pub struct TexJoinProjectReq {
    pub token: String,
}
