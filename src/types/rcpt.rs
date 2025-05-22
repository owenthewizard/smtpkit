use core::fmt;

use bitflags::bitflags;

use super::*;

#[derive(Debug, Display, PartialEq, Clone, Hash)]
#[non_exhaustive]
pub enum RcptParam {
    #[display("ORCPT=<{_0}>")]
    ORcpt(Email),
    Notify(Notify),
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct Notify: u8 {
        const NEVER = 0b000;
        const DELAY = 0b001;
        const FAILURE = 0b010;
        const SUCCESS = 0b100;
    }
}

impl Notify {
    pub fn never(&self) -> bool {
        *self == Self::NEVER
    }

    pub fn delay(&self) -> bool {
        self.contains(Self::DELAY)
    }

    pub fn failure(&self) -> bool {
        self.contains(Self::FAILURE)
    }

    pub fn success(&self) -> bool {
        self.contains(Self::SUCCESS)
    }
}

impl fmt::Display for Notify {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "NOTIFY=NEVER");
        }

        let mut first = true;
        for flag in self.iter() {
            if first {
                write!(f, "NOTIFY=")?;
                first = false;
            } else {
                write!(f, ",")?;
            }
            match flag {
                Self::DELAY => write!(f, "DELAY")?,
                Self::FAILURE => write!(f, "FAILURE")?,
                Self::SUCCESS => write!(f, "SUCCESS")?,
                _ => unreachable!(),
            }
        }

        Ok(())
    }
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

    #[rstest]
    #[case::orcpt(RcptParam::ORcpt(unsafe { Email::new_unchecked("alice@example.com".into()) }), "ORCPT=<alice@example.com>")]
    #[case::notify_never(RcptParam::Notify(Notify::NEVER), "NOTIFY=NEVER")]
    #[case::notify_delay(RcptParam::Notify(Notify::DELAY), "NOTIFY=DELAY")]
    #[case::notify_failure(RcptParam::Notify(Notify::FAILURE), "NOTIFY=FAILURE")]
    #[case::notify_success(RcptParam::Notify(Notify::SUCCESS), "NOTIFY=SUCCESS")]
    #[case::notify_delay_failure(RcptParam::Notify(Notify::DELAY | Notify::FAILURE), "NOTIFY=DELAY,FAILURE")]
    #[case::notify_all(RcptParam::Notify(Notify::DELAY | Notify::FAILURE | Notify::SUCCESS), "NOTIFY=DELAY,FAILURE,SUCCESS")]
    fn rcpt_param_display(#[case] param: RcptParam, #[case] expected: &str) {
        assert_eq!(&param.to_string(), expected);
    }
}
