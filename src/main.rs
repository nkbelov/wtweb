
mod config;
mod file_server;
mod helpers;
//mod blog;
mod markdown;

use std::str::FromStr;
use std::path::PathBuf;
use std::fs;
use log::*;

use actix_web::{get, dev::*, web, http, http::header::*, App, HttpServer, HttpResponse, Result};
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::middleware;

use config::Config;
use file_server::FileServer;
use flexi_logger::{Logger, LogSpecification, LoggerHandle, FileSpec, Duplicate, LevelFilter};

#[get("/images/{name}")]
async fn image(fs: web::Data<FileServer>, name: web::Path<String>) -> HttpResponse {
    match fs.get_image(name.path()).await {
        Some(bytes) => {
            HttpResponse::Ok()
                .content_type("image")
                .body(bytes)
        }

        None => {
            HttpResponse::NotFound()
                .finish()
        }
    }
}

#[get("/styles")]
async fn styles(fs: web::Data<FileServer>) -> HttpResponse {
    match fs.get_styles().await {
        Some(body) => {
            HttpResponse::Ok()
                .content_type("text/css")
                .body(body)
        }

        None => {
            HttpResponse::NotFound()
                .finish()
        }
    }
}

#[get("/")]
async fn index(fs: web::Data<FileServer>) -> HttpResponse {
    match fs.get_index().await {
        Some(body) => {
            HttpResponse::Ok()
                .content_type("text/html")
                .body(body)
        }

        None => {
            HttpResponse::NotFound()
                .finish()
        }
    }
}

fn render_404<B: 'static>(mut response: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let fut = async {
        info!("Rendering 404 after receiving request to URI \"{}\"", &response.request().uri());
        let fs: &FileServer = helpers::get_app_data(&response);
        let page404 = fs.get_404().await.unwrap();
        response.response_mut()
                .headers_mut()
                .insert(CONTENT_TYPE, http::HeaderValue::from_static("Error"));
        let response = response.map_body(|_, _| ResponseBody::Other(Body::from(page404)));
        Ok(response)
    };

    return Ok(ErrorHandlerResponse::Future(Box::pin(fut)));
}

fn setup_logging() -> LoggerHandle {
    let mut builder = LogSpecification::builder();
    builder.default(LevelFilter::Debug);

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
    let _ = fs::create_dir(&config.temp_dir());
    let _ = setup_logging();

    let mut server = HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(index)
            .service(styles)
            .service(image)
            .route("/", web::post().to(|| HttpResponse::MethodNotAllowed()))
            .route("/", web::put().to(|| HttpResponse::MethodNotAllowed()))
            .route("/", web::patch().to(|| HttpResponse::MethodNotAllowed()))
            .route("/", web::delete().to(|| HttpResponse::MethodNotAllowed()))
            .data(FileServer::in_dir(PathBuf::from_str("resources/").unwrap()))
            .wrap(ErrorHandlers::new().handler(http::StatusCode::NOT_FOUND, render_404))
    });

    match config.rustls_config() {
        Some(tlsconfig) => {
            server = server.bind_rustls(&config.socket(), tlsconfig)?;
        }

        None => {
            server = server.bind(&config.socket())?;
        }
    }
    

    let ret = server.run().await;
    let _ = fs::remove_dir_all(&config.temp_dir());

    ret
}