use actix_web::{HttpServer, App};

mod demo_controller;
mod synctex;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(demo_controller::config)
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
