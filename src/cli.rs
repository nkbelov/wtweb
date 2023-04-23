#[allow(unused_imports)]

mod message;

use std::net::SocketAddrV4;

use message::*;

use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let addr = SocketAddrV4::new("127.0.0.1".parse().unwrap(), message::PORT);
    let stream = TcpStream::connect(addr).await.unwrap();
    let (r, _) = stream.into_split();

    let mut message_stream = FramedRead::new(r, JsonCodec::new());
    //let lines_sink = FramedWrite::new(w, LinesCodec::new());

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(msg) => {
                dbg!(msg);
            }
            Err(e) => {
                println!("error on decoding from socket; error = {:?}", e);
            }
        }
    }

    println!("Closing CLI...");
}