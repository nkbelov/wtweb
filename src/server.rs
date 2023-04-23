#[allow(unused_imports)]
mod message;
mod render;

use bytes::Bytes;
use render::*;
use serde::Serialize;

use std::{net::SocketAddrV4, fs::{read_dir, ReadDir, read_to_string}, collections::HashMap};

use futures::SinkExt;
use tokio::{
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

fn not_found<T>(_: T) -> StatusCode {
    StatusCode::NOT_FOUND
}

#[derive(Debug)]
struct Post {
    title: String,
    abs: Option<String>,
    text: String,
    images: Vec<(String, Bytes)>
}

impl Post {

    fn to_preview(&self) -> PostPreview {
        PostPreview { title: self.title.clone(), abs: self.abs.clone() }
    }

}

#[derive(Debug, Serialize, Clone)]
struct PostPreview { title: String, abs: Option<String> }

impl Post {

    fn from_dir(path: &std::path::Path) -> std::io::Result<Self> {
        assert!(path.is_dir());
        let dir = read_dir(path)?;
        let mut md: Option<String> = None;
        let mut images: Vec<(String, Bytes)> = vec![];

        for entry in dir.into_iter().filter_map(|e| e.ok()) {
            assert!(entry.file_type()?.is_file());
            
            if entry.file_name().to_str().unwrap().ends_with(".md") {
                md = Some(read_to_string(entry.path())?);
            } else {
                let bytes = std::fs::read(entry.path())?;
                let name = entry.file_name().to_str().unwrap().to_owned();
                images.push((name, bytes.into()))
            }
        }

        let md = md.unwrap();
        let (title, abs) = extract_meta(&md);
        let body = Some(render_markdown(&md));

        let res = Self {
            title,
            abs,
            text: body.unwrap(), // FIXME: Don't unwrap
            images
        };

        Ok(res)
    }

    fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        assert!(path.is_file());
        let md = read_to_string(path)?;

        let (title, abs) = extract_meta(&md);
        let body = Some(render_markdown(&md));

        let res = Self {
            title,
            abs,
            text: body.unwrap(),
            images: vec![]
        };

        Ok(res)
    }
}

fn load_posts() -> HashMap<String, Post> {
    let mut posts = HashMap::<String, Post>::new();
    let base_dir = read_dir("posts").unwrap();

    for entry in base_dir {
        if let Ok(entry) = entry {
            let name = entry.file_name().to_str().unwrap().to_owned();

            if entry.file_type().unwrap().is_dir() {
                let post = Post::from_dir(&entry.path());
                posts.insert(name, post.unwrap());
            } else {
                let post = Post::from_file(&entry.path());
                posts.insert(name, post.unwrap());
            }
        }
    }

    posts
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
    let posts = load_posts();
    if let Some(post) = posts.get(&name) {
        let page = Page::new(Content::Article { text: post.text.clone() }, false);
        let html = render(&page);
        Ok(Html::from(html))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_index() -> Result<impl IntoResponse, StatusCode> {
    let posts = load_posts();
    let previews = posts.iter().map(|(_, p)| p.to_preview()).collect();
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
        .route("/posts/:name", get(get_post))
        .route("/output.css", get(get_styles));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}