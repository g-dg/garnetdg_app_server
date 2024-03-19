//! Generic web application server that allows for more server-client and client-client interaction in a secure way

pub mod application;
pub mod auth;
pub mod config;
pub mod database;
pub mod datastore;
pub mod endpoints;
pub mod helpers;

#[cfg(test)]
mod tests;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use application::Application;
use config::Config;
use tokio::io;

/// Test endpoint
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

/// Application entry point
#[tokio::main]
async fn main() -> io::Result<()> {
    env_logger::init();

    // load config
    let config = Config::load_file("./config.json").await;

    let application = Application::build(&config).await;

    let result = HttpServer::new(move || App::new().route("/", web::get().to(hello)))
        .bind((config.server.host.clone(), config.server.port))?
        .run()
        .await;

    application.stop().await;

    result
}
