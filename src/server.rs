#[allow(unused_imports)]
mod message;
mod render;

use std::net::SocketAddrV4;

use futures::SinkExt;
use tokio::{
    fs::read_to_string,
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::FramedWrite;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Result},
    routing::get,
    Router,
};

async fn start_console(tcp_stream: TcpStream) -> std::io::Result<()> {
    use tokio::time::{self, Duration};

    let (_, w) = tcp_stream.into_split();
    let mut message_sink = FramedWrite::new(w, message::JsonCodec::new());
    let mut interval = time::interval(Duration::from_millis(1000));

    loop {
        interval.tick().await;
        message_sink
            .send(message::Message::Text("HI".to_string()))
            .await?;
    }
}

async fn get_index() -> Result<impl IntoResponse, StatusCode> {
    use render::*;

    let p: Page = Page {
        r#type: PageType::Index,
        content: Content::new(),
    };
    
    let s = render(&p);
    Ok(Html::from(s))
}

async fn get_static(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    println!("name {name}");
    use render::*;

    let markdown = read_to_string("posts/layout/layout.md").await.unwrap();
    let mut p: Page = Page {
        r#type: PageType::Article,
        content: Content::new(),
    };
    
    p.content.text = Some(markdown);
    let s = render(&p);
    Ok(Html::from(s))
}

async fn get_styles() -> impl IntoResponse {
    let s = read_to_string("./styles/output.css").await.unwrap();
    println!("styles");
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime::TEXT_CSS.as_ref())],
        s,
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
        .route("/:path", get(get_static))
        .route("/output.css", get(get_styles));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}