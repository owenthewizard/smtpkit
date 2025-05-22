#![cfg(feature = "parse")]

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::*;

pub trait Parse<T = Bytes>: Sized {
    fn parse(input: T) -> Result<Self>;
}

impl<T: Into<Bytes>> Parse<T> for Command {
    fn parse(input: T) -> Result<Self> {
        let input = input.into();
        let mut tokens = Tokens::new(input, b' ');

        match tokens.next() {
            Some(helo) if helo.eq_ignore_ascii_case(b"HELO") => rfc5321::helo(tokens),
            Some(ehlo) if ehlo.eq_ignore_ascii_case(b"EHLO") => rfc5321::ehlo(tokens),
            Some(mail) if mail.eq_ignore_ascii_case(b"MAIL") => rfc5321::mail(tokens),
            Some(rcpt) if rcpt.eq_ignore_ascii_case(b"RCPT") => rfc5321::rcpt(tokens),
            Some(data) if data.eq_ignore_ascii_case(b"DATA") => rfc5321::data(tokens),
            Some(rset) if rset.eq_ignore_ascii_case(b"RSET") => rfc5321::rset(tokens),
            Some(vrfy) if vrfy.eq_ignore_ascii_case(b"VRFY") => rfc5321::vrfy(tokens),
            Some(expn) if expn.eq_ignore_ascii_case(b"EXPN") => rfc5321::expn(tokens),
            Some(help) if help.eq_ignore_ascii_case(b"HELP") => rfc5321::help(tokens),
            Some(noop) if noop.eq_ignore_ascii_case(b"NOOP") => rfc5321::noop(tokens),
            Some(quit) if quit.eq_ignore_ascii_case(b"QUIT") => rfc5321::quit(tokens),
            x => todo!("{x:?}"),
        }
    }
}

impl Parse for Host {
    fn parse(input: Bytes) -> Result<Self> {
        if let Some(bracketed) = input.strip_brackets() {
            if let Ok(ipv4) = Ipv4Addr::parse_ascii(&bracketed) {
                Ok(Self::Ip(IpAddr::V4(ipv4)))
            } else if let Some((tag, content)) = bracketed.split_once_str(b":") {
                if tag == b"IPv6"
                    && let Ok(ipv6) = Ipv6Addr::parse_ascii(content)
                {
                    Ok(Self::Ip(IpAddr::V6(ipv6)))
                } else {
                    unsafe {
                        // SAFETY: We've confirmed `input` is bracketed and contains at least one
                        // colon.
                        Ok(Self::Address(Address::new_unchecked(input)))
                    }
                }
            } else {
                Err(Error::InvalidSyntax)
            }
        } else {
            Domain::parse(input).map(Self::Domain)
        }
    }
}

impl Parse for Email {
    fn parse(input: Bytes) -> Result<Self> {
        let (local, host) = input.rsplit_once_str(b"@").ok_or(Error::InvalidSyntax)?;

        if is_local_part(local) && is_domain(host) {
            // SAFETY: `is_local_part`, `is_domain`, and `rsplit_once_str(b"@")` ensure the input
            // is valid.
            return unsafe { Ok(Self::new_unchecked(input)) };
        }

        Err(Error::InvalidSyntax)
    }
}

impl Parse for Domain {
    fn parse(input: Bytes) -> Result<Self> {
        let (a, b) = input
            .split_once(b'.')
            .unwrap_or_else(|| (input.clone(), Bytes::new()));

        if !is_subdomain(a.as_ref()) {
            return Err(Error::InvalidSyntax);
        }

        if b.is_empty() {
            // SAFETY: `is_subdomain` ensures the input is valid.
            return unsafe { Ok(Self::new_unchecked(a)) };
        }

        b.split(|&x| x == b'.')
            .all(is_subdomain)
            // SAFETY: `is_subdomain` ensures the input is valid.
            .then_some(unsafe { Self::new_unchecked(input) })
            .ok_or(Error::InvalidSyntax)
    }
}

impl Parse for XText {
    fn parse(input: Bytes) -> Result<Self> {
        let mut i = 0;
        while i < input.len() {
            if i + 2 < input.len() && input[i] == b'+' {
                if !(input[i + 1].is_ascii_hexdigit() && input[i + 2].is_ascii_hexdigit()) {
                    return Err(Error::InvalidSyntax);
                }
                i += 3;
            } else if is_xchar(input[i]) {
                i += 1;
            } else {
                return Err(Error::InvalidSyntax);
            }
        }

        // SAFETY: `is_xchar` and `is_ascii_hexdigit` ensure the input is valid.
        unsafe { Ok(Self::new_unchecked(input)) }
    }
}
