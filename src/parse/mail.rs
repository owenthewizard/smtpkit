use super::*;

type MailResult = Result<MailParam>;

impl Parse for MailParam {
    fn parse(mut input: Bytes) -> MailResult {
        let (key, value) = if let Some(pos) = input.iter().position(|&b| b == b'=') {
            let k = input.split_to(pos);
            input.advance(1); // the `=`
            (k, Some(input))
        } else {
            (input, None)
        };

        match (key, value) {
            (size, Some(n)) if size.eq_ignore_ascii_case(b"SIZE") => usize::from_ascii(&n)
                .map_err(|_| Error::InvalidSyntax)
                .map(Self::Size),

            (ret, Some(x)) if ret.eq_ignore_ascii_case(b"RET") => Ret::parse(x).map(Self::Ret),

            (envid, Some(x)) if envid.eq_ignore_ascii_case(b"ENVID") => {
                EnvId::parse(x).map(Self::EnvId)
            }

            (auth, Some(x)) if auth.eq_ignore_ascii_case(b"AUTH") => Auth::parse(x).map(Self::Auth),

            /*
            (smtputf8, None) if smtputf8.eq_ignore_ascii_case(b"SMTPUTF8") => {
                Ok(MailParam::SmtpUtf8)
            }

            (mtp, Some(x)) if mtp.eq_ignore_ascii_case(b"MT-PRIORITY") => {
                Ok(MailParam::MtPriority(MtPriority::parse(x)?))
            }

            (deliverby, Some(x)) if deliverby.eq_ignore_ascii_case(b"DELIVERBY") => {
                Ok(MailParam::DeliverBy(DeliverBy::parse(x)?))
            }

            (rrvs, Some(x)) if rrvs.eq_ignore_ascii_case(b"RRVS") => {
                Ok(MailParam::Rrvs(Rrvs::parse(x)?))
            }

            (burl, Some(x)) if burl.eq_ignore_ascii_case(b"BURL") => {
                Ok(MailParam::Burl(Burl::parse(x)?))
            }
            */
            _ => Err(Error::InvalidParameter),
        }
    }
}

impl Parse for Ret {
    fn parse(input: Bytes) -> Result<Self> {
        match input {
            full if full.eq_ignore_ascii_case(b"FULL") => Ok(Self::Full),
            headers if headers.eq_ignore_ascii_case(b"HDRS") => Ok(Self::Headers),
            _ => Err(Error::InvalidSyntax),
        }
    }
}

impl Parse for EnvId {
    fn parse(input: Bytes) -> Result<Self> {
        XText::parse(input).map(Self)
    }
}

impl Parse for Auth {
    fn parse(input: Bytes) -> Result<Self> {
        if input.as_ref() == b"<>" {
            return Ok(Self::Anonymous);
        }

        XText::parse(input).map(Self::Identity)
    }
}
