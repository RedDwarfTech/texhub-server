use crate::model::request::profile::profile_active_req::ProfileActiveReq;
use actix_web::web;
use actix_web::HttpResponse;
use jemalloc_ctl::{Access, AsName};
use log::error;
use rust_wheel::common::wrapper::actix_http_resp::box_actix_rest_response;

const PROF_ACTIVE: &'static [u8] = b"prof.active\0";
const PROF_DUMP: &'static [u8] = b"prof.dump\0";
const PROFILE_OUTPUT: &'static [u8] = b"/opt/data/dump/profile.out\0";

pub async fn do_dump() -> HttpResponse {
    set_prof_active(true);
    let name = PROF_DUMP.name();
    let result = name.write(PROFILE_OUTPUT);
    if let Err(err) = result {
        error!("write dump file failed,{}", err);
    }
    box_actix_rest_response("ok")
}

fn set_prof_active(active: bool) {
    let name = PROF_ACTIVE.name();
    let result = name.write(active);
    if let Err(err) = result {
        error!("set_prof_active write active info failed,{}", err);
    }
}

pub async fn do_active(form: web::Query<ProfileActiveReq>) -> HttpResponse {
    let name = PROF_ACTIVE.name();
    let result = name.write(form.0.active);
    if let Err(err) = result {
        error!("write active info failed,{}", err);
    }
    box_actix_rest_response("ok")
}

pub async fn metrics() -> HttpResponse {
    HttpResponse::Ok().body("Hello, World!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/inner-tex/profile")
            .route("/dump", web::get().to(do_dump))
            .route("/active", web::get().to(do_active))
            .route("/metrics", web::get().to(metrics)),
    );
}
