#[allow(unused_imports)]

mod message;

use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::FramedWrite;
use futures::SinkExt;

use axum::{
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
        message_sink.send(message::Message::Text("HI".to_string())).await.unwrap();
    }
}

#[tokio::main]
async fn main() {

    // TODO: possible different way? 
    // TODO: join with the web server?
    // TODO: auto-restart on failure?
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
        loop {
            // The second item contains the IP and port of the new connection.
            let (socket, _) = listener.accept().await.unwrap();
            start_console(socket).await.unwrap();
        }
    });

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}