// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tags (WS-003): hierarchical labels like `bio::cells`, matched regardless of case.
//!
//! A tag is one row per distinct path; its parents (`bio`) are implied by the prefix and
//! need no row. Items are tagged through `taggings`, a polymorphic join (09 §3).

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

use crate::Core;
use crate::domain::Id;
use crate::error::{Error, Result};
use crate::storage::oplog::{self, Op};
use crate::write::Writer;

/// Separator between tag levels (same as decks and Anki).
pub const TAG_SEPARATOR: &str = "::";

const MAX_TAG_LEN: usize = 200;

/// Entity kinds that can carry tags. Only facts exist so far (notes, tasks, exams and
/// resources join as they are built).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Taggable {
    Fact,
}

impl Taggable {
    fn as_str(self) -> &'static str {
        match self {
            Taggable::Fact => "fact",
        }
    }
}

/// Canonical form of a tag path: segments trimmed, no empty segments, no whitespace or
/// control characters inside (tags are space-separated in Anki and in queries).
pub fn normalize(raw: &str) -> Result<String> {
    let invalid = |why: &str| Error::validation(format!("invalid tag '{raw}': {why}"));
    let parts: Vec<&str> = raw.trim().split(TAG_SEPARATOR).map(str::trim).collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(invalid("levels must not be empty (use 'parent::child')"));
    }
    if parts
        .iter()
        .any(|p| p.chars().any(|c| c.is_whitespace() || c.is_control()))
    {
        return Err(invalid("tags can't contain spaces"));
    }
    let path = parts.join(TAG_SEPARATOR);
    if path.chars().count() > MAX_TAG_LEN {
        return Err(invalid("longer than 200 characters"));
    }
    Ok(path)
}

/// The case-folded key tags are matched by.
fn key(path: &str) -> String {
    path.to_lowercase()
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TagInfo {
    pub path: String,
    /// Live facts carrying exactly this tag.
    pub facts: u64,
}

impl Writer<'_> {
    /// The tag for an already-normalised path, created on first use. Returns its id and
    /// whether it was created.
    pub fn ensure_tag(&mut self, workspace: Id, path: &str) -> Result<(Id, bool)> {
        let path_key = key(path);
        let existing: Option<Vec<u8>> = self
            .conn
            .prepare_cached(
                "SELECT id FROM tags
                 WHERE workspace_id = ?1 AND path_key = ?2 AND deleted_at IS NULL",
            )?
            .query_row(params![workspace.as_bytes(), path_key], |r| r.get(0))
            .optional()?;
        if let Some(bytes) = existing {
            return Ok((Id::from_slice(&bytes)?, false));
        }
        let id = Id::new();
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "INSERT INTO tags (id, workspace_id, path, path_key, created_at, updated_at,
                                   hlc, origin_device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
            )?
            .execute(params![
                id.as_bytes(),
                workspace.as_bytes(),
                path,
                path_key,
                self.now,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "tag",
            id,
            Op::Insert,
            &serde_json::json!({ "path": path }),
        )?;
        Ok((id, true))
    }

    /// Attach a tag to an item; does nothing if it's already attached.
    pub fn tag_item(&mut self, tag: Id, kind: Taggable, item: Id) -> Result<()> {
        let id = Id::new();
        let hlc = self.stamp();
        let inserted = self
            .conn
            .prepare_cached(
                "INSERT INTO taggings (id, tag_id, item_kind, item_id, created_at, hlc,
                                       origin_device)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7
                 WHERE NOT EXISTS (SELECT 1 FROM taggings
                                   WHERE tag_id = ?2 AND item_kind = ?3 AND item_id = ?4
                                     AND deleted_at IS NULL)",
            )?
            .execute(params![
                id.as_bytes(),
                tag.as_bytes(),
                kind.as_str(),
                item.as_bytes(),
                self.now,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        if inserted == 0 {
            return Ok(());
        }
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "tagging",
            id,
            Op::Insert,
            &serde_json::json!({
                "tag_id": tag.to_string(),
                "item_kind": kind.as_str(),
                "item_id": item.to_string(),
            }),
        )?;
        Ok(())
    }

    /// Tag an item with each of `paths` (already normalised). Returns how many tags were
    /// created.
    pub fn tag_with(
        &mut self,
        workspace: Id,
        kind: Taggable,
        item: Id,
        paths: &[String],
    ) -> Result<u64> {
        let mut created = 0;
        for path in paths {
            let (tag, new) = self.ensure_tag(workspace, path)?;
            created += u64::from(new);
            self.tag_item(tag, kind, item)?;
        }
        Ok(created)
    }
}

/// Normalise a list of tags and drop case-insensitive duplicates, keeping the first spelling.
pub fn normalize_all<'a>(raw: impl IntoIterator<Item = &'a str>) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    for r in raw {
        let path = normalize(r)?;
        if !out.iter().any(|p| key(p) == key(&path)) {
            out.push(path);
        }
    }
    Ok(out)
}

impl Core {
    /// Live tags in a workspace, sorted by path, with how many facts carry each.
    pub fn list_tags(&self, workspace: Id) -> Result<Vec<TagInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT t.path,
                    (SELECT COUNT(*) FROM taggings g JOIN facts f ON f.id = g.item_id
                      WHERE g.tag_id = t.id AND g.item_kind = 'fact'
                        AND g.deleted_at IS NULL AND f.deleted_at IS NULL)
             FROM tags t
             WHERE t.workspace_id = ?1 AND t.deleted_at IS NULL
             ORDER BY t.path_key",
        )?;
        let tags = stmt
            .query_map([workspace.as_bytes()], |r| {
                Ok(TagInfo {
                    path: r.get(0)?,
                    facts: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(tags)
    }

    /// Tags on a fact, sorted by path.
    pub fn fact_tags(&self, fact: Id) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT t.path FROM taggings g JOIN tags t ON t.id = g.tag_id
             WHERE g.item_kind = 'fact' AND g.item_id = ?1
               AND g.deleted_at IS NULL AND t.deleted_at IS NULL
             ORDER BY t.path_key",
        )?;
        let tags = stmt
            .query_map([fact.as_bytes()], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_levels_and_rejects_bad_tags() {
        assert_eq!(normalize(" bio :: cells ").unwrap(), "bio::cells");
        assert_eq!(normalize("Ökologie").unwrap(), "Ökologie");
        for bad in ["", "bio::", "::bio", "a::::b", "two words", "tab\there"] {
            assert!(normalize(bad).is_err(), "{bad:?} should be rejected");
        }
        assert!(normalize(&"x".repeat(201)).is_err());
    }

    #[test]
    fn duplicates_match_regardless_of_case() {
        let tags = normalize_all(["Bio", "bio", "ÖKO", "öko", "chem"]).unwrap();
        assert_eq!(tags, ["Bio", "ÖKO", "chem"]);
    }
}
