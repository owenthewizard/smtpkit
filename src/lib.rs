#![feature(addr_parse_ascii)]
#![feature(int_from_ascii)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

//! A modular, `#![no_std]`, sans-I/O library for the SMTP protocol.
//!
//! `smtpkit` provides portable, protocol-aware building blocks for implementing
//! SMTP clients, servers, or middleware. It is designed to be reasonably small and
//! composable with optional features. 💌
//!
//! # ✨ Features
//!
//! - ⚙️ **(none)**: Includes core SMTP types such as commands and replies.
//! - 🧠 **`parse`:** Enables parsing of SMTP commands and parameters, useful for building decoders and protocol handlers.
//!
//! # 🎯 Design Goals
//!
//! - ⚙ **Sans I/O:** All logic is independent of any networking or I/O layer. Bring your own sync or
//! async runtime!
//! - 🛠️ **Modular:** Add only what you need via Cargo features.
//! - 💼 **Portable:** Usable in `no_std` environments (**`parse`** requires `alloc`).
//! - 🚀 **Efficient** Leverage [`bytes`] for low-overhead, zero-copy data manipulation.
//!
//! # 🧪 Example
//!
//! ```rust
//! // import most types
//! use smtpkit::*;
//! // import other command-specific types
//! use smtpkit::mail::*;
//! use smtpkit::rcpt::*;
//!
//! let line = b"MAIL FROM:<bob@example.com> RET=FULL SIZE=10240 ENVID=b0b's+20m@!+2B+2B\r\n";
//! let parsed = Command::parse(line);
//! ```

pub(crate) use bstr::ByteSlice;
pub(crate) use bytes::Bytes;

pub use bytes;

mod types;
pub use types::*;

mod parse;
#[cfg(feature = "parse")]
pub use parse::*;

pub mod codec;

pub(crate) fn is_xchar(input: u8) -> bool {
    matches!(input, b'!'..=b'*' | b','..=b'<' | b'>'..=b'~')
}

pub(crate) trait Helpers: Sized {
    fn split_once(&self, delim: u8) -> Option<(Self, Self)>;
    fn strip_prefix_ci(&self, prefix: &[u8]) -> Option<Self>;
    fn strip_angled(&self) -> Option<Self>;
    fn strip_brackets(&self) -> Option<Self>;
}

impl Helpers for Bytes {
    fn split_once(&self, delim: u8) -> Option<(Self, Self)> {
        self.iter()
            .position(|&x| x == delim)
            .map(|pos| (self.slice(pos..), self.slice(pos + 1..)))
    }

    fn strip_prefix_ci(&self, prefix: &[u8]) -> Option<Self> {
        if self.len() < prefix.len() {
            return None;
        }

        if self[..prefix.len()].eq_ignore_ascii_case(prefix) {
            return Some(self.slice(prefix.len()..));
        }

        None
    }

    fn strip_angled(&self) -> Option<Self> {
        if self.starts_with(b"<") && self.ends_with(b">") {
            return Some(self.slice(1..self.len() - 1));
        }

        None
    }

    fn strip_brackets(&self) -> Option<Self> {
        if self.starts_with(b"[") && self.ends_with(b"]") {
            return Some(self.slice(1..self.len() - 1));
        }

        None
    }
}
