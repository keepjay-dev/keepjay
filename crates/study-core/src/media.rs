// SPDX-License-Identifier: AGPL-3.0-or-later
//! Media used by facts (D-026, IO-001): images and sounds referenced by name from field HTML,
//! the way Anki does it (`<img src="cell.png">`, `[sound:hund.mp3]`).
//!
//! Content lives in the content-addressed store ([`crate::storage::media`]); each workspace
//! maps names to content in `media_names`. Fields keep the names unchanged, so they stay
//! lossless and exportable. Attachments and the resource library (RES-001, P1) build on the
//! same store.

use std::collections::HashMap;
use std::ops::Range;
use std::path::PathBuf;

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

use crate::Core;
use crate::cards::render::decode_entities;
use crate::domain::Id;
use crate::error::Result;
use crate::storage::media::{MediaStore, Stored, safe_ext};
use crate::storage::oplog::{self, Op};
use crate::write::Writer;

/// A media name is usable if it's a plain file name: no path separators, no control
/// characters, not `.`/`..`, at most 255 bytes.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 255
        && name != "."
        && name != ".."
        && !name
            .chars()
            .any(|c| c == '/' || c == '\\' || c.is_control())
}

/// The extension of a name as used on disk (`Cell.PNG` → `png`).
pub fn ext_of(name: &str) -> String {
    safe_ext(name.rsplit_once('.').map(|(_, e)| e).unwrap_or(""))
}

pub fn mime_of(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "mp3" => "audio/mpeg",
        "ogg" | "oga" => "audio/ogg",
        "opus" => "audio/opus",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "pdf" => "application/pdf",
        _ => return None,
    })
}

/// Tags whose `src` (or `data` for `object`) names a media file.
const MEDIA_TAGS: &[&str] = &["img", "audio", "video", "source", "object"];

/// Byte ranges of the media names referenced in field HTML: `[sound:…]` and `src`
/// attributes of media tags. Remote URLs and `data:` URIs are not media references.
pub fn references(html: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = html[from..].find("[sound:") {
        let start = from + i + "[sound:".len();
        let Some(len) = html[start..].find(']') else {
            break;
        };
        out.push(start..start + len);
        from = start + len;
    }
    let lower = html.to_ascii_lowercase(); // same byte offsets as `html`
    let mut from = 0;
    while let Some(i) = lower[from..].find('<') {
        let tag_start = from + i + 1;
        let Some(len) = lower[tag_start..].find('>') else {
            break;
        };
        let tag = &lower[tag_start..tag_start + len];
        from = tag_start + len;
        let name: String = tag
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if !MEDIA_TAGS.contains(&name.as_str()) {
            continue;
        }
        let attr = if name == "object" { "data" } else { "src" };
        if let Some(r) = attr_value(tag, attr) {
            out.push(tag_start + r.start..tag_start + r.end);
        }
    }
    out.retain(|r| {
        let v = html[r.clone()].trim();
        !v.is_empty() && !v.contains("://") && !v.starts_with("data:") && !v.starts_with("//")
    });
    out.sort_by_key(|r| r.start);
    let mut end = 0;
    out.retain(|r| {
        let keep = r.start >= end; // drop overlaps, e.g. `[sound:` inside an attribute
        end = end.max(r.end);
        keep
    });
    out
}

/// Range of an attribute's value inside a (lowercased) tag body. The attribute name must
/// start the tag or follow whitespace, so `data-src` doesn't match `src`.
fn attr_value(tag: &str, name: &str) -> Option<Range<usize>> {
    let pattern = format!("{name}=");
    let mut from = 0;
    let at = loop {
        let i = from + tag[from..].find(&pattern)?;
        if i > 0 && tag.as_bytes()[i - 1].is_ascii_whitespace() {
            break i + pattern.len();
        }
        from = i + pattern.len();
    };
    let quote = tag[at..].chars().next()?;
    if quote == '"' || quote == '\'' {
        let len = tag[at + 1..].find(quote)?;
        Some(at + 1..at + 1 + len)
    } else {
        let len = tag[at..]
            .find(|c: char| c.is_whitespace() || c == '/')
            .unwrap_or(tag.len() - at);
        Some(at..at + len)
    }
}

