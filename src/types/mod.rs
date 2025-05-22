extern crate alloc;
use alloc::vec::Vec;

use core::{fmt, net::IpAddr};

use bytes::{BufMut, BytesMut};
use derive_more::{AsRef, Display};

use crate::*;

pub mod mail;
use mail::*;

pub mod rcpt;
use rcpt::*;

#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[as_ref([u8])]
#[display("{}", self.0.as_bstr())]
pub struct Email(Bytes);

impl Email {
    /// Consume the `Email`, returning the inner [`Bytes`].
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq, Clone, Hash)]
pub enum Command {
    Helo(Host),
    Ehlo(Host),
    Mail {
        reverse_path: ReversePath,
        parameters: Vec<MailParam>,
    },
    Rcpt {
        forward_path: Email,
        parameters: Vec<RcptParam>,
    },
    Data,
    Rset,
    Quit,
    Noop,
    StartTls,
    Auth {
        mechanism: Mechanism,
        initial_response: Option<Base64>,
    },
    Expn,
    Help,
    Vrfy,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Helo(host) => write!(f, "HELO {host}"),
            Self::Ehlo(host) => write!(f, "EHLO {host}"),
            Self::Mail {
                reverse_path,
                parameters,
            } => {
                write!(f, "MAIL FROM:")?;
                match reverse_path {
                    ReversePath::Email(email) => write!(f, "<{email}>")?,
                    ReversePath::Null => write!(f, "<>")?,
                }
                for param in parameters {
                    write!(f, " {param}")?;
                }
                Ok(())
            }

            Self::Rcpt {
                forward_path,
                parameters,
            } => {
                write!(f, "RCPT TO:<{forward_path}>")?;
                for param in parameters {
                    write!(f, " {param}")?;
                }
                Ok(())
            }

            Self::Data => write!(f, "DATA"),
            Self::Rset => write!(f, "RSET"),
            Self::Quit => write!(f, "QUIT"),
            Self::Noop => write!(f, "NOOP"),
            Self::StartTls => write!(f, "STARTTLS"),

            Self::Auth {
                mechanism,
                initial_response,
            } => {
                write!(f, "AUTH {mechanism}")?;
                if let Some(initial_response) = initial_response {
                    write!(f, " {initial_response}")?;
                }
                Ok(())
            }

            Self::Expn => write!(f, "EXPN"),
            Self::Help => write!(f, "HELP"),
            Self::Vrfy => write!(f, "VRFY"),
        }
    }
}

#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Base64(Bytes);

impl Base64 {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Host {
    Domain(Domain),
    Ip(IpAddr),
    Address(Address),
}

#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Domain(Bytes);

impl Domain {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for Host {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Domain(d) => write!(f, "{d}"),
            Self::Address(addr) => write!(f, "{addr}"),
            Self::Ip(ip) => match ip {
                IpAddr::V4(ipv4) => write!(f, "[{ipv4}]"),
                IpAddr::V6(ipv6) => write!(f, "[IPv6:{ipv6}]"),
            },
        }
    }
}

#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Address(Bytes);

impl Address {
    pub fn parts(&self) -> (Bytes, Bytes) {
        self.0
            .strip_brackets()
            // the only way to get an `Address` is to use `Parse`, where it will always be bracketed.
            .unwrap()
            .split_once(b':')
            // the only way to get an `Address` is to use `Parse`, where it will always contain a
            // `:`.
            .unwrap()
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

#[derive(Debug, Display, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Mechanism {
    #[default]
    Anonymous,
    #[display("CRAM-MD5")]
    CramMd5,
    #[display("DIGEST-MD5")]
    DigestMd5,
    #[display("GSSAPI")]
    Gssapi,
    #[display("LOGIN")]
    Login,
    #[display("NTLM")]
    Ntlm,
    #[display("OAUTHBEARER")]
    OAuthBearer,
    #[display("PLAIN")]
    Plain,
    #[display("SCRAM-SHA-1")]
    ScramSha1,
    #[display("SCRAM-SHA-256")]
    ScramSha256,
    #[display("XOAUTH2")]
    XOAuth2,
}

#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct XText(Bytes);

impl XText {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn into_bytes(self) -> Bytes {
        self.0
    }

