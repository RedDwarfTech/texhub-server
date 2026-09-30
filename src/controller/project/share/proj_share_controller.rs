use crate::{
    model::{
        request::project::{
            query::share_query_params::ShareQueryParams,
            share::invite_create::{InvitePreviewParams, InviteQueryParams, TexInviteCreateReq},
            share::invite_revoke::TexInviteRevokeReq,
            share::share_del::ShareDel,
        },
        response::project::share::tex_proj_share_resp::TexProjShareResp,
    },
    service::project::share::{
        invite_service::{create_or_reuse_invite, get_invite, preview_invite, revoke_invite},
        share_service::{del_share_bind_impl, get_collar_permission, get_collar_users},
    },
};
use actix_web::{web, Responder};
use log::{error, info};
use rust_wheel::{
    common::{
        util::model_convert::map_entity,
        wrapper::actix_http_resp::{
            box_actix_rest_response, box_err_actix_rest_response, box_error_actix_rest_response,
        },
    },
    model::user::login_user_info::LoginUserInfo,
};

use crate::model::request::project::share::collar_permission_params::CollarPermissionParams;

pub async fn proj_share_list(form: web::Query<ShareQueryParams>) -> impl Responder {
    let collar_users = get_collar_users(&form.0).await;
    let resp: Vec<TexProjShareResp> = map_entity(collar_users);
    box_actix_rest_response(resp)
}

/// 供 texhub-broadcast 在 WebSocket 据连时校验协作者身份。
///
/// 身份**只**来自 `LoginUserInfo`（由 AuthMiddleware 校验 JWT 后写入），
/// 刻意不接受调用方传入 user_id：否则 broadcast 侧只要被攻破，就能以任意
/// 用户身份查询任意项目的权限，鉴权边界形同虚设。
pub async fn get_collar_permission_of(
    form: web::Query<CollarPermissionParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    // 请求体里根本没有 user_id 这个字段可读，身份只能来自 token
    match get_collar_permission(&form.0.project_id, login_user_info.userId).await {
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

/// 签发（或复用）项目邀请链接。仅项目 Owner 可调用。
///
/// 重复调用不会作废已发出的链接：service 命中有效凭证时直接返回同一条，
/// 保持"链接稳定、随时可撤销"的产品语义。
pub async fn create_invite(
    form: web::Json<TexInviteCreateReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    match create_or_reuse_invite(&form.0, login_user_info.userId) {
        Ok(resp) => box_actix_rest_response(resp),
        Err(e) => {
            error!("create invite failed, {:?}", e);
            box_err_actix_rest_response(e)
        }
    }
}

/// 读机项目当为的邀请凭证（不签发）。仅 Owner 可调用；没有可用凭证时返回空。
pub async fn get_invite_handler(
    form: web::Query<InviteQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    match get_invite(&form.0.project_id, login_user_info.userId) {
        Ok(resp) => box_actix_rest_response(resp),
        Err(e) => {
            error!("get invite failed, {:?}", e);
            box_err_actix_rest_response(e)
        }
    }
}

/// 撤销项目当为的邀请链接。仅项目 Owner 可调用，重复调用幂等。
pub async fn revoke_invite_handler(
    form: web::Json<TexInviteRevokeReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    match revoke_invite(&form.0.project_id, login_user_info.userId) {
        Ok(_) => box_actix_rest_response("ok"),
        Err(e) => {
            error!("revoke invite failed, {:?}", e);
            box_err_actix_rest_response(e)
        }
    }
}

/// 预览邀请链接背后的项目信息。
///
/// 只需要登录，**不需要**已经是项目成员 —— 这正是"还没接受邀请"的状态。
/// 只回显展示所需的最小信息，不回显 token 本身。
pub async fn preview_invite_handler(
    form: web::Query<InvitePreviewParams>,
    _login_user_info: LoginUserInfo,
) -> impl Responder {
    match preview_invite(&form.0.token) {
        Ok(resp) => box_actix_rest_response(resp),
        Err(e) => {
            // 预览失败是正常业务分支（链接过期 / 已撤销 / 不存在），
            // 用 info 级别，避免无效链接的反复尝试把 error 日志刷爆。
            info!("preview invite rejected: {:?}", e);
            box_err_actix_rest_response(e)
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/share")
            .route("/list", web::get().to(proj_share_list))
            .route("/del", web::delete().to(del_share_bind))
            .route("/permission", web::get().to(get_collar_permission_of))
            .route("/invite", web::post().to(create_invite))
            .route("/invite", web::get().to(get_invite_handler))
            .route("/invite/revoke", web::post().to(revoke_invite_handler))
            .route("/invite/preview", web::get().to(preview_invite_handler)),
    );
}
