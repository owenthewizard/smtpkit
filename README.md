A modular, `#![no_std]` (requires [`alloc`]), sans-I/O library for the SMTP protocol.

`smtpkit` provides portable, protocol-aware building blocks for implementing
SMTP clients, servers, or middleware. It is designed to be reasonably small and
composable with optional features. 💌

# ✨ Features

- ⚙️ **(none)**: Includes core SMTP types such as commands and replies.
  - ✔️ Always enabled.
- 🧠 **`parse`:** Enables parsing of SMTP commands and parameters, useful for building decoders
  and protocol handlers.
  - ✔️ Enabled by default.
  - 🔋 Includes a ready-to-use [`Parser`] that can also serve as an example of how to use `parse`.

# 🎯 Design Goals

- ⚙️ **Sans I/O:** All logic is independent of any networking or I/O layer. Bring your own sync or
  async runtime!
- 🛠️ **Modular:** Add only what you need via Cargo features.
- 💼 **Portable:** Usable in `no_std` environments (requires [`alloc`]).
- 🚀 **Efficient** Leverages [`bytes`] for low-overhead, zero-copy data manipulation.

# 🧪 Example

```rust
use bytes::{Bytes, BytesMut};

// most types, Parser
use smtpkit::*;
// MAIL and RCPT types
use smtpkit::mail::*;
use smtpkit::rcpt::*;

// Parser state machine, with the default max buffer size
let mut parser = Parser::default();
// buffer from your I/O layer, e.g. TCP socket
let mut buf = BytesMut::with_capacity(4096);
// read some input
buf.extend_from_slice(b"EHLO hello.world\r\nMAIL FROM:<bob@example.com> RET=FULL SIZE=10240 ENVID=b0b's+20m@!+2B+2B\r\nRCPT TO:<alice@example.com>\r\nDATA\r\n");
// Let's parse some commands!
let helo = parser.parse(&mut buf);
// Ok(Some(Command::Ehlo(Host::Domain("hello.world"))))
let mail = parser.parse(&mut buf);
// Ok(Some(Command::Mail(Mail {
//    size: Some(10240),
//    ret: Some(Ret::Full),
//    envid: Some("b0b's+20m@!+2B+2B"),
//    auth: None,
//    body: None,
//    from: Email("bob@example.com"
// }))))
assert_eq!(XText::parse(Bytes::from("b0b's+20m@!+2B+2B")).unwrap().decode(), Bytes::from(&b"b0b's m@!++"[..]));
let rcpt = parser.parse(&mut buf);
// Ok(Some(Command::Rcpt(Rcpt {
//     auth: None,
//     orcpt: None,
//     notify: None,
//     to: Email("alice@example.com")
// })))
let data = parser.parse(&mut buf);
// we are waiting for more input
assert_eq!(parser.parse(&mut buf), Ok(None));
buf.extend_from_slice(&b"Hi Alice!\r\n.\r\nQUI"[..]);
let data = parser.parse(&mut buf);
assert_eq!(data, Ok(Some(Command::Data(Bytes::from(&b"Hi Alice!"[..])))));
// waiting for more input again
assert_eq!(parser.parse(&mut buf), Ok(None));
buf.extend_from_slice(&b"T\r\n"[..]);
assert_eq!(parser.parse(&mut buf), Ok(Some(Command::Quit)));
```
