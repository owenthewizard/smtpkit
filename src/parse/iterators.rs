use core::iter::FusedIterator;

use bstr::{ByteSlice, Finder};
use bytes::{Buf, Bytes};

use super::Error;

#[derive(Debug, Clone)]
pub struct Tokens {
    bytes: Bytes,
    delim: u8,
    //len: usize,
}

impl Tokens {
    /// Create a new `Tokens` iterator.
    pub fn new(bytes: Bytes, delim: u8) -> Self {
        //let len = bytes.len();
        Self { bytes, delim }
    }

    /*
    /// Consume the `Tokens` and return the remaining `Bytes`.
    pub fn remainder(self) -> Bytes {
        self.bytes
    }

    /// The original length of the `Bytes` when this `Tokens` was created.
    pub const fn len(&self) -> usize {
        self.len
    }
    */
}

impl Iterator for Tokens {
    type Item = Bytes;

    /// Return the next token.
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        let pos = self
            .bytes
            .as_ref()
            .find_byte(self.delim)
            .unwrap_or(self.bytes.len());
        let token = self.bytes.split_to(pos);

        if !self.bytes.is_empty() {
            self.bytes.advance(1);
        }

        Some(token)
    }

    #[cfg_attr(test, mutants::skip)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.bytes.is_empty() {
            return (0, Some(0));
        }

        #[expect(clippy::naive_bytecount, reason = "small input")]
        let remaining_delims = self.bytes.iter().filter(|&&b| b == self.delim).count();
        //let remaining_delims = self.bytes.as_ref().find_iter(&[self.delim]).count();

        (1, Some(remaining_delims + 1))
    }
}

impl FusedIterator for Tokens {}

pub trait Parameters<T> {
    fn parameters(&mut self, parameters: impl Iterator<Item = T>) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
pub struct Lines {
    bytes: Bytes,
    next_index: usize,
    finder: Finder<'static>,
}

impl Lines {
    /// Create a new `Lines` iterator.
    pub fn new(bytes: Bytes) -> Self {
        Self {
            bytes,
            next_index: 0,
            finder: Finder::new(b"\r\n"),
        }
    }

    /// Consume the `Lines` and return the remaining `Bytes`.
    pub fn into_bytes(self) -> Bytes {
        self.bytes
    }
}

impl Iterator for Lines {
    type Item = Bytes;

    /// Return the next line.
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.finder.find(&self.bytes[self.next_index..]) {
            let ret = self.bytes.slice(self.next_index..self.next_index + pos);
            self.next_index += pos + 2;
            return Some(ret);
        }

        None
    }
}

impl FusedIterator for Lines {}
