use crate::model::request::project::queue::queue_start_time_req::QueueStartTimeReq;
use crate::service::project::project_queue_service::{
    update_expired_proj_queue, update_queue_start_time,
};
use actix_web::{web, HttpResponse};
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;

async fn update_expired_queue_rec() -> HttpResponse {
    update_expired_proj_queue();
    box_actix_rest_response("ok")
}

async fn update_queue_start_time_rec(form: web::Json<QueueStartTimeReq>) -> HttpResponse {
    update_queue_start_time(&form.0)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inner-tex/queue")
            .route("/expire-check", web::post().to(update_expired_queue_rec))
            .route("/start-time", web::put().to(update_queue_start_time_rec)),
    );
}
