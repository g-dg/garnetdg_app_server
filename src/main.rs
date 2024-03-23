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

use application::Application;
use config::Config;
use tokio::io;

/// Application entry point
#[tokio::main]
async fn main() -> io::Result<()> {
    // load config
    let config = Config::load_file("./config.json").await;

    let application = Application::build(&config).await;

    let result = io::Result::Ok(());

    application.stop().await;

    result
}
