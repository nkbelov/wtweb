
mod config;
mod file_server;
mod helpers;

use std::str::FromStr;
use std::path::PathBuf;
use log::*;

use actix_web::{get, dev::*, web, http, App, HttpServer, Responder, Result};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};

use config::Config;
use file_server::FileServer;
use flexi_logger::{Logger, LogSpecification, LoggerHandle, FileSpec, Duplicate, LevelFilter};

#[get("/")]
async fn hello(fs: web::Data<FileServer>) -> impl Responder {
    fs.get_index()
}

fn render_404<B: 'static>(mut response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let fut = async {
        info!("Rendering 404 after receiving request to URI \"{}\"", &response.request().uri());
        let fs: &FileServer = helpers::get_app_data(&response);
        let page404 = fs.get_404().unwrap();
        response.response_mut()
                .headers_mut()
                .insert(http::header::CONTENT_TYPE, http::HeaderValue::from_static("Error"));
        let response = response.map_body(|_, _| ResponseBody::Other(Body::from(page404)));
        Ok(response)
    };

    return Ok(ErrorHandlerResponse::Future(Box::pin(fut)));
}

fn setup_logging() -> LoggerHandle {
    let mut builder = LogSpecification::builder();
    builder.default(LevelFilter::Info);

    let logger = Logger::with(builder.finalize())
                            .log_to_file(FileSpec::default())
                            .duplicate_to_stderr(Duplicate::Warn)
                            .start()
                            .unwrap();
    logger
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

	let config = Config::load().unwrap();
    let logger = setup_logging();

    HttpServer::new(|| {
        App::new().service(hello)
                  .data(FileServer::in_dir(PathBuf::from_str("resources").unwrap()))
                  .wrap(ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, render_404))
    })
    .bind(&config.socket)?
    .run()
    .await
}