use super::*;

type RcptResult = Result<RcptParam>;

impl Parse for RcptParam {
    fn parse(mut input: Bytes) -> RcptResult {
        let (key, value) = if let Some(pos) = input.iter().position(|&b| b == b'=') {
            let k = input.split_to(pos);
            input.advance(1); // the `=`
            (k, Some(input))
        } else {
            (input, None)
        };

        match (key, value) {
            (notify, Some(never)) if notify.eq_ignore_ascii_case(b"NOTIFY") => {
                Notify::parse(never).map(RcptParam::Notify)
            }

            (orcpt, Some(x)) if orcpt.eq_ignore_ascii_case(b"ORCPT") => {
                Email::parse(x).map(RcptParam::ORcpt)
            }
            _ => Err(Error::InvalidParameter),
        }
    }
}

impl Parse for Notify {
    fn parse(input: Bytes) -> Result<Self> {
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
