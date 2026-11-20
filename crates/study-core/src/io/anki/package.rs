// SPDX-License-Identifier: AGPL-3.0-or-later
//! Opening `.apkg` / `.colpkg` files (IO-001).
//!
//! A package is a zip. The real collection is `collection.anki21b` (zstd-compressed SQLite,
//! schema 18, Anki 2.1.50+ default), else `collection.anki21`, else `collection.anki2`
//! (schema 11). Packages also carry a stub `collection.anki2` telling old Anki versions to
//! update, so the order matters. Only these known entries are read: nothing is extracted
//! to paths taken from the archive.
//!
//! Media: the `media` entry maps zip entry names (`"0"`, `"1"`, …) to file names. Legacy
//! packages store files as they are; latest ones zstd-compress each file.

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
    pub media: Vec<MediaEntry>,
    zip: zip::ZipArchive<File>,
}

/// One media file in a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MediaEntry {
    /// Zip entry holding the content.
    pub entry: String,
    /// The name fields refer to it by. Not validated here.
    pub name: String,
    /// Uncompressed size as declared by the package.
    pub size: u64,
}

impl Package {
    /// The content of a media entry, decompressed.
    pub fn media_reader(&mut self, entry: &str) -> Result<Box<dyn Read + '_>> {
        let file = self
            .zip
            .by_name(entry)
            .map_err(|e| Error::validation(format!("media entry '{entry}': {e}")))?;
        Ok(match self.format {
            Format::Legacy => Box::new(file),
            Format::Latest => Box::new(
                ruzstd::decoding::StreamingDecoder::new(file)
                    .map_err(|e| Error::validation(format!("media entry '{entry}': zstd: {e}")))?,
            ),
        })
    }
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

    let mut media = if has("media") {
        let mut raw = Vec::new();
        zip.by_name("media")
            .map_err(|e| invalid(path, e))?
            .take(16 << 20)
            .read_to_end(&mut raw)?;
        media_map(&raw, format)
    } else {
        Vec::new()
    };
    // Legacy packages don't declare sizes in the map: use the zip's own.
    for m in &mut media {
        if format == Format::Legacy {
            m.size = zip.by_name(&m.entry).map(|f| f.size()).unwrap_or(0);
        }
    }
    media.retain(|m| has(&m.entry));

    Ok(Package {
        _dir: dir,
        db_path,
        format,
        media,
        zip,
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
/// (`repeated MediaEntry entries = 1`, each `name = 1`, `size = 2`, `sha1 = 3`,
/// `legacy_zip_filename = 255`); the zip entry is the list index unless the last is set.
/// Malformed maps yield no media rather than failing the import.
fn media_map(raw: &[u8], format: Format) -> Vec<MediaEntry> {
    match format {
        Format::Legacy => serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(raw)
            .map(|m| {
                m.into_iter()
                    .filter_map(|(entry, name)| {
                        Some(MediaEntry {
                            entry,
                            name: name.as_str()?.to_string(),
                            size: 0,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        Format::Latest => {
            let mut decoded = Vec::new();
            let Ok(mut decoder) = ruzstd::decoding::StreamingDecoder::new(raw) else {
                return Vec::new();
            };
            if decoder.read_to_end(&mut decoded).is_err() {
                return Vec::new();
            }
            proto::repeated_bytes(&decoded, 1)
                .into_iter()
                .enumerate()
                .filter_map(|(i, e)| {
                    Some(MediaEntry {
                        entry: proto::varint_field(e, 255)
                            .map_or_else(|| i.to_string(), |n| n.to_string()),
                        name: proto::string_field(e, 1)?,
                        size: proto::varint_field(e, 2).unwrap_or(0),
                    })
                })
                .collect()
        }
    }
}
