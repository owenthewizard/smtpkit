use smtpkit::*;

use bytes::BytesMut;
use std::io::{self, Read};

#[path = "../src/tracing_stub.rs"]
mod log;

fn main() {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    log::info!("Hello world!");

    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut buf = [0u8; 4096]; // reasonable chunk size
    let mut bytes = BytesMut::new();

    let mut parser = Parser::default();

    loop {
        match handle.read(&mut buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                bytes.extend_from_slice(&buf[..n]);
                loop {
                    match parser.parse(&mut bytes) {
                        Ok(Some(_cmd)) => {
                            //log::trace!(?cmd, "Parsed command");
                            //log::info!("OK");
                        }
                        Ok(None) => break, // No complete command yet
                        Err(_e) => {
                            //log::error!(?e, "Error parsing command");
                            break; // Exit the loop on error
                        }
                    }
                }
            }
            Err(_e) => {
                //log::error!(?e, "Failed to read from stdin");
                break;
            }
        }
    }
}
