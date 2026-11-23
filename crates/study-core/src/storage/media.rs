// SPDX-License-Identifier: AGPL-3.0-or-later
//! Content-addressed media files on disk (D-026; 08 §7): `media/ab/cd/<sha256>.<ext>`.
//!
//! Files are immutable: the same content is stored once, whatever it's called. Paths come
//! only from the hash and a sanitised extension, never from names supplied by users or
//! archives. A file is written to a temporary name, synced, then renamed into place, so a
//! crash never leaves a partial file under a real hash. Garbage collection (files nothing
//! references, after the backup window) comes later.

use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

pub struct MediaStore {
    root: PathBuf,
}

/// A file now in the store.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stored {
    /// Lowercase hex SHA-256 of the content.
    pub hash: String,
    pub size: u64,
}

/// Hashes everything written through it.
struct HashingWriter<W> {
    inner: W,
    hasher: Sha256,
    written: u64,
}

impl<W: Write> Write for HashingWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        self.written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

/// Extensions used on disk: lowercase ASCII letters and digits, at most 10, else `bin`.
pub fn safe_ext(ext: &str) -> String {
    let ext = ext.to_ascii_lowercase();
    if !ext.is_empty() && ext.len() <= 10 && ext.bytes().all(|b| b.is_ascii_alphanumeric()) {
        ext
    } else {
        "bin".into()
    }
}

impl MediaStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        MediaStore { root: root.into() }
    }

    /// Where a file with this hash and extension lives.
    pub fn path(&self, hash: &str, ext: &str) -> PathBuf {
        self.root
            .join(&hash[..2])
            .join(&hash[2..4])
            .join(format!("{hash}.{}", safe_ext(ext)))
    }

    /// Stream `reader` into the store. Fails with a validation error, leaving nothing
    /// behind, if it yields more than `max_bytes`.
    // @card What happens to the `NamedTempFile` in `put` when it returns early?
    // It's deleted when dropped; only `tmp.persist(&target)` keeps it. So an oversized
    // file or one already stored leaves nothing behind in `tmp/`.
    pub fn put(&self, reader: impl Read, ext: &str, max_bytes: u64) -> Result<Stored> {
        let tmp_dir = self.root.join("tmp");
        fs::create_dir_all(&tmp_dir)?;
        let tmp = tempfile::NamedTempFile::new_in(&tmp_dir)?;
        let mut w = HashingWriter {
            inner: tmp.as_file(),
            hasher: Sha256::new(),
            written: 0,
        };
        io::copy(&mut reader.take(max_bytes.saturating_add(1)), &mut w)?;
        if w.written > max_bytes {
            return Err(Error::validation(format!(
                "media file is larger than {max_bytes} bytes"
            )));
        }
        let (hash, size) = (hex(&w.hasher.finalize()), w.written);
        let target = self.path(&hash, ext);
        if target.exists() {
            return Ok(Stored { hash, size }); // already stored: tmp is deleted on drop
        }
        tmp.as_file().sync_all()?;
        fs::create_dir_all(target.parent().expect("media path has a parent"))?;
        tmp.persist(&target).map_err(|e| Error::Io(e.error))?;
        Ok(Stored { hash, size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_content_is_stored_once_under_its_hash() {
        let dir = tempfile::tempdir().unwrap();
        let store = MediaStore::new(dir.path());
        let a = store.put(&b"hello"[..], "PNG", 100).unwrap();
        // sha256("hello")
        assert_eq!(
            a.hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        assert_eq!(a.size, 5);
        let path = store.path(&a.hash, "png");
        assert!(path.ends_with(format!("2c/f2/{}.png", a.hash)), "{path:?}");
        assert_eq!(fs::read(&path).unwrap(), b"hello");

        let b = store.put(&b"hello"[..], "png", 100).unwrap();
        assert_eq!(a, b);
        assert_eq!(fs::read_dir(dir.path().join("tmp")).unwrap().count(), 0);
    }

    #[test]
    fn oversized_input_leaves_nothing_behind() {
        let dir = tempfile::tempdir().unwrap();
        let store = MediaStore::new(dir.path());
        assert!(store.put(&[0u8; 11][..], "bin", 10).is_err());
        assert_eq!(fs::read_dir(dir.path().join("tmp")).unwrap().count(), 0);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1, "only tmp/");
    }

    #[test]
    fn extensions_are_sanitised() {
        assert_eq!(safe_ext("JPG"), "jpg");
        assert_eq!(safe_ext("../x"), "bin");
        assert_eq!(safe_ext(""), "bin");
        assert_eq!(safe_ext("averyverylongext"), "bin");
    }
}