/// The media names a field refers to, decoded (`a&amp;b.png` → `a&b.png`).
pub fn referenced_names(html: &str) -> impl Iterator<Item = String> + '_ {
    references(html)
        .into_iter()
        .map(|r| decode_entities(html[r].trim()))
}

/// Replace referenced names according to `renames` (decoded old name → new name).
pub fn rewrite_references(html: &str, renames: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(html.len());
    let mut last = 0;
    for r in references(html) {
        let Some(new) = renames.get(&decode_entities(html[r.clone()].trim())) else {
            continue;
        };
        out.push_str(&html[last..r.start]);
        out.push_str(&new.replace('&', "&amp;").replace('"', "&quot;"));
        last = r.end;
    }
    out.push_str(&html[last..]);
    out
}

/// What happened to one imported file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Attached {
    /// New name in the workspace.
    Added,
    /// The workspace already had this name with this content.
    AlreadyPresent,
    /// The name was taken by other content; stored under this name instead.
    Renamed(String),
}

/// `cell.png` → `cell-1a2b3c4d.png`.
fn name_with_hash(name: &str, hash: &str) -> String {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => format!("{stem}-{}.{ext}", &hash[..8]),
        _ => format!("{name}-{}", &hash[..8]),
    }
}

impl Writer<'_> {
    fn media_name_hash(&self, workspace: Id, name: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .prepare_cached(
                "SELECT hash FROM media_names
                 WHERE workspace_id = ?1 AND name = ?2 AND deleted_at IS NULL",
            )?
            .query_row(params![workspace.as_bytes(), name], |r| r.get(0))
            .optional()?)
    }

    /// Record a stored file and give it `name` in the workspace. If the name already
    /// means other content, uses `name-<hash8>.ext` instead, like Anki.
    pub(crate) fn attach_media(
        &mut self,
        workspace: Id,
        name: &str,
        stored: &Stored,
        ext: &str,
    ) -> Result<Attached> {
        self.conn
            .prepare_cached(
                "INSERT OR IGNORE INTO media (hash, ext, mime, size, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
            )?
            .execute(params![
                stored.hash,
                ext,
                mime_of(ext),
                stored.size,
                self.now
            ])?;
        let mut final_name = name.to_string();
        let outcome = match self.media_name_hash(workspace, name)? {
            Some(h) if h == stored.hash => return Ok(Attached::AlreadyPresent),
            None => Attached::Added,
            Some(_) => {
                final_name = name_with_hash(name, &stored.hash);
                match self.media_name_hash(workspace, &final_name)? {
                    Some(h) if h == stored.hash => return Ok(Attached::Renamed(final_name)),
                    Some(_) => {
                        // A different file already carries the hashed name: vanishingly
                        // unlikely, but never overwrite. Fall back to the full hash.
                        final_name = format!("{}.{ext}", stored.hash);
                    }
                    None => {}
                }
                Attached::Renamed(final_name.clone())
            }
        };
        let id = Id::new();
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "INSERT INTO media_names (id, workspace_id, name, hash, created_at, updated_at,
                                          hlc, origin_device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
            )?
            .execute(params![
                id.as_bytes(),
                workspace.as_bytes(),
                final_name,
                stored.hash,
                self.now,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "media_name",
            id,
            Op::Insert,
            &serde_json::json!({ "name": final_name, "hash": stored.hash }),
        )?;
        Ok(outcome)
    }

    /// Whether the workspace has a file under this name.
    pub(crate) fn has_media(&self, workspace: Id, name: &str) -> Result<bool> {
        Ok(self.media_name_hash(workspace, name)?.is_some())
    }
}

/// Result of [`Core::add_media`].
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MediaFile {
    /// The name fields should use: `name`, or `name-<hash8>.ext` if `name` was taken.
    pub name: String,
    pub hash: String,
    pub size: u64,
}

