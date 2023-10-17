use actix_web::HttpResponse;
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;
use actix_web::web;

async fn healthz() -> HttpResponse {
    box_actix_rest_response("ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/health")
            .route("/healthz", web::get().to(healthz))
    );
}