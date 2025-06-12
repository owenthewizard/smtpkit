use super::mail::*;
use super::rcpt::*;
use super::*;

pub trait ToBytes {
    /// Write the encoded bytes data into the provided `BytesMut`.
    fn to_bytes_into(&self, buf: &mut BytesMut);

    /// Return a `BytesMut` containing the encoded bytes.
    ///
    /// This is a convenience method that allocates a new `BytesMut` and calls `to_bytes_into`.
    fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        self.to_bytes_into(&mut buf);
        buf
    }
}

impl<T: AsRef<[u8]>> ToBytes for T {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(self.as_ref());
    }
}

impl ToBytes for Bdat {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"BDAT ");

        let mut size = itoa::Buffer::new();
        let size = size.format(self.payload.len());
        buf.extend_from_slice(size.as_bytes());

        if self.last {
            buf.extend_from_slice(b" LAST");
        }

        buf.extend_from_slice(b"\r\n");
        buf.extend_from_slice(&self.payload);
    }
}

impl ToBytes for ReversePath {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"<");
        match self {
            Self::Null => {}
            Self::Email(email) => {
                email.to_bytes_into(buf);
            }
        }
        buf.extend_from_slice(b">");
    }
}

impl ToBytes for Ret {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"RET=");
        match self {
            Self::Full => buf.extend_from_slice(b"FULL"),
            Self::Headers => buf.extend_from_slice(b"HDRS"),
        }
    }
}

impl ToBytes for EnvId {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"ENVID=");
        self.0.to_bytes_into(buf);
    }
}

impl ToBytes for Auth {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"AUTH=");
        match self {
            Self::Anonymous => buf.extend_from_slice(b"<>"),
            Self::Identity(id) => id.to_bytes_into(buf),
        }
    }
}

impl ToBytes for Body {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"BODY=");
        match self {
            Self::SevenBit => buf.extend_from_slice(b"7BIT"),
            Self::EightBitMime => buf.extend_from_slice(b"8BITMIME"),
            Self::BinaryMime => buf.extend_from_slice(b"BINARYMIME"),
        }
    }
}

impl ToBytes for Mail {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"MAIL FROM:");
        self.from.to_bytes_into(buf);

        if let Some(size) = self.size {
            buf.extend_from_slice(b" SIZE=");
            let mut f = itoa::Buffer::new();
            buf.extend_from_slice(f.format(size).as_bytes());
        }

        if let Some(ret) = self.ret {
            buf.extend_from_slice(b" ");
            ret.to_bytes_into(buf);
        }

        if let Some(envid) = &self.envid {
            buf.extend_from_slice(b" ");
            envid.to_bytes_into(buf);
        }

        if let Some(auth) = &self.auth {
            buf.extend_from_slice(b" ");
            auth.to_bytes_into(buf);
        }

        if let Some(body) = &self.body {
            buf.extend_from_slice(b" ");
            body.to_bytes_into(buf);
        }

        buf.extend_from_slice(b"\r\n");
    }
}

impl ToBytes for Notify {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"NOTIFY=");
        if self.never() {
            buf.extend_from_slice(b"NEVER");
            return;
        }

        let mut first = true;
        for flags in self.iter() {
            if !first {
                buf.extend_from_slice(b",");
            }

            first = false;

            match flags {
                Self::SUCCESS => buf.extend_from_slice(b"SUCCESS"),
                Self::FAILURE => buf.extend_from_slice(b"FAILURE"),
                Self::DELAY => buf.extend_from_slice(b"DELAY"),
                _ => unreachable!(),
            }
        }
    }
}

impl ToBytes for Rcpt {
    fn to_bytes_into(&self, buf: &mut BytesMut) {
        buf.extend_from_slice(b"RCPT TO:");
        self.to.to_bytes_into(buf);
        buf.extend_from_slice(b"\r\n");
    }
}
