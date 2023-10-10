mod config;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use tokio::io;

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let config = config::load_config("./config.json").await;

    HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind((config.server.host.clone(), config.server.port))?
        .run()
        .await
}
