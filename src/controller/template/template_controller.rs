use crate::{
    model::request::tpl::query::{
        tpl_detail_query_params::TplDetailQueryParams, tpl_query_params::TplQueryParams,
    },
    service::tpl::template_service::{get_tempalte_by_id, get_tpl_list, get_tpl_page_impl, get_tpl_partial_page_impl},
};
use actix_web::{web, Responder};
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;

pub async fn get_tpl(params: web::Query<TplQueryParams>) -> impl Responder {
    let docs = get_tpl_list(&params.0);
    box_actix_rest_response(docs)
}

pub async fn get_tpl_page(params: web::Query<TplQueryParams>) -> impl Responder {
    let docs = get_tpl_page_impl(&params.0);
    box_actix_rest_response(docs)
}

pub async fn get_tpl_partial_page(params: web::Query<TplQueryParams>) -> impl Responder {
    let docs = get_tpl_partial_page_impl(&params.0);
    box_actix_rest_response(docs)
}

pub async fn get_tpl_detail(params: web::Query<TplDetailQueryParams>) -> impl Responder {
    let docs = get_tempalte_by_id(&params.id);
    box_actix_rest_response(docs)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/tpl")
            .route("/list", web::get().to(get_tpl))
            .route("/page", web::get().to(get_tpl_page))
            .route("/partial/page",web::get().to(get_tpl_partial_page))
            .route("/detail", web::get().to(get_tpl_detail)),
    );
}
