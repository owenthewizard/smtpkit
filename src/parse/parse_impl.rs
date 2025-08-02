#![cfg(feature = "parse")]

use core::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use super::*;

impl TryFrom<Bytes> for Command {
    type Error = Error;

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip_all))]
    fn try_from(input: Bytes) -> Result<Self> {
        let _span = log::info_span!("Command").entered();

        let mut tokens = Tokens::new(input, b' ');
        let token = tokens.next().ok_or(Error::Empty)?;
        log::debug!(token = ?token.as_bstr());

        match token {
            helo if helo.eq_ignore_ascii_case(b"HELO") => rfc5321::helo(tokens),
            ehlo if ehlo.eq_ignore_ascii_case(b"EHLO") => rfc5321::ehlo(tokens),
            mail if mail.eq_ignore_ascii_case(b"MAIL") => rfc5321::mail(tokens),
            rcpt if rcpt.eq_ignore_ascii_case(b"RCPT") => rfc5321::rcpt(tokens),
            data if data.eq_ignore_ascii_case(b"DATA") => rfc5321::data(tokens),
            rset if rset.eq_ignore_ascii_case(b"RSET") => rfc5321::rset(tokens),
            vrfy if vrfy.eq_ignore_ascii_case(b"VRFY") => rfc5321::vrfy(tokens),
            expn if expn.eq_ignore_ascii_case(b"EXPN") => rfc5321::expn(tokens),
            help if help.eq_ignore_ascii_case(b"HELP") => rfc5321::help(tokens),
            noop if noop.eq_ignore_ascii_case(b"NOOP") => rfc5321::noop(tokens),
            quit if quit.eq_ignore_ascii_case(b"QUIT") => rfc5321::quit(tokens),
            bdat if bdat.eq_ignore_ascii_case(b"BDAT") => rfc5321::bdat(tokens),
            _x => {
                log::error!(command = ?_x.as_bstr(), "Not implemented");
                Err(Error::CommandNotImplemented)
            }
        }
    }
}

impl TryFrom<Bytes> for Host {
    type Error = Error;

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip_all))]
    fn try_from(input: Bytes) -> Result<Self> {
        let _span = log::info_span!("Host").entered();
        log::debug!(input = ?input.as_bstr());
        if let Some(bracketed) = input.strip_brackets() {
            log::debug!("input is bracketed");
            if let Ok(ipv4) = Ipv4Addr::parse_ascii(&bracketed) {
                log::debug!("input is an IPv4 address");
                Ok(Self::Ip(IpAddr::V4(ipv4)))
            } else if let Some((tag, content)) = bracketed.split_once_str(b":") {
                log::debug!(
                    tag = ?tag.as_bstr(),
                    content = ?content.as_bstr(),
                    "input is an address literal"
                );
                if tag == b"IPv6" {
                    log::debug!("input is an IPv6 address");
                    Ok(Self::Ip(IpAddr::V6(
                        Ipv6Addr::parse_ascii(content).map_err(|_| Error::InvalidSyntax)?,
                    )))
                } else {
                    log::debug!("empty tag");
                    if tag.is_empty() {
                        return Err(Error::InvalidSyntax);
                    }

                    unsafe {
                        // SAFETY: We've confirmed `input` is bracketed and contains at least one
                        // colon.
                        Ok(Self::Address(Address::new_unchecked(input)))
                    }
                }
            } else {
                log::debug!("input is bracketed, but not an address literal or IP address");
                Err(Error::InvalidSyntax)
            }
        } else {
            log::debug!("input is not bracketed, so must be a domain");
            Domain::try_from(input).map(Self::Domain)
        }
    }
}

impl TryFrom<Bytes> for Email {
    type Error = Error;

    fn try_from(input: Bytes) -> Result<Self> {
        let _span = log::info_span!("Email").entered();
        log::debug!(input = ?input.as_bstr());
        let (local, host) = input.rsplit_once_str(b"@").ok_or(Error::InvalidSyntax)?;

        log::debug!(is_local_part = is_local_part(local), "{}", local.as_bstr());
        log::debug!(is_domain = is_domain(host), "{}", host.as_bstr());

        if local.len() <= max::LOCAL_PART
            && is_local_part(local)
            && host.len() <= max::DOMAIN
            && is_domain(host)
            && input.len() <= max::EMAIL
        {
            // SAFETY: `is_local_part`, `is_domain`, and `rsplit_once_str(b"@")` ensure the input
            // is valid.
            return unsafe { Ok(Self::new_unchecked(input)) };
        }

        Err(Error::InvalidSyntax)
    }
}

impl TryFrom<Bytes> for Domain {
    type Error = Error;

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip_all))]
    fn try_from(input: Bytes) -> Result<Self> {
        let _span = log::info_span!("Domain").entered();
        log::debug!(input = ?input.as_bstr());
        let (a, b) = input
            .split_once(b'.')
            .unwrap_or_else(|| (input.clone(), Bytes::new()));

        log::debug!(is_subdomain = is_subdomain(&a), "{}", a.as_bstr());
        if !is_subdomain(a.as_ref()) {
            return Err(Error::InvalidSyntax);
        }

        log::debug!(is_empty = b.is_empty(), "{}", b.as_bstr());
        if b.is_empty() {
            // SAFETY: `is_subdomain` ensures the input is valid.
            return unsafe { Ok(Self::new_unchecked(a)) };
        }

        b.split(|&x| x == b'.')
            .inspect(|_x| log::debug!(is_subdomain = is_subdomain(_x), "{}", _x.as_bstr()))
            .all(is_subdomain)
            // SAFETY: `is_subdomain` ensures the input is valid.
            .then_some(unsafe { Self::new_unchecked(input) })
            .ok_or(Error::InvalidSyntax)
    }
}

impl TryFrom<Bytes> for XText {
    type Error = Error;

    #[cfg_attr(feature = "tracing", tracing::instrument(level = "debug", skip_all))]
    fn try_from(input: Bytes) -> Result<Self> {
        let _span = log::info_span!("XText").entered();
        log::debug!(input = ?input.as_bstr());
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
