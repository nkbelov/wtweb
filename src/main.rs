
mod config;

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use config::Config;

// #[get("/")]
// async fn index() -> impl Responder {

// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {

	let config = Config::load();


    HttpServer::new(|| {
        App::new().route("/", web::get().to(|| HttpResponse::Ok()))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}