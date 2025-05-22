#![cfg(feature = "codec")]

use bytes::{Buf, BytesMut};
use std::{convert::Into, io};
use tokio_util::codec::{Decoder, Encoder};

use super::Command;
use super::parse;

/// Maximum Line Length for Commands
///
/// Includes SMTP command verb and arguments.
///
/// **Includes** `\r\n`.
///
/// Specified in RFC5321
const COMMAND_MAX: usize = 512;

/// Maximum `DATA` Length
///
/// Does **not** include `\r\n`.
///
/// Specified in RFC5322.
const DATA_MAX: usize = 998; // Per RFC 5322

#[derive(Debug, thiserror::Error)]
pub enum SmtpError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    CommandError(#[from] parse::CommandError),
    #[error("Maximum length exceeded")]
    TooBig,
}

#[derive(Debug)]
pub struct Smtp {
    next_index: usize,

    // Maximum bytes to read as a single frame.
    // In practice, this is the maximum BDAT chunk size, since SMTP commands and DATA lines are
    // already constrained by RFC 5321 and RFC 5322 respectively.
    // However, you could max it smaller if you wanted to.
    max_length: usize,
}

impl Smtp {
    pub const fn new() -> Self {
        Self {
            next_index: 0,
            max_length: 1024 * 1024, // 1 MB
        }
    }

    pub const fn with_max_length(max_length: usize) -> Self {
        Self {
            next_index: 0,
            max_length,
        }
    }
}

impl Default for Smtp {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for Smtp {
    //type Item = Result<Command, SmtpError>;
    //type Error = io::Error;

    type Item = Command;
    type Error = SmtpError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Look for a \r\n line
        if let Some(newline_pos) = src[self.next_index..]
            .windows(2)
            .position(|window| window == b"\r\n")
        {
            let newline_pos = newline_pos + self.next_index;

            /*
            // Check if line exceeds maximum length
            if newline_pos > Self::MAX_LENGTH {
                return Err(SmtpCodecError::CommandError(
                    "Line exceeds maximum length".to_string(),
                ));
            }
            */

            // Extract the line
            let line = &src[..newline_pos];

            // Reset next_index since we've consumed a full line
            self.next_index = 0;

            // Create command with current timestamp
            let ret = Some(parse::parse(line).map_err(Into::into)).transpose();

            // Remove the line and the \r\n from the buffer
            src.advance(newline_pos + 2);

            ret
        } else {
            // No complete line found
            self.next_index = src.len().saturating_sub(1);

            // Check if buffer is getting too large
            if src.len() > self.max_length {
                src.clear(); // Clear the buffer
                return Err(SmtpError::TooBig);
            }

            Ok(None)
        }
    }
}

impl Encoder<String> for Smtp {
    type Error = SmtpError;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Check if the item exceeds the maximum length
        if item.len() > COMMAND_MAX {
            return Err(SmtpError::TooBig);
        }

        // Encode the command with \r\n
        let command = format!("{item}\r\n");

        // Check if the command exceeds the maximum length
        if command.len() > self.max_length {
            return Err(SmtpError::TooBig);
        }

        // Push the command to the destination buffer
        dst.extend_from_slice(command.as_bytes());

        Ok(())
    }
}
