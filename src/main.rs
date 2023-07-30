use actix_web::{HttpServer, App};
use controller::collar::collar_controller;
use monitor::health_controller;

pub mod controller;
pub mod monitor;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(collar_controller::config)
            .configure(health_controller::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
