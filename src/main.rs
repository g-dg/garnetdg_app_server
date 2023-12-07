mod application;
mod config;
mod database;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use application::AppData;
use tokio::io;

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}
async fn hello2(app_data: web::Data<AppData>) -> impl Responder {
    let app_data = format!("{:?}", app_data);
    HttpResponse::Ok().body(app_data)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let config = config::load_config("./config.json").await;

    let database_schemas = database::connect_schemas(&config.databases);

    let app_data = AppData {
        database_schemas: database_schemas,
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .route("/", web::get().to(hello))
            .route("/hello", web::get().to(hello2))
    })
    .bind((config.server.host.clone(), config.server.port))?
    .run()
    .await
}
