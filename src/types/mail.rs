use derive_more::Display;

use crate::*;

/// `MAIL` Command Parameters
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Mail {
    /// `SIZE`
    pub size: Option<usize>,
    /// `RET`
    pub ret: Option<mail::Ret>,
    /// `ENVID`
    pub envid: Option<EnvId>,
    /// `AUTH`
    pub auth: Option<mail::Auth>,
    /// `BODY`
    pub body: Option<mail::Body>,
    /// `FROM:`
    pub from: ReversePath,
}

/// # `MAIL` Command Parameter
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum Parameter {
    #[display("SIZE={_0}")]
    Size(usize),
    #[display("RET={_0}")]
    Ret(Ret),
    #[display("ENVID={_0}")]
    EnvId(EnvId),
    #[display("AUTH={_0}")]
    Auth(Auth),
    #[display("BODY={_0}")]
    Body(Body),
}

/// Envelope ID
///
/// A unique identifier that can be used to track the message as it moves through the mail system.
///
/// <https://datatracker.ietf.org/doc/html/rfc3885>
#[derive(derive_more::Debug, Display, PartialEq, Eq, Clone, Hash)]
#[debug("{_0:?}")]
pub struct EnvId(pub XText);

/// # Return
///
/// Whether or not the message should be included in any failed DSN issued for this message
/// transaction.
///
/// <https://datatracker.ietf.org/doc/html/rfc1891>
#[derive(Debug, Display, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Ret {
    /// Request that only the headers of the message be returned.
    #[default]
    #[display("HDRS")]
    Headers,
    /// Request that the entire message be returned.
    #[display("FULL")]
    Full,
}

/// # Authentication Identity
///
/// The authentication identity associated with an individual message.
///
/// <https://datatracker.ietf.org/doc/html/rfc4954#section-5>
#[derive(derive_more::Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum Auth {
    #[display("<>")]
    Anonymous,
    #[debug("{_0:?}")]
    Identity(XText),
}

/// # Body
///
/// The body type of the message.
///
/// <https://datatracker.ietf.org/doc/html/rfc1652>
/// <https://datatracker.ietf.org/doc/html/rfc3030>
#[derive(Debug, Display, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Body {
    #[default]
    #[display("7BIT")]
    SevenBit,
    #[display("8BITMIME")]
    EightBitMime,
    #[display("BINARYMIME")]
    BinaryMime,
}

/// # Reverse Path
///
/// The reverse path (from address) of the message.
///
/// <https://datatracker.ietf.org/doc/html/rfc5321#section-3.3>
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum ReversePath {
    /// The reverse path is null (`<>`).
    #[display("<>")]
    Null,
    /// The reverse path is a valid email address.
    #[display("<{_0}>")]
    Email(Email),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::size(Parameter::Size(1024), "SIZE=1024")]
    #[case::ret_headers(Parameter::Ret(Ret::Headers), "RET=HDRS")]
    #[case::ret_full(Parameter::Ret(Ret::Full), "RET=FULL")]
    #[case::env_id(Parameter::EnvId(unsafe { EnvId(XText::new_unchecked("12345".into())) }), "ENVID=12345")]
    #[case::auth_anonymous(Parameter::Auth(Auth::Anonymous), "AUTH=<>")]
    #[case::auth_identity(Parameter::Auth(Auth::Identity(unsafe { XText::new_unchecked("<bob@example.com>".into()) })), "AUTH=<bob@example.com>")]
    #[case::body_7bit(Parameter::Body(Body::SevenBit), "BODY=7BIT")]
    #[case::body_8bit_mime(Parameter::Body(Body::EightBitMime), "BODY=8BITMIME")]
    #[case::body_binary_mime(Parameter::Body(Body::BinaryMime), "BODY=BINARYMIME")]
    fn mail_param_display(#[case] param: Parameter, #[case] expected: &str) {
        assert_eq!(&param.to_string(), expected);
    }
}
