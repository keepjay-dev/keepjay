// SPDX-License-Identifier: AGPL-3.0-or-later
//! Addisco shared core.
//!
//! Every shell (CLI, then the VS Code extension, mobile and web) talks to the core through
//! [`Core`] (docs/specs/08-architecture.md §6). It covers: data-dir resolution
//! (09 §8), SQLite storage with forward-only migrations (08 §7, 09 §9), sync-ready IDs and
//! clocks (D-009), the change log used for live refresh (08 §8), workspaces, decks and
//! cards, FSRS scheduling, Anki import, and sources (13).

pub mod cards;
pub mod domain;
pub mod error;
pub mod io;
pub mod media;
pub mod paths;
pub mod sources;
pub mod srs;
pub mod storage;
pub mod tags;
mod write;

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

pub use cards::{Card, CardType, Deck, Fact, NewFact};
pub use domain::{Hlc, Id};
pub use error::{Error, ErrorKind, Result};
pub use io::{AnkiImportOptions, AnkiImportSummary};
pub use media::MediaFile;
pub use paths::Paths;
pub use sources::{NewSource, ScanReport, Source, SourceKind, StaleCard};
pub use srs::{
    AnswerOutcome, DueCounts, IntervalPreview, QueueKind, QueuedCard, Rating, StudyDay, Today,
};

/// Version of this core library (shared by the CLI and desktop app).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Handle to one profile's data directory and database.
pub struct Core {
    paths: Paths,
    conn: Connection,
    device_id: Id,
    clock: Hlc,
}

/// Summary shown by `addisco status` and the desktop app.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Status {
    pub core_version: String,
    pub data_dir: String,
    pub db_path: String,
    pub schema_version: u32,
    pub device_id: String,
    pub workspaces: u64,
}

/// A workspace (WS-001).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Workspace {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub created_at: i64,
}

/// Result of `addisco doctor` (CLI-012).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DoctorReport {
    pub integrity: String,
    pub journal_mode: String,
    pub schema_version: u32,
    pub supported_schema_version: u32,
    pub ok: bool,
}

impl Core {
    /// Open (and if needed create and migrate) the database in `paths.data_dir`.
    pub fn open(paths: Paths) -> Result<Self> {
        std::fs::create_dir_all(&paths.data_dir)?;
        let device_id = storage::device::load_or_create(&paths.data_dir)?;
        let conn = storage::open(&paths)?;
        let core = Core {
            paths,
            conn,
            device_id,
            clock: Hlc::default(),
        };
        core.touch_device()?;
        Ok(core)
    }

    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    pub fn device_id(&self) -> Id {
        self.device_id
    }

    pub fn status(&self) -> Result<Status> {
        let workspaces: u64 = self.conn.query_row(
            "SELECT COUNT(*) FROM workspaces WHERE deleted_at IS NULL",
            [],
            |r| r.get(0),
        )?;
        Ok(Status {
            core_version: VERSION.to_string(),
            data_dir: self.paths.data_dir.display().to_string(),
            db_path: self.paths.db_path().display().to_string(),
            schema_version: storage::schema_version(&self.conn)?,
            device_id: self.device_id.to_string(),
            workspaces,
        })
    }

    /// Create a workspace (WS-001). Names must be non-empty and unique among live workspaces.
    pub fn create_workspace(&mut self, name: &str) -> Result<Workspace> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::validation("workspace name must not be empty"));
        }
        let id = Id::new();
        let now = domain::now_ms();
        let hlc = self.clock.tick(now, self.device_id);
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let exists: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM workspaces WHERE name = ?1 AND deleted_at IS NULL",
                [name],
                |r| r.get(0),
            )
            .optional()?;
        if exists.is_some() {
            return Err(Error::conflict(format!(
                "a workspace named '{name}' already exists"
            )));
        }
        tx.execute(
            "INSERT INTO workspaces (id, name, sort, created_at, updated_at, hlc, origin_device)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(sort), 0) + 1 FROM workspaces), ?3, ?3, ?4, ?5)",
            params![
                id.as_bytes(),
                name,
                now,
                hlc.to_string(),
                self.device_id.as_bytes()
            ],
        )?;
        storage::oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "workspace",
            id,
            storage::oplog::Op::Insert,
            &serde_json::json!({ "name": name }),
        )?;
        tx.commit()?;
        Ok(Workspace {
            id: id.to_string(),
            short_id: id.short(),
            name: name.to_string(),
            created_at: now,
        })
    }

    pub fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at FROM workspaces WHERE deleted_at IS NULL ORDER BY sort",
        )?;
        let rows = stmt.query_map([], |r| {
            let bytes: Vec<u8> = r.get(0)?;
            Ok((bytes, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
        })?;
        rows.map(|row| {
            let (bytes, name, created_at) = row?;
            let id = Id::from_slice(&bytes)?;
            Ok(Workspace {
                id: id.to_string(),
                short_id: id.short(),
                name,
                created_at,
            })
        })
        .collect()
    }

    /// Idempotent first-run setup (`addisco init`): ensures at least one workspace exists.
    /// Presets arrive in M1 (03 §4); for now this creates an empty workspace.
    pub fn init(&mut self, workspace_name: &str) -> Result<Workspace> {
        if let Some(first) = self.list_workspaces()?.into_iter().next() {
            return Ok(first);
        }
        self.create_workspace(workspace_name)
    }

    pub fn doctor(&self) -> Result<DoctorReport> {
        let integrity: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
        let journal_mode: String = self
            .conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        let schema_version = storage::schema_version(&self.conn)?;
        let ok = integrity == "ok" && journal_mode.eq_ignore_ascii_case("wal");
        Ok(DoctorReport {
            integrity,
            journal_mode,
            schema_version,
            supported_schema_version: storage::SUPPORTED_SCHEMA_VERSION,
            ok,
        })
    }

    /// SQLite's `data_version`: changes when *another* connection commits (08 §8).
    pub fn data_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("PRAGMA data_version", [], |r| r.get(0))?)
    }

    /// Highest change-log sequence number; shells remember it to fetch changes since.
    pub fn last_change_seq(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COALESCE(MAX(seq), 0) FROM oplog", [], |r| r.get(0))?)
    }

    /// Changes committed after `since_seq` (by any process), oldest first.
    pub fn changes_since(&self, since_seq: i64) -> Result<Vec<storage::oplog::Change>> {
        storage::oplog::since(&self.conn, since_seq)
    }

    fn touch_device(&self) -> Result<()> {
        let now = domain::now_ms();
        self.conn.execute(
            "INSERT INTO devices (id, name, platform, created_at, last_seen_at)
             VALUES (?1, ?2, ?3, ?4, ?4)
             ON CONFLICT(id) DO UPDATE SET last_seen_at = excluded.last_seen_at",
            params![
                self.device_id.as_bytes(),
                storage::device::host_name(),
                std::env::consts::OS,
                now
            ],
        )?;
        Ok(())
    }
}
