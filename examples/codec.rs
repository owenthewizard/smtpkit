use std::io;

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

use smtpkit::{Command, Parser, ToBytes};

#[derive(Debug, Default)]
struct Smtp(Parser);

impl Smtp {
    fn new(size: usize) -> Self {
        Self(Parser::new(size))
    }
}

#[derive(Debug, derive_more::Display, thiserror::Error)]
enum Error {
    Io(#[from] io::Error),
    Smtp(#[from] smtpkit::Error),
}

impl Decoder for Smtp {
    type Item = Result<Command, smtpkit::Error>;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let res = self.0.parse(src);
        Ok(res.transpose())
    }
}

impl Encoder<Command> for Smtp {
    type Error = io::Error; // Infallible

    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        item.to_bytes_into(dst);

        Ok(())
    }
}

impl Encoder<&[u8]> for Smtp {
    type Error = io::Error; // Infallible

    fn encode(&mut self, item: &[u8], dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_util::codec::Framed;

    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let listener = TcpListener::bind("127.0.0.1:2500").await?;
    println!("Listening on {}", listener.local_addr()?);

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted from: {addr}");

        tokio::spawn(async move {
            let mut framed = Framed::new(socket, Smtp::new(100));

            while let Some(result) = framed.next().await {
                match result {
                    Ok(Ok(line)) => {
                        println!("[{addr}] Got: {line}");
                        framed.send(format!("Got: {line}\r\n").as_bytes()).await?;
                    }
                    Ok(Err(e)) => {
                        eprintln!("[{addr}] Error: {e:?}");
                        framed.send(format!("Error: {e:?}\r\n").as_bytes()).await?;
                    }
                    Err(Error::Io(e)) => {
                        eprintln!("[{addr}] Fatal error: {e:?}");
                        return framed
                            .send(format!("Fatal Error: {e:?}\r\n").as_bytes())
                            .await;
                    }
                    Err(Error::Smtp(_)) => unreachable!(),
                }
            }
            Ok(())
        })
        .await??;
    }
}
