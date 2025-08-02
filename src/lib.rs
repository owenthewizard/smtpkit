#![feature(addr_parse_ascii)]
#![feature(int_from_ascii)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

extern crate alloc;

pub(crate) use bstr::ByteSlice;
pub(crate) use bytes::{Bytes, BytesMut};

mod types;
pub use types::*;

mod parse;
#[cfg(feature = "parse")]
pub use parse::*;

mod parser;
#[cfg(feature = "parse")]
pub use parser::*;

pub mod max {
    /// Maximum length of the local part of an email address.
    pub const LOCAL_PART: usize = 64;

    /// Maximum length of the domain part of an email address.
    pub const DOMAIN: usize = 255;

    /// Maximum length of an email address, **excluding** the `<>`.
    pub const EMAIL: usize = 254;

    /// Maximum length of a command line, **excluding** the trailing CRLF.
    pub const COMMAND_LINE: usize = 510;

    /// Maximum length of a `DATA` line, **excluding** the trailing CRLF.
    pub const DATA_LINE: usize = 998;
}

mod tracing_stub;
#[allow(
    unused_imports,
    reason = "only parse feature uses tracing right now, may change"
)]
pub(crate) use tracing_stub as log;

pub(crate) use core::fmt;

pub(crate) fn is_xchar(input: u8) -> bool {
    matches!(input, b'!'..=b'*' | b','..=b'<' | b'>'..=b'~')
}

pub(crate) trait Helpers: Sized {
    fn split_once(&self, delim: u8) -> Option<(Self, Self)>;
    #[cfg(feature = "parse")]
    fn strip_prefix_ci(&self, prefix: &[u8]) -> Option<Self>;
    #[cfg(feature = "parse")]
    fn strip_angled(&self) -> Option<Self>;
    fn strip_brackets(&self) -> Option<Self>;
}

impl Helpers for Bytes {
    fn split_once(&self, delim: u8) -> Option<(Self, Self)> {
        self.as_ref()
            .find_byte(delim)
            .map(|pos| (self.slice(..pos), self.slice(pos + 1..)))
    }

    #[cfg(feature = "parse")]
    fn strip_prefix_ci(&self, prefix: &[u8]) -> Option<Self> {
        if self.len() < prefix.len() {
            return None;
        }

        if self[..prefix.len()].eq_ignore_ascii_case(prefix) {
            return Some(self.slice(prefix.len()..));
        }

        None
    }

    #[cfg(feature = "parse")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::basic_colon(b"a:b", b':', Some((&b"a"[..], &b"b"[..])))]
    #[case::basic_equal(b"key=value", b'=', Some((&b"key"[..], &b"value"[..])))]
    #[case::multiple_delimiters(b"a:b:c", b':', Some((&b"a"[..], &b"b:c"[..])))]
    #[case::delimiter_not_present(b"abc", b':', None)]
    #[case::empty_input(b"", b':', None)]
    #[case::empty_left(b":b", b':', Some((&b""[..], &b"b"[..])))]
    #[case::empty_right(b"a:", b':', Some((&b"a"[..], &b""[..])))]
    #[case::both_empty(b":", b':', Some((&b""[..], &b""[..])))]
    #[case::unicode_utf8(b"\xce\xb1=\xce\xb2", b'=', Some((&b"\xce\xb1"[..], &b"\xce\xb2"[..])))] // α=β
    #[case::chinese_utf8(b"\xe4\xbd\xa0\xe5\xa5\xbd=\xe4\xb8\x96\xe7\x95\x8c", b'=', Some((&b"\xe4\xbd\xa0\xe5\xa5\xbd"[..], &b"\xe4\xb8\x96\xe7\x95\x8c"[..])))] // 你好=世界
    fn split_once(
        #[case] input: &'static [u8],
        #[case] splitter: u8,
        #[case] expected: Option<(&'static [u8], &'static [u8])>,
    ) {
        assert_eq!(
            Bytes::from_static(input).split_once(splitter),
            expected.map(|(a, b)| (Bytes::from(a), Bytes::from(b)))
        );
    }

    #[rstest]
    #[case::both(b"[test]", Some(&b"test"[..]))]
    #[case::none(b"test", None)]
    #[case::open(b"[test", None)]
    #[case::close(b"test]", None)]
    #[case::empty(b"[]", Some(&b""[..]))]
    #[case::empty_none(b"", None)]
    fn strip_brackets(#[case] input: &'static [u8], #[case] expected: Option<&[u8]>) {
        assert_eq!(
            Bytes::from_static(input).strip_brackets().as_deref(),
            expected
        );
    }

    #[cfg(feature = "parse")]
    #[rstest]
    #[case::both(b"<test>", Some(&b"test"[..]))]
    #[case::none(b"test", None)]
    #[case::open(b"<test", None)]
    #[case::close(b"test>", None)]
    #[case::empty(b"<>", Some(&b""[..]))]
    #[case::empty_none(b"", None)]
    fn strip_angled(#[case] input: &'static [u8], #[case] expected: Option<&[u8]>) {
        assert_eq!(
            Bytes::from_static(input).strip_angled().as_deref(),
            expected
        );
    }

    #[cfg(feature = "parse")]
    #[rstest]
    #[case::prefix(b"prefix", b"pre", Some(b"fix".as_slice()))]
    #[case::case_insensitive(b"PrEfIx", b"pre", Some(b"fIx".as_slice()))]
    #[case::not_found(b"prefix", b"foo", None)]
    #[case::empty_prefix(b"prefix", b"", Some(b"prefix".as_slice()))]
    #[case::empty_input(b"", b"prefix", None)]
    #[case::empty_both(b"", b"", Some(b"".as_slice()))]
    #[case::longer_prefix(b"prefix", b"prefixes", None)]
    fn test_strip_prefix_ci(
        #[case] input: &'static [u8],
        #[case] prefix: &'static [u8],
        #[case] expected: Option<&'static [u8]>,
    ) {
        assert_eq!(
            Bytes::from(input).strip_prefix_ci(prefix),
            expected.map(Bytes::from)
        );
    }
}
