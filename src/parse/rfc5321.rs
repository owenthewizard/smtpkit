extern crate alloc;
use alloc::vec::Vec;

use super::*;

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
    let from = tokens.next().ok_or(Error::MissingParameter)?;
    let rp = from
        .strip_prefix_ci(b"FROM:")
        .as_ref()
        .and_then(Helpers::strip_angled)
        .ok_or(Error::InvalidSyntax)?;

    let reverse_path = if rp == b"<>".as_slice() {
        ReversePath::Null
    } else {
        ReversePath::Email(Email::parse(rp)?)
    };

    let parameters = tokens.map(MailParam::parse).collect::<Result<Vec<_>>>()?;

    Ok(Command::Mail {
        reverse_path,
        parameters,
    })
}

pub(super) fn rcpt(mut tokens: Tokens) -> CommandResult {
    let to = tokens.next().ok_or(Error::MissingParameter)?;

    let forward_path = Email::parse(to)?;

    let args = rcpt_args(tokens)?;

    Ok(Command::Rcpt {
        forward_path,
        parameters: args,
    })
}

#[allow(unused_variables, unused_mut, reason = "TODO")]
fn rcpt_args(mut tokens: Tokens) -> Result<Vec<RcptParam>> {
    todo!();
}

pub(super) fn data(mut tokens: Tokens) -> CommandResult {
    tokens
        .next()
        .is_none()
        .then_some(Command::Data)
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
