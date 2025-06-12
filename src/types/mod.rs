use core::net::IpAddr;

use derive_more::{AsRef, Display};

use crate::*;

pub mod mail;
use mail::{Mail, ReversePath};

pub mod rcpt;
use rcpt::Rcpt;

mod serialize;
pub use serialize::*;

/// # [SMTP Commands](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1)
#[non_exhaustive]
#[derive(derive_more::Debug, PartialEq, Clone, Hash)]
pub enum Command {
    /// Identify the client to the server.
    ///
    /// [Deprecated](https://datatracker.ietf.org/doc/html/rfc5321#appendix-F.3) in favor of
    /// `EHLO`.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.1>
    Helo(Host),
    /// Identify the client to the server and request extended SMTP.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.1>
    Ehlo(Host),
    /// Initiate a mail transaction.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.2>
    #[debug("{_0:?}")]
    Mail(Mail),
    /// Identify the recipient of the mail transaction.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.3>
    #[debug("{_0:?}")]
    Rcpt(Rcpt),
    /// Initiate the transfer of the message data.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.4>
    #[debug("Data([u8; {}])", _0.len())]
    Data(Bytes),
    /// Initiate the transfer of binary data.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc3030>
    #[debug("{_0:?}")]
    Bdat(Bdat),
    /// Reset the current mail transaction.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.5>
    Rset,
    /// Verify an email address.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.6>
    Vrfy,
    /// Expand a mailing list.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.7>
    Expn,
    /// Request help from the server.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.8>
    Help,
    /// Do nothing.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.9>
    Noop,
    /// Terminate the session.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.1.10>
    Quit,
    /// Initiate a TLS session.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc3207>
    StartTls,
    /// Authenticate the client to the server.
    ///
    /// <https://datatracker.ietf.org/doc/html/rfc4954>
    Auth {
        mechanism: Mechanism,
        initial_response: Option<Base64>,
    },
}

/// # Binary Data Chunk
///
/// <https://datatracker.ietf.org/doc/html/rfc3030>
#[derive(derive_more::Debug, PartialEq, Eq, Clone, Hash)]
pub struct Bdat {
    /// Expected size of this chunk of data.
    pub size: usize,

    /// Whether this is the last chunk of data.
    pub last: bool,

    /// Binary data payload.
    #[debug(skip)]
    pub payload: Bytes,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Helo(host) => write!(f, "HELO {host}"),
            Self::Ehlo(host) => write!(f, "EHLO {host}"),
            Self::Mail(mail) => {
                write!(f, "MAIL FROM:")?;
                match mail.from {
                    ReversePath::Email(ref email) => write!(f, "<{email}>")?,
                    ReversePath::Null => write!(f, "<>")?,
                }

                if let Some(size) = mail.size {
                    write!(f, " SIZE={size}")?;
                }

                if let Some(ret) = mail.ret {
                    write!(f, " RET={ret}")?;
                }

                if let Some(envid) = &mail.envid {
                    write!(f, " ENVID={envid}")?;
                }

                if let Some(auth) = &mail.auth {
                    write!(f, " AUTH={auth}")?;
                }

                if let Some(body) = mail.body {
                    write!(f, " BODY={body}")?;
                }

                Ok(())
            }

            Self::Rcpt(rcpt) => {
                write!(f, "RCPT TO:<{}>", rcpt.to)?;

                if let Some(notify) = rcpt.notify {
                    write!(f, " NOTIFY={notify}")?;
                }

                if let Some(orcpt) = &rcpt.orcpt {
                    write!(f, " ORCPT=<{orcpt}>")?;
                }

                Ok(())
            }

            Self::Data(payload) => write!(f, "DATA\r\n{}\r\n.\r\n", payload.as_bstr()),
            Self::Bdat(bdat) => {
                write!(f, "BDAT {}", bdat.payload.len())?;
                if bdat.last {
                    write!(f, " LAST")?;
                }
                write!(f, "\r\n{}", bdat.payload.as_bstr())?;

                Ok(())
            }

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

/// Base64-Encoded String
#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Base64(Bytes);

impl Base64 {
    /// Consume the `Base64`, returning the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Get a reference to the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn bytes(&self) -> &Bytes {
        &self.0
    }

    /// Create a new `Base64` from the given `Bytes`.
    ///
    /// # Safety
    ///
    /// The inner `Bytes` must be a valid base64-encoded string.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

/// Domain, IP address, or address literaly identifying an SMTP client to the server.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Host {
    Domain(Domain),
    Ip(IpAddr),
    Address(Address),
}

/// # Domain Name
#[derive(derive_more::Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[debug("{:?}", self.0.as_bstr())]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Domain(Bytes);

impl Domain {
    /// Consume the `Domain`, returning the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Get a reference to the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn bytes(&self) -> &Bytes {
        &self.0
    }

    /// Create a new `Domain` from the given `Bytes`.
    ///
    /// # Safety
    ///
    /// The inner `Bytes` must be a valid domain name.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
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

/// # Address Literal
/// 
/// As defined in [RFC 5321](https://datatracker.ietf.org/doc/html/rfc5321#section-4.1.3). Takes the form of `[tag:content]`.
#[derive(Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[display("{}", self.0.as_bstr())]
#[as_ref([u8])]
pub struct Address(Bytes);

