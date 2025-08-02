use super::*;
use crate::rcpt::*;

type RcptResult = Result<Parameter>;

impl TryFrom<Bytes> for Parameter {
    type Error = Error;

    fn try_from(mut input: Bytes) -> RcptResult {
        let (key, value) = if let Some(pos) = input.find_byte(b'=') {
            let k = input.split_to(pos);
            input.advance(1); // the `=`
            (k, Some(input))
        } else {
            (input, None)
        };

        match (key, value) {
            (notify, Some(never)) if notify.eq_ignore_ascii_case(b"NOTIFY") => {
                Notify::try_from(never).map(Parameter::Notify)
            }

            (orcpt, Some(x)) if orcpt.eq_ignore_ascii_case(b"ORCPT") => {
                Email::try_from(x).map(Parameter::ORcpt)
            }
            _ => Err(Error::InvalidParameter),
        }
    }
}

impl TryFrom<Bytes> for Notify {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        if input.eq_ignore_ascii_case(b"NEVER") {
            return Ok(Self::NEVER);
        }

        let mut flags = Self::empty();
        for token in Tokens::new(input, b',').map(|t| match t {
            delay if delay.eq_ignore_ascii_case(b"DELAY") => Ok(Self::DELAY),
            failure if failure.eq_ignore_ascii_case(b"FAILURE") => Ok(Self::FAILURE),
            success if success.eq_ignore_ascii_case(b"SUCCESS") => Ok(Self::SUCCESS),
            _ => Err(Error::InvalidSyntax),
        }) {
            flags |= token?;
        }

        Ok(flags)
    }
}

impl Parameters<Result<Parameter>> for Rcpt {
    fn parameters(&mut self, parameters: impl Iterator<Item = Result<Parameter>>) -> Result<()> {
        for parameter in parameters {
            match parameter? {
                Parameter::ORcpt(email) => self.orcpt = Some(email),
                Parameter::Notify(notify) => self.notify = Some(notify),
            }
        }

        Ok(())
    }
}
