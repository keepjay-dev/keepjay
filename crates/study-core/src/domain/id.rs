// SPDX-License-Identifier: AGPL-3.0-or-later
//! UUIDv7 identifiers (D-009), stored as 16-byte BLOBs and shown as lowercase
//! Crockford base32. The short form is the *last* 8 characters: the leading characters
//! of a UUIDv7 are its timestamp, so items created within ~1 s would share a prefix.
//! The tail is random (40 bits), so it's what the CLI shows and accepts (CLI-007).

use std::fmt;
use std::str::FromStr;

use crate::error::{Error, Result};

const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id(uuid::Uuid);

impl Id {
    /// New time-sortable id generated on this device.
    pub fn new() -> Self {
        Id(uuid::Uuid::now_v7())
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        self.0.as_bytes()
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        uuid::Uuid::from_slice(bytes)
            .map(Id)
            .map_err(|e| Error::Unexpected(format!("invalid id bytes: {e}")))
    }

    /// Last 8 characters of the base32 form (random bits; see module docs).
    // @card(kph8grz0) Why is a short id the *last* 8 characters of a UUIDv7, not the first?
    // The leading characters are the timestamp, so ids created within about a second share
    // them; the tail is 40 random bits.
    pub fn short(&self) -> String {
        self.to_string()[18..].to_string()
    }

    /// A new id whose short form is `short`. Used for card ids written in source files
    /// (SRC-005): a teammate's clone creates facts with the same short ids. The low 40
    /// bits of a UUIDv7 are random, so replacing them keeps it a valid UUIDv7.
    pub fn with_short(short: &str) -> Result<Self> {
        if short.len() != 8 {
            return Err(Error::validation(format!(
                "'{short}' is not a short id (8 characters)"
            )));
        }
        let mut tail: u128 = 0;
        for c in short.bytes() {
            let digit = ALPHABET
                .iter()
                .position(|&a| a == c.to_ascii_lowercase())
                .ok_or_else(|| Error::validation(format!("invalid character in id '{short}'")))?;
            tail = (tail << 5) | digit as u128;
        }
        let base = u128::from_be_bytes(*uuid::Uuid::now_v7().as_bytes());
        let mask = (1u128 << 40) - 1;
        Ok(Id(uuid::Uuid::from_bytes(
            ((base & !mask) | tail).to_be_bytes(),
        )))
    }
}

impl Default for Id {
    fn default() -> Self {
        Id::new()
    }
}

impl fmt::Display for Id {
    /// 26-character Crockford base32 (128 bits, big-endian, 2 padding bits up front).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = u128::from_be_bytes(*self.0.as_bytes());
        let mut out = [0u8; 26];
        for (i, slot) in out.iter_mut().enumerate() {
            let shift = 5 * (25 - i);
            *slot = ALPHABET[((v >> shift) & 0x1f) as usize];
        }
        f.write_str(std::str::from_utf8(&out).expect("alphabet is ASCII"))
    }
}

impl fmt::Debug for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Id({self})")
    }
}

impl FromStr for Id {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.len() != 26 {
            return Err(Error::validation(format!(
                "'{s}' is not a full id (26 characters)"
            )));
        }
        let mut v: u128 = 0;
        for c in s.bytes() {
            let c = c.to_ascii_lowercase();
            let digit = ALPHABET
                .iter()
                .position(|&a| a == c)
                .ok_or_else(|| Error::validation(format!("invalid character in id '{s}'")))?;
            v = (v << 5) | digit as u128;
        }
        Ok(Id(uuid::Uuid::from_bytes(v.to_be_bytes())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_through_text_and_bytes() {
        let id = Id::new();
        let text = id.to_string();
        assert_eq!(text.len(), 26);
        assert_eq!(text.parse::<Id>().unwrap(), id);
        assert_eq!(Id::from_slice(id.as_bytes()).unwrap(), id);
        assert_eq!(id.short(), &text[18..]);
    }

    #[test]
    fn ids_are_time_sortable() {
        let a = Id::new();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = Id::new();
        assert!(a < b);
        assert!(a.to_string() < b.to_string(), "text form keeps sort order");
    }

    #[test]
    fn short_ids_differ_for_ids_created_together() {
        let shorts: std::collections::HashSet<String> =
            (0..1000).map(|_| Id::new().short()).collect();
        assert_eq!(shorts.len(), 1000);
    }

    #[test]
    fn with_short_keeps_the_tail_and_stays_a_v7() {
        let id = Id::with_short("k22k1m8e").unwrap();
        assert_eq!(id.short(), "k22k1m8e");
        assert_eq!(id.0.get_version_num(), 7);
        assert_eq!(id.to_string().parse::<Id>().unwrap(), id);
        assert!(Id::with_short("k22k1m8").is_err());
        assert!(Id::with_short("k22k1m8u").is_err());
    }

    #[test]
    fn rejects_bad_input() {
        assert!("short".parse::<Id>().is_err());
        assert!("uuuuuuuuuuuuuuuuuuuuuuuuuu".parse::<Id>().is_err()); // 'u' not in alphabet
    }
}
