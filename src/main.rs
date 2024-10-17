extern crate openssl;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rust_i18n;

i18n!("locales");

use crate::controller::profile::profile_controller;
use actix_multipart::MultipartError;
use actix_web::Error;
use actix_web::{App, HttpRequest, HttpServer};
use controller::appconf::appconf_controller;
use controller::{
    collar::collar_controller,
    file::{file_controller, file_version_controller},
    project::{
        proj_controller, proj_event_handler::consume_sys_events, share::proj_share_controller,
        snippet_controller,
    },
    template::template_controller,
};
use log::error;
use monitor::health_controller;
use rust_wheel::config::app::app_conf_reader::get_app_config;

pub mod common;
pub mod controller;
pub mod model;
pub mod monitor;
pub mod net;
pub mod service;
pub mod tests;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    rust_i18n::set_locale("zh-CN");
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let port: u16 = get_app_config("texhub.port").parse().unwrap();
    let address = ("0.0.0.0", port);
    consume_sys_events();
    HttpServer::new(|| {
        App::new()
            .configure(collar_controller::config)
            .configure(health_controller::config)
            .configure(proj_controller::config)
            .configure(template_controller::config)
            .configure(file_controller::config)
            .configure(profile_controller::config)
            .configure(snippet_controller::config)
            .configure(proj_share_controller::config)
            .configure(file_version_controller::config)
            .configure(appconf_controller::config)
    })
    .workers(3)
    .bind(address)?
    .run()
    .await
}

fn handle_multipart_error(err: MultipartError, _req: &HttpRequest) -> Error {
    error!("Multipart error: {}", err);
    return Error::from(err);
}
