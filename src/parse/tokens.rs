use core::iter::FusedIterator;

use bytes::{Buf, Bytes};

#[derive(Debug, Clone)]
pub struct Tokens {
    bytes: Bytes,
    delim: u8,
}

impl Tokens {
    pub fn new(bytes: Bytes, delim: u8) -> Self {
        Self { bytes, delim }
    }

    pub fn remainder(self) -> Bytes {
        self.bytes
    }
}

impl Iterator for Tokens {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }

        let pos = self
            .bytes
            .iter()
            .position(|&b| b == self.delim)
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

        let remaining_delims = self.bytes.iter().filter(|&&b| b == self.delim).count();

        (1, Some(remaining_delims + 1))
    }
}

impl FusedIterator for Tokens {}
