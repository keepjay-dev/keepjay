// SPDX-License-Identifier: AGPL-3.0-or-later
//! A minimal, read-only protobuf wire-format reader: just enough to pull a few fields out
//! of Anki's schema-18 config blobs without generating code for its whole API.

fn varint(buf: &[u8], pos: &mut usize) -> Option<u64> {
    let mut value = 0u64;
    for shift in (0..64).step_by(7) {
        let byte = *buf.get(*pos)?;
        *pos += 1;
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some(value);
        }
    }
    None
}

#[derive(Debug, PartialEq)]
pub(crate) enum Value<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    Fixed,
}

/// Top-level `(field number, value)` pairs. Stops quietly at malformed input.
pub(crate) fn fields(buf: &[u8]) -> Vec<(u32, Value<'_>)> {
    let mut out = Vec::new();
    let mut pos = 0;
    while pos < buf.len() {
        let Some(key) = varint(buf, &mut pos) else {
            break;
        };
        let (number, wire) = ((key >> 3) as u32, key & 7);
        let value = match wire {
            0 => match varint(buf, &mut pos) {
                Some(v) => Value::Varint(v),
                None => break,
            },
            1 | 5 => {
                pos += if wire == 1 { 8 } else { 4 };
                if pos > buf.len() {
                    break;
                }
                Value::Fixed
            }
            2 => {
                let Some(len) = varint(buf, &mut pos) else {
                    break;
                };
                let end = pos.saturating_add(len as usize);
                let Some(bytes) = buf.get(pos..end) else {
                    break;
                };
                pos = end;
                Value::Bytes(bytes)
            }
            _ => break,
        };
        out.push((number, value));
    }
    out
}

pub(crate) fn varint_field(buf: &[u8], number: u32) -> Option<u64> {
    fields(buf).into_iter().rev().find_map(|(n, v)| match v {
        Value::Varint(x) if n == number => Some(x),
        _ => None,
    })
}

pub(crate) fn string_field(buf: &[u8], number: u32) -> Option<String> {
    fields(buf).into_iter().rev().find_map(|(n, v)| match v {
        Value::Bytes(b) if n == number => Some(String::from_utf8_lossy(b).into_owned()),
        _ => None,
    })
}

pub(crate) fn repeated_bytes(buf: &[u8], number: u32) -> Vec<&[u8]> {
    fields(buf)
        .into_iter()
        .filter_map(|(n, v)| match v {
            Value::Bytes(b) if n == number => Some(b),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_varints_strings_and_skips_unknown_wire_types() {
        // field 1 varint 1; field 3 bytes "abc"; field 8 varint 300; field 9 fixed32
        let buf = [
            0x08, 0x01, 0x1a, 0x03, b'a', b'b', b'c', 0x40, 0xac, 0x02, 0x4d, 1, 2, 3, 4,
        ];
        assert_eq!(varint_field(&buf, 1), Some(1));
        assert_eq!(string_field(&buf, 3).as_deref(), Some("abc"));
        assert_eq!(varint_field(&buf, 8), Some(300));
        assert!(fields(&buf).iter().any(|(n, _)| *n == 9));
        assert_eq!(varint_field(&buf, 2), None);
    }

    #[test]
    fn truncated_input_does_not_panic() {
        assert!(fields(&[0x1a, 0x05, b'a']).is_empty());
        assert!(fields(&[0xff]).is_empty());
    }
}
