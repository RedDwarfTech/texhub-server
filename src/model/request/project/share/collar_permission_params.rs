use validator::Validate;

/**
 * 协作权限查询入参。
 *
 * 刻意**不含** user_id：调用方只能指定查哪个项目，查谁的身份一律取自
 * AuthMiddleware 校验过的 JWT（LoginUserInfo.userId）。
 *
 * 这里曾复用 CollarQueryParams 复用成 bug：该结构体的 `user_id: i64` 是必填，
 * 而调用方不会也不该传它，导致 `web::Query` 反序列化失败、接口固定返回 400。
 * 语义上也自相矛盾 —— 一个声称"忽略"的字段却又强制要求提供。
 */
#[derive(serde::Deserialize, Validate, Debug)]
pub struct CollarPermissionParams {
    #[validate(length(min = 1))]
    pub project_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::web::Query;

    /// 这组测试的目的是把「broadcast 实际发出的 query 串」钉死。
    ///
    /// 之前出过一次线上事故：这里复用了 `CollarQueryParams`，而它的
    /// `user_id: i64` 必填，但调用方按设计只发 project_id，导致
    /// `web::Query` 反序列化失败、接口恒返回 400、协作鉴权全线不可用。
    /// 那次改动通过了编译、也通过了前端侧 32 项检查，却没有一个用例
    /// 覆盖请求契约本身，所以问题一路漏到线上日志才暴露。
    #[test]
    fn accepts_query_without_user_id() {
        // 与 collar_permission_client.ts 拼接的 URL 保持一致
        let parsed = Query::<CollarPermissionParams>::from_query(
            "project_id=f68dcc41e68343049d9515f37cdee25b",
        )
        .expect("broadcast 发送的 query 串必须能被反序列化");
        assert_eq!(parsed.project_id, "f68dcc41e68343049d9515f37cdee25b");
    }

    #[test]
    fn ignores_supplied_user_id() {
        // 即使有人试图在 query 里塞 user_id，也必须被忽略而不是采信：
        // 身份只能来自 AuthMiddleware 校验过的 token。
        let parsed = Query::<CollarPermissionParams>::from_query(
            "project_id=proj-1&user_id=999",
        )
        .expect("带多余字段也应能解析");
        assert_eq!(parsed.project_id, "proj-1");
    }

    #[test]
    fn rejects_missing_project_id() {
        // 缺 project_id 属于契约错误，应当被拒（由 web::Query 产出 400），
        // 不能退化成"查询全部项目"。
        assert!(Query::<CollarPermissionParams>::from_query("").is_err());
    }
}
