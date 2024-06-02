use crate::{
    model::{
        request::project::{
            query::share_query_params::ShareQueryParams, share::share_del::ShareDel,
        },
        response::project::share::tex_proj_share_resp::TexProjShareResp,
    },
    service::project::share::share_service::{del_share_bind_impl, get_collar_users},
};
use actix_web::{web, Responder};
use log::error;
use rust_wheel::{
    common::{
        util::model_convert::map_entity,
        wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    },
    model::user::login_user_info::LoginUserInfo,
};

pub async fn proj_share_list(form: web::Query<ShareQueryParams>) -> impl Responder {
    let collar_users = get_collar_users(&form.0).await;
    let resp: Vec<TexProjShareResp> = map_entity(collar_users);
    box_actix_rest_response(resp)
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
            .route("/del", web::delete().to(del_share_bind)),
    );
}
