extern crate openssl;
#[macro_use]
extern crate diesel;

use actix_web::{HttpServer, App};
use controller::{collar::collar_controller, doc::doc_controller};
use log::warn;
use monitor::health_controller;

pub mod controller;
pub mod monitor;
pub mod model;
pub mod service;
pub mod common;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    HttpServer::new(|| {
        App::new()
            .configure(collar_controller::config)
            .configure(health_controller::config)
            .configure(doc_controller::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