    pub fn decode(&self) -> Bytes {
        let mut ret = BytesMut::with_capacity(self.0.len());

        let mut i = 0;
        while i < self.0.len() {
            if i + 2 < self.0.len() && self.0[i] == b'+' {
                let high = decode_hex(self.0[i + 1]);
                let low = decode_hex(self.0[i + 2]);
                ret.put_u8((high << 4) | low);
                i += 3;
            } else {
                ret.put_u8(self.0[i]);
                i += 1;
            }
        }

        ret.freeze()
    }

    pub fn encode(input: &Bytes) -> Self {
        let mut ret = BytesMut::with_capacity(input.len() * 3);

        for byte in input.clone() {
            if is_xchar(byte) {
                ret.put_u8(byte);
                continue;
            }

            ret.put_u8(b'+');
            ret.put_u8(encode_hex(byte >> 4));
            ret.put_u8(encode_hex(byte & 0x0F));
        }

        Self(ret.freeze())
    }
}

fn encode_hex(byte: u8) -> u8 {
    match byte {
        0..=9 => b'0' + byte,
        10..=15 => b'A' + (byte - 10),
        _ => unreachable!("Invalid nibble"),
    }
}

fn decode_hex(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => unreachable!("Invalid hex character"),
    }
}

#[cfg(test)]
#[expect(non_snake_case)]
mod tests {
    use super::*;
    use bstr::{BStr, ByteSlice};
    use rstest::*;

    #[rstest]
    #[case::both(b"[test]", Some(b"test".as_slice().as_bstr()))]
    #[case::none(b"test", None)]
    #[case::open(b"[test", None)]
    #[case::close(b"test]", None)]
    #[case::empty(b"[]", Some(b"".as_slice().as_bstr()))]
    #[case::empty_none(b"", None)]
    fn test_strip_brackets(#[case] input: &[u8], #[case] expected: Option<&BStr>) {
        assert_eq!(strip_brackets(input).map(ByteSlice::as_bstr), expected);
    }

    #[rstest]
    #[case::both(b"<test>", Some(b"test".as_slice().as_bstr()))]
    #[case::none(b"test", None)]
    #[case::open(b"<test", None)]
    #[case::close(b"test>", None)]
    #[case::empty(b"<>", Some(b"".as_slice().as_bstr()))]
    #[case::empty_none(b"", None)]
    fn test_strip_angled(#[case] input: &[u8], #[case] expected: Option<&BStr>) {
        assert_eq!(strip_angled(input).map(ByteSlice::as_bstr), expected);
    }

    #[test]
    fn test_address_parts() {
        let addr = Address(Bytes::from("[test:1234]"));
        assert_eq!(addr.parts(), (b"test".as_slice(), b"1234".as_slice()));
    }

    #[rstest]
    #[case::hexchars(b"he+40llo+0A+2Bworld+2B", b"he@llo\n+world+".as_bstr())]
    #[case::xchars(b"AbCd,1234,Foo", b"AbCd,1234,Foo".as_bstr())]
    #[case::empty(b"", b"".as_bstr())]
    #[case::unencoded(b"hello", b"hello".as_bstr())]
    #[case::encoded(b"+48+65+6c+6c+6f", b"Hello".as_bstr())]
    #[case::mixed(b"Mixed+20Text", b"Mixed Text".as_bstr())]
    #[case::empty(b"", b"".as_bstr())]
    #[case::FF(b"+FF", b"\xFF".as_bstr())]
    fn xtext_decode(#[case] input: &'static [u8], #[case] expected: &BStr) {
        let encoded = XText(Bytes::from(input));
        assert_eq!(encoded.decode().as_ref().as_bstr(), expected);
    }

    #[rstest]
    #[case::hexchars(b"he@llo\n+world+", b"he@llo+0A+2Bworld+2B".as_bstr())]
    #[case::xchars(b"AbCd,1234,Foo", b"AbCd,1234,Foo".as_bstr())]
    #[case::empty(b"", b"".as_bstr())]
    fn xtext_encode(#[case] input: &'static [u8], #[case] expected: &BStr) {
        let encoded = XText::encode(Bytes::from(input));
        assert_eq!(encoded.as_ref().as_bstr(), expected);
    }

