use crate::service::project::project_queue_service::update_expired_proj_queue;
use actix_web::{web, HttpResponse};
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;

async fn update_expired_queue_rec() -> HttpResponse {
    update_expired_proj_queue();
    box_actix_rest_response("ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inner-tex/project/queue")
            .route("/expire-check", web::post().to(update_expired_queue_rec)),
    );
}
