// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sources (SRC-001; docs/specs/13-code-sources.md): folders and Git repositories that
//! are scanned for cards. Card content stays in the user's files; the database keeps the
//! registration, scheduling state and anchors (D-029).
//!
//! This module registers, lists and removes sources. Scanning arrives with SRC-002/003.

use std::path::{Path, PathBuf};
use std::process::Command;

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

use crate::Core;
use crate::cards::{DECK_SEPARATOR, Deck};
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::storage::oplog::{self, Op};
use crate::write::Writer;

const MAX_NAME_LEN: usize = 100;

/// What a source is. Problems and deck repositories arrive later (PROB-005, SRC-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Inside a Git work tree.
    Repo,
    /// A plain folder.
    Folder,
}

impl SourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SourceKind::Repo => "repo",
            SourceKind::Folder => "folder",
        }
    }

    fn parse(s: &str) -> Result<Self> {
        match s {
            "repo" => Ok(SourceKind::Repo),
            "folder" => Ok(SourceKind::Folder),
            other => Err(Error::Unexpected(format!("unknown source kind '{other}'"))),
        }
    }
}

/// A registered source.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Source {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub kind: SourceKind,
    pub path: String,
    /// Root commit hash of the Git repository, if it has commits.
    pub repo: Option<String>,
    pub deck_id: String,
    pub deck: String,
    pub tags: Vec<String>,
    /// Whether scans write card ids back into the files (SRC-005, OQ-028).
    pub write_ids: bool,
    pub last_scan_at: Option<i64>,
    pub created_at: i64,
}

/// Input for [`Core::add_source`].
#[derive(Debug, Clone, Default)]
pub struct NewSource {
    pub path: PathBuf,
    /// Defaults to the folder's name.
    pub name: Option<String>,
    /// Deck path for the source's cards. Defaults to the source name; created if missing.
    pub deck: Option<String>,
    pub tags: Vec<String>,
    pub write_ids: bool,
}

impl Core {
    /// Register a folder or Git repository as a source (SRC-001).
    pub fn add_source(&mut self, workspace: Id, new: NewSource) -> Result<Source> {
        let path = canonical_dir(&new.path)?;
        let path_text = display_path(&path);
        let name = match new.name.as_deref() {
            Some(n) => validate_name(n)?,
            None => validate_name(
                path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .as_deref()
                    .unwrap_or("source"),
            )?,
        };
        let deck_path = new.deck.clone().unwrap_or_else(|| name.clone());
        let deck_parts: Vec<&str> = deck_path.split(DECK_SEPARATOR).map(str::trim).collect();
        if deck_parts.iter().any(|p| p.is_empty()) {
            return Err(Error::validation(format!(
                "invalid deck path '{deck_path}': names must not be empty (use 'Parent{DECK_SEPARATOR}Child')"
            )));
        }
        let tags = crate::tags::normalize_all(new.tags.iter().map(String::as_str))?;
        let (kind, repo) = git_identity(&path);

        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let clash: Option<(String, String)> = tx
            .query_row(
                "SELECT name, path FROM sources
                 WHERE workspace_id = ?1 AND (name = ?2 OR path = ?3) AND deleted_at IS NULL",
                params![workspace.as_bytes(), name, path_text],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((existing_name, existing_path)) = clash {
            return Err(if existing_path == path_text {
                Error::conflict(format!(
                    "'{path_text}' is already registered as source '{existing_name}'"
                ))
            } else {
                Error::conflict(format!(
                    "a source named '{name}' already exists; choose another with --name"
                ))
            });
        }

        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let (deck_id, _) = w.ensure_deck_path(workspace, &deck_parts)?;
        let id = Id::new();
        let hlc = w.stamp();
        let tags_json = serde_json::to_string(&tags)
            .map_err(|e| Error::Unexpected(format!("could not encode tags: {e}")))?;
        tx.execute(
            "INSERT INTO sources (id, workspace_id, name, kind, path, repo, deck_id, tags,
                                  write_ids, created_at, updated_at, hlc, origin_device)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?11, ?12)",
            params![
                id.as_bytes(),
                workspace.as_bytes(),
                name,
                kind.as_str(),
                path_text,
                repo,
                deck_id.as_bytes(),
                tags_json,
                new.write_ids,
                w.now,
                hlc.to_string(),
                self.device_id.as_bytes()
            ],
        )?;
        oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "source",
            id,
            Op::Insert,
            &serde_json::json!({ "name": name, "kind": kind.as_str(), "repo": repo }),
        )?;
        tx.commit()?;
        self.source_by_id(workspace, id)
    }

    /// Live sources in a workspace, by name.
    pub fn list_sources(&self, workspace: Id) -> Result<Vec<Source>> {
        let decks = self.list_decks(workspace)?;
        let mut stmt = self.conn.prepare(
            "SELECT id, name, kind, path, repo, deck_id, tags, write_ids, last_scan_at, created_at
             FROM sources WHERE workspace_id = ?1 AND deleted_at IS NULL ORDER BY name",
        )?;
        let rows = stmt.query_map([workspace.as_bytes()], |r| {
            Ok(SourceRow {
                id: r.get(0)?,
                name: r.get(1)?,
                kind: r.get(2)?,
                path: r.get(3)?,
                repo: r.get(4)?,
                deck_id: r.get(5)?,
                tags: r.get(6)?,
                write_ids: r.get(7)?,
                last_scan_at: r.get(8)?,
                created_at: r.get(9)?,
            })
        })?;
        rows.map(|row| row?.into_source(&decks)).collect()
    }

    /// A source by name or short id (CLI-007).
    pub fn find_source(&self, workspace: Id, key: &str) -> Result<Source> {
        let key = key.trim();
        let sources = self.list_sources(workspace)?;
        if let Some(s) = sources.iter().find(|s| s.name == key) {
            return Ok(s.clone());
        }
        let matches: Vec<&Source> = sources
            .iter()
            .filter(|s| s.short_id == key || s.id == key)
            .collect();
        match matches.as_slice() {
            [one] => Ok((*one).clone()),
            [] => Err(Error::NotFound(format!(
                "no source '{key}'; see `addisco source list`"
            ))),
            many => Err(Error::conflict(format!(
                "'{key}' matches several sources: {}",
                many.iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))),
        }
    }

    /// Unregister a source. Cards already created from it, and their review history, are
    /// kept; registering the folder again reconnects them through the ids in the files.
    pub fn remove_source(&mut self, workspace: Id, key: &str) -> Result<Source> {
        let source = self.find_source(workspace, key)?;
        let id: Id = source.id.parse()?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let hlc = w.stamp();
        tx.execute(
            "UPDATE sources SET deleted_at = ?2, updated_at = ?2, hlc = ?3 WHERE id = ?1",
            params![id.as_bytes(), w.now, hlc.to_string()],
        )?;
        oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "source",
            id,
            Op::Delete,
            &serde_json::json!({}),
        )?;
        tx.commit()?;
        Ok(source)
    }

    fn source_by_id(&self, workspace: Id, id: Id) -> Result<Source> {
        let wanted = id.to_string();
        self.list_sources(workspace)?
            .into_iter()
            .find(|s| s.id == wanted)
            .ok_or_else(|| Error::Unexpected("source not found after insert".into()))
    }
}

