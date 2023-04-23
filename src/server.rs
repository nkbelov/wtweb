#[allow(unused_imports)]
mod message;
mod render;
mod posts;

use posts::Posts;
use render::*;

use std::{net::SocketAddrV4, fs::{read_dir, ReadDir, read_to_string}, collections::HashMap};

use futures::SinkExt;
use tokio::{
    net::{TcpListener, TcpStream},
};
use tokio_util::codec::FramedWrite;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Result, Response},
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

fn to_not_found<T>(_: T) -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn get_styles() -> impl IntoResponse {
    let s = read_to_string("./styles/output.css").unwrap();
    println!("styles");
    (
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, mime::TEXT_CSS.as_ref())],
        s,
    )
}

async fn get_post(Path(name): Path<String>) -> Result<impl IntoResponse, StatusCode> {
    let posts = Posts::load();
    if let Some(post) = posts.get_post(&name) {
        let page = Page::new(Content::Article { text: post.text.clone() }, false);
        let html = render(&page);
        Ok(Html::from(html))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_img(Path((post_name, img_name)): Path<(String, String)>) -> Response {
    let posts = Posts::load();
    if let Some((bytes, mime))= posts.get_image(&post_name, &img_name) {
        return (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, mime)],
            bytes.clone()
        ).into_response()
    } else {
        (StatusCode::NOT_FOUND).into_response()
    }
}

async fn get_posts() -> Result<impl IntoResponse, StatusCode> {
    let posts = Posts::load();
    let previews = posts.previews();
    let page: Page = Page::new(Content::Posts { posts: previews }, false);
    let html = render(&page);
    Ok(Html::from(html))
}

async fn get_index() -> Result<impl IntoResponse, StatusCode> {
    let posts = Posts::load();
    let previews = posts.previews();
    let page: Page = Page::new(Content::Index { posts: previews }, false);
    let html = render(&page);
    Ok(Html::from(html))
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
        .route("/posts", get(get_posts))
        .route("/posts/:name", get(get_post))
        .route("/posts/:name/:img", get(get_img))
        .route("/output.css", get(get_styles));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}