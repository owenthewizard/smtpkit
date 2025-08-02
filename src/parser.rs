#![cfg(feature = "parse")]

use bstr::Finder;

use crate::*;

/// # `Parser` Internal State
#[derive(Debug)]
enum State {
    Command,
    Data,
    Bdat(Bdat),
}

/// # Parser State Machine
///
/// This parser can be used as-is, or serve as an example of using the lower level parsing functions.
#[derive(Debug)]
pub struct Parser {
    state: State,
    max: usize,
    crlf_finder: Finder<'static>,
    data_finder: Finder<'static>,
}

impl Default for Parser {
    /// Create a `Parser` with a default `max` of 25 MiB.
    fn default() -> Self {
        Self::new(1024 * 1024 * 25)
    }
}

impl Parser {
    /// Create a `Parser` with a custom `max`.
    #[must_use]
    pub fn new(max: usize) -> Self {
        Self {
            state: State::Command,
            max,
            crlf_finder: Finder::new(b"\r\n"),
            data_finder: Finder::new(b"\r\n.\r\n"),
        }
    }

    /// Read and parse bytes from the buffer.
    ///
    /// - Returns `Ok(Some(Command))` if a command was parsed.
    /// - Returns `Ok(None)` if more bytes are needed.
    /// - Returns `Err(Error::TooLong)` if the buffer exceeds `max` bytes.
    pub fn parse(&mut self, buf: &mut BytesMut) -> Result<Option<Command>, Error> {
        let _span = log::debug_span!("parser").entered();
        loop {
            let _span = log::trace_span!("loop").entered();
            log::trace!(buf_len = buf.len());

            if buf.len() > self.max {
                log::debug!(
                    buf_len = buf.len(),
                    max = self.max,
                    "Buffer too long; clearing"
                );
                buf.clear();
                self.state = State::Command;
                return Err(Error::TooLong);
            }

            let _span = log::debug_span!("state").entered();
            match self.state {
                State::Command => {
                    let _span = log::debug_span!("Command").entered();

                    let Some(pos) = self.crlf_finder.find(&buf) else {
                        log::debug!("No CRLF found, need more bytes");
                        return Ok(None);
                    };

                    if pos > max::COMMAND_LINE {
                        log::debug!(
                            len = pos,
                            max = max::COMMAND_LINE,
                            "Command line too long; advancing"
                        );
                        buf.advance(pos);
                        return Err(Error::TooLong);
                    }

                    let command = buf.split_to(pos);
                    // consume CRLF
                    buf.advance(2);

                    match Command::try_from(command.freeze())? {
                        Command::Data(payload) => {
                            log::debug!("Parsed DATA");

                            debug_assert!(
                                payload.is_empty(),
                                "DATA command payload should not have been read yet"
                            );

                            self.state = State::Data;
                        }

                        Command::Bdat(bdat) => {
                            log::debug!(chunk_len = bdat.size, last = bdat.last, "Parsed BDAT");

                            debug_assert!(
                                bdat.payload.is_empty(),
                                "BDAT command payload should not have been read yet"
                            );

                            self.state = State::Bdat(bdat);
                        }

                        command => {
                            log::debug!(command = ?command, "Parsed");
                            return Ok(Some(command));
                        }
                    }
                }

                State::Data => {
                    let _span = log::debug_span!("Data").entered();

                    let Some(pos) = self.data_finder.find(&buf) else {
                        log::debug!("No CRLF.CRLF found, need more bytes");
                        return Ok(None);
                    };

                    let payload = buf.split_to(pos);
                    // consume \r\n.\r\n
                    buf.advance(5);

                    let mut lines = Lines::new(payload.freeze());
                    #[expect(clippy::unused_enumerate_index, reason = "tracing stub")]
                    for (_i, line) in lines.by_ref().enumerate() {
                        if line.len() > max::DATA_LINE {
                            log::debug!(
                                line = _i,
                                len = line.len(),
                                max = max::DATA_LINE,
                                "DATA line too long"
                            );
                            self.state = State::Command;
                            return Err(Error::TooLong);
                        }
                    }
                    let payload = lines.into_bytes();

                    self.state = State::Command;
                    let command = Command::Data(payload);
                    log::debug!(command = ?command, "Parsed");
                    return Ok(Some(command));
                }

                State::Bdat(ref bdat) => {
                    let _span = log::debug_span!("Bdat").entered();

                    debug_assert!(
                        bdat.payload.is_empty(),
                        "BDAT command payload should not have been read yet"
                    );

                    if bdat.size > self.max {
                        log::debug!(
                            len = bdat.size,
                            max = self.max,
                            "BDAT size exceeds max, skipping"
                        );
                        buf.advance(bdat.size);
                        self.state = State::Command;
                        return Err(Error::TooLong);
                    }

                    if buf.len() < bdat.size {
                        log::debug!(
                            buf_len = buf.len(),
                            bdat_size = bdat.size,
                            "Need more bytes for BDAT"
                        );
                        return Ok(None);
                    }

                    let payload = buf.split_to(bdat.size).freeze();
                    let bdat = Command::Bdat(Bdat {
                        size: bdat.size,
                        last: bdat.last,
                        payload,
                    });

                    self.state = State::Command;
                    log::debug!(command = ?bdat, "Parsed");
                    return Ok(Some(bdat));
                }
            }
        }
    }
}
