use bitflags::bitflags;

use super::*;

/// `RCPT` Command Parameters
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct Rcpt {
    pub orcpt: Option<Email>,
    pub notify: Option<rcpt::Notify>,
    pub to: Email,
}

/// Parameters for the `RCPT` command.
#[derive(Debug, Display, PartialEq, Clone, Hash)]
#[non_exhaustive]
pub enum Parameter {
    #[display("ORCPT=<{_0}>")]
    ORcpt(Email),
    Notify(Notify),
}

bitflags! {
    /// Flags for the `NOTIFY` parameter in the `RCPT` command.
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct Notify: u8 {
        const NEVER = 0b000;
        const DELAY = 0b001;
        const FAILURE = 0b010;
        const SUCCESS = 0b100;
    }
}

impl Notify {
    #[must_use]
    pub fn never(&self) -> bool {
        self.is_empty()
    }

    #[must_use]
    pub fn delay(&self) -> bool {
        self.contains(Self::DELAY)
    }

    #[must_use]
    pub fn failure(&self) -> bool {
        self.contains(Self::FAILURE)
    }

    #[must_use]
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

    // TODO
    /*
    #[rstest]
    #[case::size(Parameter::Size(1024), "SIZE=1024")]
    #[case::ret_headers(Parameter::Ret(Ret::Headers), "RET=HDRS")]
    #[case::ret_full(Parameter::Ret(Ret::Full), "RET=FULL")]
    #[case::env_id(Parameter::EnvId(EnvId(unsafe { XText::new_unchecked("12345".into()) })), "ENVID=12345")]
    #[case::auth_anonymous(Parameter::Auth(Auth::Anonymous), "AUTH=<>")]
    #[case::auth_identity(Parameter::Auth(Auth::Identity(unsafe { XText::new_unchecked("<bob@example.com>".into()) })), "AUTH=<bob@example.com>")]
    #[case::body_7bit(Parameter::Body(Body::SevenBit), "BODY=7BIT")]
    #[case::body_8bit_mime(Parameter::Body(Body::EightBitMime), "BODY=8BITMIME")]
    #[case::body_binary_mime(Parameter::Body(Body::BinaryMime), "BODY=BINARYMIME")]
    fn mail_param_display(#[case] param: Parameter, #[case] expected: &str) {
        assert_eq!(&param.to_string(), expected);
    }

    #[rstest]
    #[case::orcpt(Parameter::ORcpt(unsafe { Email::new_unchecked("alice@example.com".into()) }), "ORCPT=<alice@example.com>")]
    #[case::notify_never(Parameter::Notify(Notify::NEVER), "NOTIFY=NEVER")]
    #[case::notify_delay(Parameter::Notify(Notify::DELAY), "NOTIFY=DELAY")]
    #[case::notify_failure(Parameter::Notify(Notify::FAILURE), "NOTIFY=FAILURE")]
    #[case::notify_success(Parameter::Notify(Notify::SUCCESS), "NOTIFY=SUCCESS")]
    #[case::notify_delay_failure(Parameter::Notify(Notify::DELAY | Notify::FAILURE), "NOTIFY=DELAY,FAILURE")]
    #[case::notify_all(Parameter::Notify(Notify::DELAY | Notify::FAILURE | Notify::SUCCESS), "NOTIFY=DELAY,FAILURE,SUCCESS")]
    fn rcpt_param_display(#[case] param: Parameter, #[case] expected: &str) {
        assert_eq!(&param.to_string(), expected);
    }
    */
}
