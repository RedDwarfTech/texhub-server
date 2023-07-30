use actix_web::{web, HttpResponse, Responder};

pub async fn health() -> impl Responder  {
    HttpResponse::Ok().body("Ok")
}

pub async fn liveness() -> impl Responder  {
    HttpResponse::Ok().body("Ok")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/texhub/actuator")
            .route("/liveness", web::get().to(liveness))
    );
}
