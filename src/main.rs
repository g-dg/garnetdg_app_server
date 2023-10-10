use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    config::load_config();

    HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
