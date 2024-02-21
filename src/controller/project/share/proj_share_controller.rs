use crate::{
    model::request::{
        project::query::share_query_params::ShareQueryParams,
        snippet::{del::snippet_del::SnippetDel, edit::snippet_req::SnippetReq},
    },
    service::project::{
        share::share_service::get_collar_users,
        snippet_service::{del_snippet_impl, edit_snippet_impl},
    },
};
use actix_web::{web, Responder};
use log::error;
use rust_wheel::{
    common::wrapper::actix_http_resp::{box_actix_rest_response, box_error_actix_rest_response},
    model::user::login_user_info::LoginUserInfo,
};

pub async fn proj_share_list(
    form: web::Query<ShareQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let collar_users = get_collar_users(form.0, &login_user_info).await;
    box_actix_rest_response(collar_users)
}

pub async fn edit_snippet(
    form: actix_web_validator::Json<SnippetReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let snippets = edit_snippet_impl(&form.0, &login_user_info);
    box_actix_rest_response(snippets)
}

pub async fn add_snippet(
    form: actix_web_validator::Json<SnippetReq>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let snippets = edit_snippet_impl(&form.0, &login_user_info);
    box_actix_rest_response(snippets)
}

pub async fn del_snippet(
    form: actix_web_validator::Json<SnippetDel>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let result = del_snippet_impl(&form.0.id, &login_user_info);
    if let Err(e) = result {
        error!("del snippet failed, {}", e);
        return box_error_actix_rest_response(
            "",
            "del_failed".to_owned(),
            "del snippet failed".to_owned(),
        );
    }
    box_actix_rest_response("ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/share")
            .route("/list", web::get().to(proj_share_list))
            .route("/edit", web::put().to(edit_snippet))
            .route("/add", web::put().to(edit_snippet))
            .route("/del", web::delete().to(del_snippet)),
    );
}
