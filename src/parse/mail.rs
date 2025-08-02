use super::*;
use crate::mail::*;

type MailResult = Result<Parameter>;

impl TryFrom<Bytes> for Parameter {
    type Error = Error;

    fn try_from(mut input: Bytes) -> MailResult {
        let (key, value) = if let Some(pos) = input.find_byte(b'=') {
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

            (ret, Some(x)) if ret.eq_ignore_ascii_case(b"RET") => Ret::try_from(x).map(Self::Ret),

            (envid, Some(x)) if envid.eq_ignore_ascii_case(b"ENVID") => {
                EnvId::try_from(x).map(Self::EnvId)
            }

            (auth, Some(x)) if auth.eq_ignore_ascii_case(b"AUTH") => {
                Auth::try_from(x).map(Self::Auth)
            }

            (body, Some(x)) if body.eq_ignore_ascii_case(b"BODY") => {
                Body::try_from(x).map(Self::Body)
            }

            /*
            (smtputf8, None) if smtputf8.eq_ignore_ascii_case(b"SMTPUTF8") => {
                Ok(Parameter::SmtpUtf8)
            }

            (mtp, Some(x)) if mtp.eq_ignore_ascii_case(b"MT-PRIORITY") => {
                Ok(Parameter::MtPriority(MtPriority::try_from(x)?))
            }

            (deliverby, Some(x)) if deliverby.eq_ignore_ascii_case(b"DELIVERBY") => {
                Ok(Parameter::DeliverBy(DeliverBy::try_from(x)?))
            }

            (rrvs, Some(x)) if rrvs.eq_ignore_ascii_case(b"RRVS") => {
                Ok(Parameter::Rrvs(Rrvs::try_from(x)?))
            }

            (burl, Some(x)) if burl.eq_ignore_ascii_case(b"BURL") => {
                Ok(Parameter::Burl(Burl::try_from(x)?))
            }
            */
            _ => Err(Error::InvalidParameter),
        }
    }
}

impl TryFrom<Bytes> for Ret {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        match input {
            full if full.eq_ignore_ascii_case(b"FULL") => Ok(Self::Full),
            headers if headers.eq_ignore_ascii_case(b"HDRS") => Ok(Self::Headers),
            _ => Err(Error::InvalidSyntax),
        }
    }
}

impl TryFrom<Bytes> for EnvId {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        XText::try_from(input).map(Self)
    }
}

impl TryFrom<Bytes> for Auth {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        if input.as_ref() == b"<>" {
            return Ok(Self::Anonymous);
        }

        XText::try_from(input).map(Self::Identity)
    }
}

impl Parameters<Result<Parameter>> for Mail {
    fn parameters(&mut self, parameters: impl Iterator<Item = Result<Parameter>>) -> Result<()> {
        for parameter in parameters {
            match parameter? {
                Parameter::Size(size) => self.size = Some(size),
                Parameter::Ret(ret) => self.ret = Some(ret),
                Parameter::EnvId(envid) => self.envid = Some(envid),
                Parameter::Auth(auth) => self.auth = Some(auth),
                Parameter::Body(body) => self.body = Some(body),
            }
        }

        Ok(())
    }
}

impl TryFrom<Bytes> for Body {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        match input.as_ref() {
            seven_bit if seven_bit.eq_ignore_ascii_case(b"7BIT") => Ok(Self::SevenBit),

            eight_bit if eight_bit.eq_ignore_ascii_case(b"8BITMIME") => Ok(Self::EightBitMime),

            binary if binary.eq_ignore_ascii_case(b"BINARYMIME") => Ok(Self::BinaryMime),

            _ => Err(Error::InvalidSyntax),
        }
    }
}
