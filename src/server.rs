#[allow(unused_imports)]

mod message;
mod render;

use std::{net::SocketAddrV4};

use tokio::{net::{TcpListener, TcpStream}, fs::{read, read_to_string}};
use tokio_util::{codec::FramedWrite, io::read_buf};
use futures::SinkExt;

use axum::{
    routing::get,
    Router, response::{Html, IntoResponse, Result}, http::{StatusCode, Response},
    extract::Path
};

async fn start_console(tcp_stream: TcpStream) -> std::io::Result<()> {
    use tokio::time::{self, Duration};

    let (_, w) = tcp_stream.into_split();
    let mut message_sink = FramedWrite::new(w, message::JsonCodec::new());
    let mut interval = time::interval(Duration::from_millis(1000));

    loop {
        interval.tick().await;
        message_sink.send(message::Message::Text("HI".to_string())).await?;
    }
}

async fn get_index() -> Result<impl IntoResponse, StatusCode> {
    use render::*;

    let p: Page = Page { r#type: PageType::Index, content:  Content::new() };
    let s = render(&p);
    Ok(Html::from(s))
}

async fn get_page(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    println!("name {name}");
    use render::*;

    let p: Page = Page { r#type: PageType::Index, content: Content::new() };
    let s = render(&p);
    Ok(Html::from(s))
}

async fn get_styles() -> impl IntoResponse {
    let s = read_to_string("./styles/output.css").await.unwrap();
    println!("styles");
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime::TEXT_CSS.as_ref())],
        s
    )
}

#[tokio::main]
async fn main() {
    
    // TODO: possible different way? 
    // TODO: join with the web server?
    // TODO: auto-restart on failure?
    tokio::spawn(async move {
        let addr = SocketAddrV4::new("127.0.0.1".parse().unwrap(), message::PORT);
        let listener = TcpListener::bind(addr).await.unwrap();
        loop {
            // The second item contains the IP and port of the new connection.
            let (socket, _) = listener.accept().await.unwrap();
            start_console(socket).await.unwrap();
        }
    });

    let app = Router::new()
        .route("/", get(get_index))
        .route("/:path", get(get_page))
        .route("/output.css", get(get_styles));
        
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}