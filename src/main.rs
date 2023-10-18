extern crate openssl;
#[macro_use]
extern crate diesel;

use actix_web::{App, HttpServer};
use controller::{
    collar::collar_controller, file::file_controller, project::project_controller,
    template::template_controller,
};
use monitor::health_controller;
use rust_wheel::config::app::app_conf_reader::get_app_config;
use crate::controller::profile::profile_controller;

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
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let port: u16 = get_app_config("texhub.port").parse().unwrap();
    let address = ("0.0.0.0", port);
    HttpServer::new(|| {
        App::new()
            .configure(collar_controller::config)
            .configure(health_controller::config)
            .configure(project_controller::config)
            .configure(template_controller::config)
            .configure(file_controller::config)
            .configure(profile_controller::config)
    })
    .bind(address)?
    .run()
    .await
}
