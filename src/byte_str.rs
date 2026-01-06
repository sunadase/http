use bytes::Bytes;

use std::{ops, str};

#[derive(Debug, Clone, Ord, PartialOrd)]
pub(crate) struct ByteStr {
    // Invariant: bytes contains valid UTF-8
    bytes: Bytes,
}

impl ByteStr {
    #[inline]
    pub fn new() -> ByteStr {
        ByteStr {
            // Invariant: the empty slice is trivially valid UTF-8.
            bytes: Bytes::new(),
        }
    }

    #[inline]
    pub const fn from_static(val: &'static str) -> ByteStr {
        ByteStr {
            // Invariant: val is a str so contains valid UTF-8.
            bytes: Bytes::from_static(val.as_bytes()),
        }
    }

    #[inline]
    /// ## Panics
    /// In a debug build this will panic if `bytes` is not valid UTF-8.
    ///
    /// ## Safety
    /// `bytes` must contain valid UTF-8. In a release build it is undefined
    /// behavior to call this with `bytes` that is not valid UTF-8.
    pub unsafe fn from_utf8_unchecked(bytes: Bytes) -> ByteStr {
        if cfg!(debug_assertions) {
            match str::from_utf8(&bytes) {
                Ok(_) => (),
                Err(err) => panic!(
                    "ByteStr::from_utf8_unchecked() with invalid bytes; error = {}, bytes = {:?}",
                    err, bytes
                ),
            }
        }
        // Invariant: assumed by the safety requirements of this function.
        ByteStr { bytes }
    }

    pub(crate) fn from_utf8(bytes: Bytes) -> Result<ByteStr, std::str::Utf8Error> {
        str::from_utf8(&bytes)?;
        // Invariant: just checked is utf8
        Ok(ByteStr { bytes })
    }
}

impl PartialEq for ByteStr {
    fn eq(&self, other: &Self) -> bool {
        self.bytes.len() == other.bytes.len()
            && self
                .bytes
                .iter()
                .zip(other.bytes.iter())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
    }
}

impl Eq for ByteStr {}

impl std::hash::Hash for ByteStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for b in &self.bytes {
            state.write_u8(b.to_ascii_lowercase());
        }
    }
}

impl ops::Deref for ByteStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        let b: &[u8] = self.bytes.as_ref();
        // Safety: the invariant of `bytes` is that it contains valid UTF-8.
        unsafe { str::from_utf8_unchecked(b) }
    }
}

impl From<String> for ByteStr {
    #[inline]
    fn from(src: String) -> ByteStr {
        ByteStr {
            // Invariant: src is a String so contains valid UTF-8.
            bytes: Bytes::from(src),
        }
    }
}

impl<'a> From<&'a str> for ByteStr {
    #[inline]
    fn from(src: &'a str) -> ByteStr {
        ByteStr {
            // Invariant: src is a str so contains valid UTF-8.
            bytes: Bytes::copy_from_slice(src.as_bytes()),
        }
    }
}

impl From<ByteStr> for Bytes {
    fn from(src: ByteStr) -> Self {
        src.bytes
    }
}
