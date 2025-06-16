use btoi::{ParseIntegerErrorKind, btou_radix};

use super::*;
use crate::mail::{self, Mail, ReversePath};
use crate::rcpt::{self, Rcpt};

pub(super) fn helo(mut tokens: Tokens) -> CommandResult {
    match (tokens.next(), tokens.next()) {
        (Some(d), None) => Domain::parse(d).map(Host::Domain).map(Command::Helo),
        (Some(_), Some(_)) => Err(Error::UnexpectedParameter),
        (None, _) => Err(Error::MissingParameter),
    }
}

pub(super) fn ehlo(mut tokens: Tokens) -> CommandResult {
    match (tokens.next(), tokens.next()) {
        (Some(d), None) => Host::parse(d).map(Command::Ehlo),
        (Some(_), Some(_)) => Err(Error::UnexpectedParameter),
        (None, _) => Err(Error::MissingParameter),
    }
}

pub(super) fn mail(mut tokens: Tokens) -> CommandResult {
    let token = tokens.next().ok_or(Error::MissingParameter)?;
    let rp = token
        .strip_prefix_ci(b"FROM:")
        .ok_or(Error::InvalidSyntax)?;

    let from = if rp == b"<>"[..] {
        ReversePath::Null
    } else {
        ReversePath::Email(
            rp.strip_angled()
                .ok_or(Error::InvalidSyntax)
                .and_then(Email::parse)?,
        )
    };

    let mut mail = Mail {
        from,
        size: None,
        ret: None,
        envid: None,
        auth: None,
        body: None,
    };

    mail.parameters(tokens.map(mail::Parameter::parse))?;

    Ok(Command::Mail(mail))
}

pub(super) fn rcpt(mut tokens: Tokens) -> CommandResult {
    let token = tokens.next().ok_or(Error::MissingParameter)?;
    let to = token
        .strip_prefix_ci(b"TO:")
        .as_ref()
        .and_then(Helpers::strip_angled)
        .ok_or(Error::InvalidSyntax)
        .and_then(Email::parse)?;

    let mut rcpt = Rcpt {
        to,
        orcpt: None,
        notify: None,
    };

    rcpt.parameters(tokens.map(rcpt::Parameter::parse))?;

    Ok(Command::Rcpt(rcpt))
}

pub(super) fn data(mut tokens: Tokens) -> CommandResult {
    tokens
        .next()
        .is_none()
        // caller should perform further processing to get bytes
        .then_some(Command::Data(Bytes::new()))
        .ok_or(Error::UnexpectedParameter)
}

pub(super) fn rset(mut tokens: Tokens) -> CommandResult {
    tokens
        .next()
        .is_none()
        .then_some(Command::Rset)
        .ok_or(Error::UnexpectedParameter)
}

pub(super) fn quit(mut tokens: Tokens) -> CommandResult {
    tokens
        .next()
        .is_none()
        .then_some(Command::Quit)
        .ok_or(Error::UnexpectedParameter)
}

pub(super) fn noop(mut tokens: Tokens) -> CommandResult {
    tokens
        .next()
        .is_none()
        .then_some(Command::Noop)
        .ok_or(Error::UnexpectedParameter)
}

//#[expect(unused_variables, unused_mut, reason = "TODO")]
pub(super) fn bdat(mut tokens: Tokens) -> CommandResult {
    let size = tokens
        .next()
        .ok_or(Error::MissingParameter)
        .and_then(|token| {
            btou_radix::<usize>(&token, 10).map_err(|e| match e.kind() {
                ParseIntegerErrorKind::Empty | ParseIntegerErrorKind::InvalidDigit => {
                    Error::InvalidSyntax
                }
                ParseIntegerErrorKind::PosOverflow => Error::TooLong,
                ParseIntegerErrorKind::NegOverflow => unreachable!(),
            })
        })?;

    let last = match tokens.next() {
        Some(last) if last.eq_ignore_ascii_case(b"LAST") => true,
        Some(_) => return Err(Error::UnexpectedParameter),
        None => false,
    };

    tokens
        .next()
        .is_none()
        .then_some(Command::Bdat(Bdat {
            size,
            last,
            // caller should perform further processing to get payload
            payload: Bytes::new(),
        }))
        .ok_or(Error::UnexpectedParameter)
}

#[allow(unused_variables, unused_mut, reason = "TODO")]
pub(super) fn vrfy(mut tokens: Tokens) -> CommandResult {
    todo!();
}

#[allow(unused_variables, unused_mut, reason = "TODO")]
pub(super) fn expn(mut tokens: Tokens) -> CommandResult {
    todo!();
}

#[allow(unused_variables, unused_mut, reason = "TODO")]
pub(super) fn help(mut tokens: Tokens) -> CommandResult {
    todo!();
}
