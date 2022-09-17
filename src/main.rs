
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};
use tokio_stream::StreamExt;
use futures::SinkExt;

use axum::{
    routing::get,
    Router,
};

async fn start_console(tcp_stream: TcpStream) -> std::io::Result<()> {
    let mut lines = Framed::new(tcp_stream, LinesCodec::new());

    while let Some(result) = lines.next().await {
        match result {
            Ok(line) => {
                let response = line;

                if let Err(e) = lines.send(response.as_str()).await {
                    println!("error on sending response; error = {:?}", e);
                }
            }
            Err(e) => {
                println!("error on decoding from socket; error = {:?}", e);
            }
        }
    }

    Ok(())
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