    #[rstest]
    #[case::hexchars(b"he@llo\n+world+".as_bstr())]
    #[case::xchars(b"AbCd,1234,Foo".as_bstr())]
    #[case::empty(b"".as_bstr())]
    fn xtext_roundtrip_encode(#[case] input: &'static [u8]) {
        let hex = XText::encode(Bytes::from(input.as_bytes())).decode();
        assert_eq!(hex.as_ref().as_bstr(), input);
    }

    #[rstest]
    #[case::zero(b'0', 0)]
    #[case::nine(b'9', 9)]
    #[case::a(b'a', 10)]
    #[case::f(b'f', 15)]
    #[case::A(b'A', 10)]
    #[case::F(b'F', 15)]
    fn test_decode_hex(#[case] input: u8, #[case] expected: u8) {
        assert_eq!(decode_hex(input), expected);
    }

    #[rstest]
    #[case::g(b'g')]
    #[case::G(b'G')]
    #[case::slash(b'/')]
    #[case::colon(b':')]
    #[case::space(b' ')]
    #[should_panic]
    fn test_decode_hex_fail(#[case] input: u8) {
        let _ = decode_hex(input);
    }

    #[rstest]
    #[case::zero(0, b'0')]
    #[case::nine(9, b'9')]
    #[case::a(10, b'A')]
    #[case::f(15, b'F')]
    #[case::A(10, b'A')]
    #[case::F(15, b'F')]
    fn test_encode_hex(#[case] input: u8, #[case] expected: u8) {
        assert_eq!(encode_hex(input), expected);
    }

    #[test]
    #[should_panic]
    fn test_encode_hex_fail() {
        let _ = encode_hex(16);
    }

    #[test]
    fn address_parts() {
        let addr = Address(Bytes::from("[test:1234]"));
        assert_eq!(addr.parts(), (b"test".as_slice(), b"1234".as_slice()));
    }

    #[test]
    fn address_display() {
        let address = Address(Bytes::from("[test:1234]"));
        assert_eq!(address.to_string(), "[test:1234]");
    }

    #[rstest]
    #[case::domain(Host::Domain(Domain(Bytes::from("example.com"))), "example.com")]
    #[case::ipv4(Host::Ip("127.0.0.1".parse::<IpAddr>().unwrap()), "[127.0.0.1]")]
    #[case::ipv6(
        Host::Ip("2001:db8::".parse::<IpAddr>().unwrap()),
        "[IPv6:2001:db8::]"
    )]
    #[case::address(Host::Address(Address(Bytes::from("[test:1234]"))), "[test:1234]")]
    fn host_display(#[case] input: Host, #[case] expected: &str) {
        assert_eq!(input.to_string(), expected);
    }

    // TODO add MailParam and RcptParam
    #[rstest]
    #[case::helo(
        Command::Helo(Host::Domain(Domain(Bytes::from("example.com")))),
        "HELO example.com"
    )]
    #[case::ehlo_domain(
        Command::Ehlo(Host::Domain(Domain(Bytes::from("example.com")))),
        "EHLO example.com"
    )]
    #[case::ehlo_ipv4(
        Command::Ehlo(Host::Ip("127.0.0.1".parse::<IpAddr>().unwrap())), "EHLO [127.0.0.1]")]
    #[case::ehlo_ipv6(
        Command::Ehlo(Host::Ip("2001:db8::".parse::<IpAddr>().unwrap())),
        "EHLO [IPv6:2001:db8::]"
    )]
    #[case::ehlo_address(
        Command::Ehlo(Host::Address(Address(Bytes::from("[test:1234]")))),
        "EHLO [test:1234]"
    )]
    #[case::mail_null(
        Command::Mail {
            reverse_path: ReversePath::Null,
            parameters: vec![],
        },
        "MAIL FROM:<>"
    )]
    #[case::mail(
        Command::Mail {
            reverse_path: ReversePath::Email(Email(Bytes::from("bob@example.com"))), parameters: vec![],}, "MAIL FROM:<bob@example.com>")]
    #[case::rcpt(
        Command::Rcpt {
            forward_path: Email(Bytes::from("alice@example.com")), parameters: vec![],}, "RCPT TO:<alice@example.com>")]
    fn command_display(#[case] input: Command, #[case] expected: &str) {
        assert_eq!(input.to_string(), expected);
    }
}
