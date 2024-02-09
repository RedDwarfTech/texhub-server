use actix_web::{web, Responder};
use rust_wheel::{
    common::wrapper::actix_http_resp::box_actix_rest_response,
    model::user::login_user_info::LoginUserInfo,
};
use crate::{
    model::request::project::query::snippet_query_params::SnippetQueryParams,
    service::project::snippet_service::get_snippets,
};

pub async fn snippet_list(
    form: actix_web_validator::Json<SnippetQueryParams>,
    login_user_info: LoginUserInfo,
) -> impl Responder {
    let snippets = get_snippets(form.0, &login_user_info).await;
    box_actix_rest_response(snippets)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/tex/snippet").route("/code", web::get().to(snippet_list)));
}