struct SourceRow {
    id: Vec<u8>,
    name: String,
    kind: String,
    path: String,
    repo: Option<String>,
    deck_id: Vec<u8>,
    tags: String,
    write_ids: bool,
    last_scan_at: Option<i64>,
    created_at: i64,
}

impl SourceRow {
    fn into_source(self, decks: &[Deck]) -> Result<Source> {
        let id = Id::from_slice(&self.id)?;
        let deck_id = Id::from_slice(&self.deck_id)?.to_string();
        let deck = decks
            .iter()
            .find(|d| d.id == deck_id)
            .map(|d| d.path.clone())
            .unwrap_or_default();
        let tags: Vec<String> = serde_json::from_str(&self.tags)
            .map_err(|e| Error::Unexpected(format!("invalid source tags: {e}")))?;
        Ok(Source {
            id: id.to_string(),
            short_id: id.short(),
            name: self.name,
            kind: SourceKind::parse(&self.kind)?,
            path: self.path,
            repo: self.repo,
            deck_id,
            deck,
            tags,
            write_ids: self.write_ids,
            last_scan_at: self.last_scan_at,
            created_at: self.created_at,
        })
    }
}

fn validate_name(raw: &str) -> Result<String> {
    let name = raw.trim();
    if name.is_empty() {
        return Err(Error::validation("source name must not be empty"));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(Error::validation(format!(
            "source name '{name}' is longer than {MAX_NAME_LEN} characters"
        )));
    }
    if name.chars().any(char::is_control) {
        return Err(Error::validation(
            "source name must not contain control characters",
        ));
    }
    Ok(name.to_string())
}

/// The absolute, symlink-free form of an existing directory.
fn canonical_dir(path: &Path) -> Result<PathBuf> {
    let canonical = std::fs::canonicalize(path).map_err(|e| {
        Error::validation(format!("can't use '{}' as a source: {e}", path.display()))
    })?;
    if !canonical.is_dir() {
        return Err(Error::validation(format!(
            "'{}' is not a folder; a source is a folder or repository",
            path.display()
        )));
    }
    Ok(canonical)
}

/// Path text as users expect it. On Windows, `canonicalize` adds a `\\?\` prefix to
/// drive paths; drop it when the rest is an ordinary drive path.
fn display_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => rest.to_string(),
        _ => text.into_owned(),
    }
}

/// Whether `path` is inside a Git work tree, and the repository's root commit if it has
/// one. Uses the system `git` (08 §3); without it every source is a plain folder.
fn git_identity(path: &Path) -> (SourceKind, Option<String>) {
    let git = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    if git(&["rev-parse", "--is-inside-work-tree"]).as_deref() != Some("true") {
        return (SourceKind::Folder, None);
    }
    // A history can have several roots (merged repositories); the smallest hash is a
    // stable choice on every clone.
    let root = git(&["rev-list", "--max-parents=0", "HEAD"]).and_then(|roots| {
        roots
            .lines()
            .map(str::trim)
            .filter(|l| l.len() == 40 && l.bytes().all(|b| b.is_ascii_hexdigit()))
            .min()
            .map(str::to_string)
    });
    (SourceKind::Repo, root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_trimmed_and_validated() {
        assert_eq!(validate_name("  app ").unwrap(), "app");
        assert!(validate_name("   ").is_err());
        assert!(validate_name("a\u{7}b").is_err());
        assert!(validate_name(&"x".repeat(101)).is_err());
    }

    #[test]
    fn windows_verbatim_prefix_is_dropped_for_drive_paths() {
        assert_eq!(display_path(Path::new(r"\\?\C:\code\app")), r"C:\code\app");
        assert_eq!(
            display_path(Path::new(r"\\?\UNC\server\share")),
            r"\\?\UNC\server\share"
        );
        assert_eq!(display_path(Path::new("/home/me/app")), "/home/me/app");
    }
}