impl Address {
    /// Returns the `tag` and `content` parts of the address literal.
    #[must_use] pub fn parts(&self) -> (Bytes, Bytes) {
        self.0
            .strip_brackets()
            // the only way to get an `Address` is to use `Parse`, where it will always be bracketed.
            .unwrap()
            .split_once(b':')
            // the only way to get an `Address` is to use `Parse`, where it will always contain a
            // `:`.
            .unwrap()
    }

    /// Get a reference to the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn bytes(&self) -> &Bytes {
        &self.0
    }

    /// Consume the `Address`, returning the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Create a new `Address` from the given `Bytes`.
    ///
    /// # Safety
    ///
    /// The inner `Bytes` must be a valid address literal.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

/// # Authentication Mechanisms
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

/// # `XText` String
///
/// As defined in [RFC 3461](https://datatracker.ietf.org/doc/html/rfc3461#section-4).
#[derive(derive_more::Debug, AsRef, Display, PartialEq, Eq, Clone, Hash)]
#[as_ref([u8])]
#[debug("{:?}", self.0.as_bstr())]
#[display("{}", self.0.as_bstr())]
pub struct XText(Bytes);

impl XText {
    /// Get a reference to the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn bytes(&self) -> &Bytes {
        &self.0
    }

    /// Create a new `XText` from the given `Bytes`.
    ///
    /// # Safety
    ///
    /// The inner `Bytes` must be a valid `XText` string.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }

    /// Consume the `XText`, returning the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Decode hexchars in the `XText` string into the provided `BytesMut`.
    pub fn decode_into(&self, buf: &mut BytesMut) {
        let mut i = 0;
        while i < self.0.len() {
            if i + 2 < self.0.len() && self.0[i] == b'+' {
                let high = decode_hex(self.0[i + 1]);
                let low = decode_hex(self.0[i + 2]);
                buf.extend_from_slice(&[(high << 4) | low]);
                i += 3;
            } else {
                buf.extend_from_slice(&[self.0[i]]);
                i += 1;
            }
        }
    }

    /// Return a `BytesMut` containing the decoded bytes of the `XText` string.
    ///
    /// This is a convenience method that allocates a new `BytesMut` and calls `decode_into`.
    #[must_use] pub fn decode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        self.decode_into(&mut buf);
        buf
    }

    /// Encode the input into hexchars where necessary, returning a new `XText` string.
    #[must_use] pub fn encode(input: &Bytes) -> Self {
        let mut ret = BytesMut::with_capacity(input.len() * 3);

        for &byte in input {
            if is_xchar(byte) {
                ret.extend_from_slice(&[byte]);
                continue;
            }

            ret.extend_from_slice(b"+");
            ret.extend_from_slice(&[encode_hex(byte >> 4)]);
            ret.extend_from_slice(&[encode_hex(byte & 0x0F)]);
        }

        Self(ret.freeze())
    }
}

/// # Email Address
///
/// As defined in [RFC 5321](https://datatracker.ietf.org/doc/html/rfc5321).
#[derive(AsRef, derive_more::Debug, Display, PartialEq, Eq, Clone, Hash)]
#[as_ref([u8])]
#[debug("{:?}", self.0.as_bstr())]
#[display("<{}>", self.0.as_bstr())]
pub struct Email(Bytes);

impl Email {
    /// Consume the `Email`, returning the inner `Bytes`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub fn into_bytes(self) -> Bytes {
        self.0
    }

    /// Create a new `Email` from the given `Bytes`.
    ///
    /// # Safety
    ///
    /// The inner `Bytes` must take the form of `<local-part>@<domain>`.
    #[cfg_attr(coverage_nightly, coverage(off))]
    #[must_use] pub const unsafe fn new_unchecked(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

/// Encode a hex value into a hex character.
fn encode_hex(byte: u8) -> u8 {
    match byte {
        0..=9 => b'0' + byte,
        10..=15 => b'A' + (byte - 10),
        _ => unreachable!("Invalid digit"),
    }
}

/// Decode a hex character into a hex value.
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

    #[test]
    fn test_address_parts() {
        let addr = Address(Bytes::from("[test:1234]"));
        assert_eq!(
            addr.parts(),
            (Bytes::from_static(b"test"), Bytes::from_static(b"1234"))
        );
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
        let encoded = XText::encode(&Bytes::from(input));
        assert_eq!(encoded.as_ref().as_bstr(), expected);
    }

    #[rstest]
    #[case::hexchars(b"he@llo\n+world+".as_bstr())]
    #[case::xchars(b"AbCd,1234,Foo".as_bstr())]
    #[case::empty(b"".as_bstr())]
    fn xtext_roundtrip_encode(#[case] input: &'static [u8]) {
        let hex = XText::encode(&Bytes::from(input.as_bytes())).decode();
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
        assert_eq!(
            addr.parts(),
            (Bytes::from_static(b"test"), Bytes::from_static(b"1234"))
        );
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

    // TODO add Parameter and Parameter
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
    /* TODO
    #[case::mail_null(
        Command::Mail {
            from: ReversePath::Null,
        },
        "MAIL FROM:<>"
    )]
    #[case::mail(
        Command::Mail {
            from: ReversePath::Email(Email(Bytes::from("bob@example.com"))), parameters: vec![],}, "MAIL FROM:<bob@example.com>")]
    #[case::rcpt(
        Command::Rcpt {
            forward_path: Email(Bytes::from("alice@example.com")), parameters: vec![],}, "RCPT TO:<alice@example.com>")]
    */
    fn command_display(#[case] input: Command, #[case] expected: &str) {
        assert_eq!(input.to_string(), expected);
    }
}
