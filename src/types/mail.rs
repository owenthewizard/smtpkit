use derive_more::Display;

use crate::*;

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
#[non_exhaustive]
pub enum MailParam {
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

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub struct EnvId(pub XText);

#[derive(Debug, Display, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Ret {
    #[default]
    #[display("HDRS")]
    Headers,
    #[display("FULL")]
    Full,
}

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum Auth {
    #[display("<>")]
    Anonymous,
    Identity(XText),
}

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

#[derive(Debug, Display, PartialEq, Eq, Clone, Hash)]
pub enum ReversePath {
    #[display("<>")]
    Null,
    #[display("<{_0}>")]
    Email(Email),
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::size(MailParam::Size(1024), "SIZE=1024")]
    #[case::ret_headers(MailParam::Ret(Ret::Headers), "RET=HDRS")]
    #[case::ret_full(MailParam::Ret(Ret::Full), "RET=FULL")]
    #[case::env_id(MailParam::EnvId(EnvId(unsafe { XText::new_unchecked("12345".into()) })), "ENVID=12345")]
    #[case::auth_anonymous(MailParam::Auth(Auth::Anonymous), "AUTH=<>")]
    #[case::auth_identity(MailParam::Auth(Auth::Identity(unsafe { XText::new_unchecked("<bob@example.com>".into()) })), "AUTH=<bob@example.com>")]
    #[case::body_7bit(MailParam::Body(Body::SevenBit), "BODY=7BIT")]
    #[case::body_8bit_mime(MailParam::Body(Body::EightBitMime), "BODY=8BITMIME")]
    #[case::body_binary_mime(MailParam::Body(Body::BinaryMime), "BODY=BINARYMIME")]
    fn mail_param_display(#[case] param: MailParam, #[case] expected: &str) {
        assert_eq!(&param.to_string(), expected);
    }
}
