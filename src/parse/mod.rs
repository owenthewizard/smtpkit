#![cfg(feature = "parse")]

pub(crate) use bytes::Buf;

pub(crate) use super::*;

mod parse_impl;

mod iterators;
pub(crate) use iterators::*;

mod helpers;
use helpers::*;

mod mail;
mod rcpt;
//mod rfc3461;
mod rfc5321;

type Result<T> = core::result::Result<T, Error>;
type CommandResult = Result<Command>;

#[non_exhaustive]
#[derive(thiserror::Error, Debug, Clone, PartialEq, Hash)]
pub enum Error {
    #[error("Command not recognized")]
    InvalidCommand,

    #[error("Parameter not recognized")]
    InvalidParameter,

    #[error("Command is missing a required parameter")]
    MissingParameter,

    #[error("Command has too many parameters or unexpected trailing data")]
    UnexpectedParameter,

    #[error("Invalid syntax")]
    InvalidSyntax,

    #[error("Empty command")]
    Empty,

    #[error("Line too long")]
    TooLong,

    #[error("Input ended unexpectedly")]
    Eoi,

    #[error("Command not implemented")]
    CommandNotImplemented,

    #[error("Parameter not implemented")]
    ParameterNotImplemented,
}

/*
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::example_com(b"HELO example.com", Ok(Command::Helo(b"example.com".into())))]
    #[case::localhost(b"HELO localhost", Ok(Command::Helo(b"localhost".into())))]
    // 1.1.1.1 is an IP, but it meets the ABNF for domain. We might want to disallow this.
    #[case::one_one_one_one(b"HELO 1.1.1.1", Ok(Command::Helo(b"1.1.1.1".into())))]
    #[case::mail_example_com(b"HELO mail.example.com", Ok(Command::Helo(b"mail.example.com".into())))]
    #[case::_invalid(b"HELO -invalid", Err(Error::InvalidSyntax))]
    #[case::invalid_(b"HELO invalid-.com", Err(Error::InvalidSyntax))]
    #[case::invalid__com(b"HELO invalid..com", Err(Error::InvalidSyntax))]
    #[case::missing(b"HELO", Err(Error::MissingParameter))]
    #[case::unexpected(b"HELO foo bar", Err(Error::UnexpectedParameter))]
    fn test_helo(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case::example_com(b"EHLO example.com", Ok(Command::Ehlo(Host::Domain(b"example.com".into()))))]
    #[case::localhost(b"EHLO localhost", Ok(Command::Ehlo(Host::Domain(b"localhost".into()))))]
    // 1.1.1.1 is an IP, but it meets the ABNF for domain. We might want to disallow this.
    #[case::one_one_one_one(b"EHLO 1.1.1.1", Ok(Command::Ehlo(Host::Domain(b"1.1.1.1".into()))))]
    #[case::ipv4(
        b"EHLO [1.1.1.1]",
        Ok(Command::Ehlo(Host::Ip(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)))))
    )]
    #[case::ipv6(b"EHLO [IPv6:2001:db8::1]", Ok(Command::Ehlo(Host::Ip(IpAddr::V6(Ipv6Addr::parse_ascii(b"2001:db8::1").unwrap())))))]
    #[case::mail_example_com(b"EHLO mail.example.com", Ok(Command::Ehlo(Host::Domain(b"mail.example.com".into()))))]
    #[case::address_literal(b"EHLO [tag:content]", Ok(Command::Ehlo(Host::Address(b"tag".into(), b"content".into()))))]
    #[case::_invalid(b"EHLO -invalid", Err(Error::InvalidSyntax))]
    #[case::invalid_(b"EHLO invalid-.com", Err(Error::InvalidSyntax))]
    #[case::invalid__com(b"EHLO invalid..com", Err(Error::InvalidSyntax))]
    #[case::missing(b"EHLO", Err(Error::MissingParameter))]
    #[case::unexpected(b"EHLO foo bar", Err(Error::UnexpectedParameter))]
    fn test_ehlo(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case::data(b"DATA", Ok(Command::Data))]
    #[case::unexpected(b"DATA foo", Err(Error::UnexpectedParameter))]
    fn test_data(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case::rset(b"RSET", Ok(Command::Rset))]
    #[case::unexpected(b"RSET foo", Err(Error::UnexpectedParameter))]
    fn test_rset(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case::noop(b"NOOP", Ok(Command::Noop))]
    #[case::unexpected(b"NOOP foo", Err(Error::UnexpectedParameter))]
    fn test_noop(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }

    #[rstest]
    #[case::quit(b"QUIT", Ok(Command::Quit))]
    #[case::unexpected(b"QUIT foo", Err(Error::UnexpectedParameter))]
    fn test_quit(#[case] input: &[u8], #[case] expected: Result) {
        assert_eq!(Command::parse(input), expected);
    }
}
*/
