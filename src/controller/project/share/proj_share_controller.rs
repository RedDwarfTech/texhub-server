use crate::{
    model::{
        request::project::{
            query::share_query_params::ShareQueryParams, share::share_del::ShareDel,
        },
        response::project::share::tex_proj_share_resp::TexProjShareResp,
    },
    service::project::share::share_service::{
        del_share_bind_impl, get_collar_permission, get_collar_users,
    },
};
use actix_web::{web, Responder};
use log::error;
use rust_wheel::{
    common::{
        util::model_convert::map_entity,
        wrapper::actix_http_resp::{
            box_actix_rest_response, box_error_actix_rest_response,
        },
    },
    model::user::login_user_info::LoginUserInfo,
};

use crate::model::request::project::share::collar_query_params::CollarQueryParams;

pub async fn proj_share_list(form: web::Query<ShareQueryParams>) -> impl Responder {
    let collar_users = get_collar_users(&form.0).await;
    let resp: Vec<TexProjShareResp> = map_entity(collar_users);
    box_actix_rest_response(resp)
}

/// 供 texhub-broadcast 在 WebSocket 建连时校验协作者身份。
///
/// 身份**只**来自 `LoginUserInfo`（由 AuthMiddleware 校验 JWT 后写入），
/// 刻意不接受调用方传入 user_id：否则 broadcast 侧只要被攻破，就能以任意
/// 用户身份查询任意项目的权限，鉴权边界形同虚设。
pub async fn get_collar_permission_of(
    form: web::Query<CollarQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    // 忽略 form.0.user_id，以 token 解析出的身份为准
    let params = CollarQueryParams {
        project_id: form.0.project_id.clone(),
        user_id: login_user_info.userId,
    };
    match get_collar_permission(&params).await {
        Ok(perm) => box_actix_rest_response(perm),
        Err(e) => {
            error!("get collar permission failed, {:?}", e);
            // DB 异常时返回明确的失败语义，调用方据此 fail-closed 拒绝连接，
            // 绝不能回退成"非成员但放行"或"查不到就当有权限"
            box_error_actix_rest_response(
                "",
                "collar_permission_failed".to_owned(),
                "failed to resolve collaboration permission".to_owned(),
            )
        }
    }
}

pub async fn del_share_bind(
    params: actix_web_validator::Query<ShareDel>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let result = del_share_bind_impl(&params.0, &login_user_info);
    if let Err(e) = result {
        error!("del share bind, {}", e);
        return box_error_actix_rest_response(
            "",
            "del_failed".to_owned(),
            "del share bind failed".to_owned(),
        );
    }
    box_actix_rest_response("ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/share")
            .route("/list", web::get().to(proj_share_list))
            .route("/del", web::delete().to(del_share_bind))
            .route("/permission", web::get().to(get_collar_permission_of)),
    );
}
