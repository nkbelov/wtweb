#[allow(unused_imports)]

mod message;
mod render;

use std::{net::SocketAddrV4};

use tokio::{net::{TcpListener, TcpStream}};
use tokio_util::codec::FramedWrite;
use futures::SinkExt;

use axum::{
    routing::get,
    Router, response::{Html, IntoResponse, Result}, http::StatusCode,
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

    let p = Page::Index { name: "kek".to_string() };
    let s = render(&p);
    Ok(Html::from(s))
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

    // build our application with a single route
    let app = Router::new().route("/", get(get_index));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}