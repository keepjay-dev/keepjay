// SPDX-License-Identifier: AGPL-3.0-or-later
//! Opening `.apkg` / `.colpkg` files (IO-001).
//!
//! A package is a zip. The real collection is `collection.anki21b` (zstd-compressed SQLite,
//! schema 18, Anki 2.1.50+ default), else `collection.anki21`, else `collection.anki2`
//! (schema 11). Packages also carry a stub `collection.anki2` telling old Anki versions to
//! update, so the order matters. Only these known entries are read: nothing is extracted
//! to paths taken from the archive.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::io::proto;

/// Refuse to inflate a collection beyond this (protects against zip bombs).
const MAX_COLLECTION_BYTES: u64 = 4 << 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Format {
    /// `collection.anki21` / `.anki2`, schema 11, plain SQLite.
    Legacy,
    /// `collection.anki21b`, schema 18, zstd-compressed.
    Latest,
}

pub(crate) struct Package {
    /// Holds the extracted collection; deleted when dropped.
    _dir: tempfile::TempDir,
    pub db_path: PathBuf,
    pub format: Format,
    pub media_files: u64,
}

fn invalid(path: &Path, why: impl std::fmt::Display) -> Error {
    Error::validation(format!(
        "'{}' isn't a readable Anki package: {why}",
        path.display()
    ))
}

pub(crate) fn open(path: &Path) -> Result<Package> {
    let file = File::open(path)
        .map_err(|e| Error::NotFound(format!("can't open '{}': {e}", path.display())))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| invalid(path, e))?;
    let names: Vec<String> = zip
        .file_names()
        .filter_map(|n| n.ok().map(|n| n.into_owned()))
        .collect();
    let has = |n: &str| names.iter().any(|x| x == n);

    let dir = tempfile::tempdir()?;
    let db_path = dir.path().join("collection.sqlite");
    let format = if has("collection.anki21b") {
        let entry = zip
            .by_name("collection.anki21b")
            .map_err(|e| invalid(path, e))?;
        let decoder = ruzstd::decoding::StreamingDecoder::new(entry)
            .map_err(|e| invalid(path, format!("zstd: {e}")))?;
        copy_limited(decoder, &db_path)?;
        Format::Latest
    } else {
        let name = ["collection.anki21", "collection.anki2"]
            .into_iter()
            .find(|n| has(n))
            .ok_or_else(|| invalid(path, "no collection inside"))?;
        let entry = zip.by_name(name).map_err(|e| invalid(path, e))?;
        copy_limited(entry, &db_path)?;
        Format::Legacy
    };

    let media_files = if has("media") {
        let mut raw = Vec::new();
        zip.by_name("media")
            .map_err(|e| invalid(path, e))?
            .take(16 << 20)
            .read_to_end(&mut raw)?;
        count_media(&raw, format)
    } else {
        0
    };

    Ok(Package {
        _dir: dir,
        db_path,
        format,
        media_files,
    })
}

fn copy_limited(reader: impl Read, to: &Path) -> Result<()> {
    let mut out = File::create(to)?;
    let copied = std::io::copy(&mut reader.take(MAX_COLLECTION_BYTES + 1), &mut out)?;
    if copied > MAX_COLLECTION_BYTES {
        return Err(Error::validation("Anki collection is larger than 4 GiB"));
    }
    out.flush()?;
    Ok(())
}

/// Legacy: JSON `{"0": "cell.png", ...}`. Latest: zstd-compressed `MediaEntries` protobuf
/// (`repeated MediaEntry entries = 1`).
fn count_media(raw: &[u8], format: Format) -> u64 {
    match format {
        Format::Legacy => serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(raw)
            .map(|m| m.len() as u64)
            .unwrap_or(0),
        Format::Latest => {
            let mut decoded = Vec::new();
            let Ok(mut decoder) = ruzstd::decoding::StreamingDecoder::new(raw) else {
                return 0;
            };
            if decoder.read_to_end(&mut decoded).is_err() {
                return 0;
            }
            proto::repeated_bytes(&decoded, 1).len() as u64
        }
    }
}
