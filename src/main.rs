extern crate openssl;
#[macro_use]
extern crate diesel;

use actix_web::{App, HttpServer};
use controller::{
    collar::collar_controller, file::file_controller, project::project_controller,
    template::template_controller,
};
use monitor::health_controller;

pub mod common;
pub mod controller;
pub mod model;
pub mod monitor;
pub mod service;
pub mod net;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    HttpServer::new(|| {
        App::new()
            .configure(collar_controller::config)
            .configure(health_controller::config)
            .configure(project_controller::config)
            .configure(template_controller::config)
            .configure(file_controller::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
