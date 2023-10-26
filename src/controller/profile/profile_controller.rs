use crate::model::request::profile::profile_active_req::ProfileActiveReq;
use actix_web::web;
use actix_web::HttpResponse;
use jemalloc_ctl::{Access, AsName};
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;

const PROF_ACTIVE: &'static [u8] = b"prof.active\0";
const PROF_DUMP: &'static [u8] = b"prof.dump\0";
const PROFILE_OUTPUT: &'static [u8] = b"/opt/data/dump/profile.out\0";

pub async fn do_dump() -> HttpResponse {
    let name = PROF_DUMP.name();
    name.write(PROFILE_OUTPUT)
        .expect("Should succeed to dump profile");
    box_actix_rest_response("ok")
}

pub async fn do_active(form: web::Query<ProfileActiveReq>) -> HttpResponse {
    let name = PROF_ACTIVE.name();
    name.write(form.0.active)
        .expect("Should succeed to set prof");
    box_actix_rest_response("ok")
}

pub async fn metrics() -> HttpResponse {
    HttpResponse::Ok().body("Hello, World!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/tex/profile")
            .route("/dump", web::get().to(do_dump))
            .route("/active", web::get().to(do_active))
            .route("/metrics", web::get().to(metrics)),
    );
}
