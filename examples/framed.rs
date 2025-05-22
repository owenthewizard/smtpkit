use futures_util::sink::SinkExt;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

use snail_smtp::*;

const CONNECTION: &str = "localhost:2500";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(CONNECTION).await?;
    println!("SMTP server listening on {CONNECTION}");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {addr}");

        tokio::spawn(async move {
            let mut stream = Framed::new(socket, codec::Smtp::new());

            while let Some(res) = stream.next().await {
                match res {
                    Ok(command) => {
                        println!("Received: {command:?}");
                        stream.send("250 OK".into()).await?;
                    }
                    Err(err) => {
                        eprintln!("Error: {err}");
                        stream.send("554 Error".into()).await?;
                    }
                }
            }
            stream.send("221 Bye".into()).await?;
            stream.close().await
        });
    }
}
