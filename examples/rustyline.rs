use bytes::*;
use rustyline::DefaultEditor;

use smtpkit::*;

fn main() {
    let mut rl = DefaultEditor::new().expect("Failed to init editor");

    let mut parser = Parser::default();
    let mut buf = BytesMut::with_capacity(4096);

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rustyline=off".parse().unwrap()),
        )
        .init();

    while let Ok(line) = rl.readline("> ") {
        buf.extend_from_slice(line.as_bytes());
        buf.extend_from_slice(b"\r\n"); // Add CRLF for SMTP

        loop {
            match parser.parse(&mut buf) {
                Ok(None) => break,

                Ok(cmd) => {
                    println!("Parsed command: {cmd:#?}");
                }

                Err(e) => {
                    println!("Error parsing command: {e}");
                }
            }
        }

        rl.add_history_entry(line).expect("Failed to save history");
        //buf.clear();
    }
}