/// Largest file [`Core::add_media`] accepts.
pub const MAX_MEDIA_FILE: u64 = 1 << 30;

impl Core {
    pub(crate) fn media_store(&self) -> MediaStore {
        MediaStore::new(self.paths.media_dir())
    }

    /// Store a file and make it available to fields under `name` (or the returned name, if
    /// `name` already means other content).
    pub fn add_media(
        &mut self,
        workspace: Id,
        name: &str,
        content: impl std::io::Read,
    ) -> Result<MediaFile> {
        if !valid_name(name) {
            return Err(crate::error::Error::validation(format!(
                "invalid media name '{name}': use a plain file name"
            )));
        }
        let ext = ext_of(name);
        let stored = self.media_store().put(content, &ext, MAX_MEDIA_FILE)?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: crate::domain::now_ms(),
        };
        let name = match w.attach_media(workspace, name, &stored, &ext)? {
            Attached::Renamed(new) => new,
            _ => name.to_string(),
        };
        tx.commit()?;
        Ok(MediaFile {
            name,
            hash: stored.hash,
            size: stored.size,
        })
    }

    /// The file a media name refers to in a workspace, if there is one.
    pub fn media_file(&self, workspace: Id, name: &str) -> Result<Option<PathBuf>> {
        let found: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT m.hash, m.ext FROM media_names n JOIN media m ON m.hash = n.hash
                 WHERE n.workspace_id = ?1 AND n.name = ?2 AND n.deleted_at IS NULL",
                params![workspace.as_bytes(), name],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        Ok(found.map(|(hash, ext)| self.media_store().path(&hash, &ext)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(html: &str) -> Vec<String> {
        referenced_names(html).collect()
    }

    #[test]
    fn finds_images_sounds_and_other_media() {
        assert_eq!(
            names(r#"Mito<br><IMG class=x SRC="cell.png"> [sound:hund.mp3] <img src='a b.jpg'>"#),
            ["cell.png", "hund.mp3", "a b.jpg"]
        );
        assert_eq!(
            names(r#"<img src=plain.gif/><audio src="s.ogg"></audio><object data="d.svg">"#),
            ["plain.gif", "s.ogg", "d.svg"]
        );
        assert_eq!(names(r#"<img src="a&amp;b.png">"#), ["a&b.png"]);
    }

    #[test]
    fn ignores_remote_urls_data_uris_and_lookalike_attributes() {
        assert!(
            names(r#"<img src="https://x.org/a.png"><img src="data:image/png;base64,AA">"#)
                .is_empty()
        );
        assert!(names(r#"<img data-src="lazy.png"><a src="x.png">"#).is_empty());
        assert!(names("a < b and [sound:").is_empty());
    }

    #[test]
    fn rewrites_only_renamed_references() {
        let renames = HashMap::from([("cell.png".to_string(), "cell-1a2b3c4d.png".to_string())]);
        assert_eq!(
            rewrite_references(
                r#"<img src="cell.png"> <img src="other.png"> [sound:cell.png]"#,
                &renames
            ),
            r#"<img src="cell-1a2b3c4d.png"> <img src="other.png"> [sound:cell-1a2b3c4d.png]"#
        );
    }

    #[test]
    fn names_and_extensions() {
        assert!(valid_name("cell.png") && valid_name("_font.ttf") && valid_name("Zelle ü.png"));
        for bad in ["", ".", "..", "a/b.png", "..\\x", "a\0b", &"x".repeat(256)] {
            assert!(!valid_name(bad), "{bad:?}");
        }
        assert_eq!(ext_of("Cell.PNG"), "png");
        assert_eq!(ext_of("noext"), "bin");
        assert_eq!(
            name_with_hash("cell.png", "1a2b3c4d5e"),
            "cell-1a2b3c4d.png"
        );
        assert_eq!(name_with_hash(".hidden", "1a2b3c4d5e"), ".hidden-1a2b3c4d");
    }
}
