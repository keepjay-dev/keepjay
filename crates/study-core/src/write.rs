// SPDX-License-Identifier: AGPL-3.0-or-later
//! Low-level row writers shared by commands and importers. Each insert or update also
//! appends to the change log (D-009), inside the caller's transaction. Keeping the SQL in
//! one place means `add_fact`, `answer_card` and the Anki importer can't drift apart.

use std::collections::BTreeMap;

use rusqlite::{Connection, OptionalExtension, params};

use crate::cards::CardType;
use crate::domain::{Hlc, HlcStamp, Id};
use crate::error::Result;
use crate::srs::{Memory, Rating};
use crate::storage::oplog::{self, Op};

pub(crate) struct Writer<'a> {
    pub conn: &'a Connection,
    pub clock: &'a mut Hlc,
    pub device: Id,
    /// Wall-clock time used for created/updated timestamps and HLC stamps.
    pub now: i64,
}

/// A card's scheduling cache, written in full (SRS-006).
pub(crate) struct Schedule<'s> {
    pub state: &'s str,
    pub due: Option<i64>,
    pub memory: Option<Memory>,
    pub reps: u32,
    pub lapses: u32,
    pub last_review_at: Option<i64>,
}

/// One answer for the review log.
pub(crate) struct ReviewRow<'s> {
    pub card: Id,
    pub reviewed_at: i64,
    pub rating: Rating,
    pub answer_ms: i64,
    pub state_before: &'s str,
    pub memory_before: Option<Memory>,
    pub memory_after: Memory,
    pub interval_days: f64,
    pub due_after: i64,
    pub scheduler: &'s str,
}

impl Writer<'_> {
    pub(crate) fn stamp(&mut self) -> HlcStamp {
        self.clock.tick(self.now, self.device)
    }

    /// Find or create each deck along `parts` (`["Biology", "Cells"]`). Returns the leaf id
    /// and the full paths of decks that were created.
    pub fn ensure_deck_path(&mut self, workspace: Id, parts: &[&str]) -> Result<(Id, Vec<String>)> {
        let mut parent: Option<Id> = None;
        let mut created = Vec::new();
        for (i, part) in parts.iter().enumerate() {
            let existing: Option<Vec<u8>> = self
                .conn
                .prepare_cached(
                    "SELECT id FROM decks
                     WHERE workspace_id = ?1 AND parent_id IS ?2 AND name = ?3
                       AND deleted_at IS NULL",
                )?
                .query_row(
                    params![
                        workspace.as_bytes(),
                        parent.as_ref().map(|p| p.as_bytes().to_vec()),
                        part
                    ],
                    |r| r.get(0),
                )
                .optional()?;
            let id = match existing {
                Some(bytes) => Id::from_slice(&bytes)?,
                None => {
                    let id = Id::new();
                    let hlc = self.stamp();
                    self.conn
                        .prepare_cached(
                            "INSERT INTO decks (id, workspace_id, parent_id, name, sort,
                                            created_at, updated_at, hlc, origin_device)
                         VALUES (?1, ?2, ?3, ?4,
                                 (SELECT COALESCE(MAX(sort), 0) + 1 FROM decks), ?5, ?5, ?6, ?7)",
                        )?
                        .execute(params![
                            id.as_bytes(),
                            workspace.as_bytes(),
                            parent.as_ref().map(|p| p.as_bytes().to_vec()),
                            part,
                            self.now,
                            hlc.to_string(),
                            self.device.as_bytes()
                        ])?;
                    oplog::append(
                        self.conn,
                        &hlc,
                        self.device,
                        "deck",
                        id,
                        Op::Insert,
                        &serde_json::json!({ "name": part }),
                    )?;
                    created.push(parts[..=i].join(crate::cards::DECK_SEPARATOR));
                    id
                }
            };
            parent = Some(id);
        }
        Ok((parent.expect("deck path has at least one part"), created))
    }

    pub fn insert_fact(
        &mut self,
        workspace: Id,
        deck: Id,
        card_type: CardType,
        fields: &BTreeMap<String, String>,
        source: Option<&str>,
    ) -> Result<Id> {
        self.insert_fact_with_id(Id::new(), workspace, deck, card_type, fields, source)
    }

    /// Like [`Writer::insert_fact`], with a caller-chosen id (ids written in source files).
    pub fn insert_fact_with_id(
        &mut self,
        id: Id,
        workspace: Id,
        deck: Id,
        card_type: CardType,
        fields: &BTreeMap<String, String>,
        source: Option<&str>,
    ) -> Result<Id> {
        let sort_field = fields
            .get(card_type.fields()[0])
            .map(|v| v.trim().to_string())
            .unwrap_or_default();
        let fields_json = serde_json::to_string(fields).map_err(|e| {
            crate::error::Error::Unexpected(format!("could not encode fields: {e}"))
        })?;
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "INSERT INTO facts (id, workspace_id, deck_id, card_type, fields, sort_field,
                                    source, created_at, updated_at, hlc, origin_device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?8, ?9, ?10)",
            )?
            .execute(params![
                id.as_bytes(),
                workspace.as_bytes(),
                deck.as_bytes(),
                card_type.id(),
                fields_json,
                sort_field,
                source,
                self.now,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "fact",
            id,
            Op::Insert,
            &serde_json::json!({ "card_type": card_type.id(), "fields": fields }),
        )?;
        Ok(id)
    }

    pub fn insert_card(&mut self, fact: Id, deck: Id, ord: u32, suspended: bool) -> Result<Id> {
        let id = Id::new();
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "INSERT INTO cards (id, fact_id, ord, deck_id, suspended, created_at, updated_at,
                                    hlc, origin_device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8)",
            )?
            .execute(params![
                id.as_bytes(),
                fact.as_bytes(),
                ord,
                deck.as_bytes(),
                suspended,
                self.now,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "card",
            id,
            Op::Insert,
            &serde_json::json!({ "fact_id": fact.to_string(), "ord": ord }),
        )?;
        Ok(id)
    }

    pub fn insert_review(&mut self, r: &ReviewRow<'_>) -> Result<Id> {
        let id = Id::new();
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "INSERT INTO review_logs (id, card_id, reviewed_at, rating, answer_ms,
                     state_before, stability_before, difficulty_before, stability_fast_before,
                     stability_after, difficulty_after, stability_fast_after,
                     interval_days, due_after, scheduler, hlc, device_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            )?
            .execute(params![
                id.as_bytes(),
                r.card.as_bytes(),
                r.reviewed_at,
                r.rating as u8,
                r.answer_ms.max(0),
                r.state_before,
                r.memory_before.map(|m| m.stability),
                r.memory_before.map(|m| m.difficulty),
                r.memory_before.map(|m| m.stability_fast),
                r.memory_after.stability,
                r.memory_after.difficulty,
                r.memory_after.stability_fast,
                r.interval_days,
                r.due_after,
                r.scheduler,
                hlc.to_string(),
                self.device.as_bytes()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "review_log",
            id,
            Op::Insert,
            &serde_json::json!({ "card_id": r.card.to_string(), "rating": r.rating as u8 }),
        )?;
        Ok(id)
    }

    pub fn update_schedule(&mut self, card: Id, s: &Schedule<'_>) -> Result<()> {
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "UPDATE cards SET state = ?2, due = ?3, stability = ?4, difficulty = ?5,
                     stability_fast = ?6, reps = ?7, lapses = ?8, last_review_at = ?9,
                     updated_at = ?10, hlc = ?11
                 WHERE id = ?1",
            )?
            .execute(params![
                card.as_bytes(),
                s.state,
                s.due,
                s.memory.map(|m| m.stability),
                s.memory.map(|m| m.difficulty),
                s.memory.map(|m| m.stability_fast),
                s.reps,
                s.lapses,
                s.last_review_at,
                self.now,
                hlc.to_string()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "card",
            card,
            Op::Update,
            &serde_json::json!({ "state": s.state, "due": s.due }),
        )?;
        Ok(())
    }

    /// Replace a fact's card type, fields and provenance (cards from sources, SRC-005).
    pub fn update_fact(
        &mut self,
        fact: Id,
        card_type: CardType,
        fields: &BTreeMap<String, String>,
        source: Option<&str>,
    ) -> Result<()> {
        let sort_field = fields
            .get(card_type.fields()[0])
            .map(|v| v.trim().to_string())
            .unwrap_or_default();
        let fields_json = serde_json::to_string(fields).map_err(|e| {
            crate::error::Error::Unexpected(format!("could not encode fields: {e}"))
        })?;
        let hlc = self.stamp();
        self.conn
            .prepare_cached(
                "UPDATE facts SET card_type = ?2, fields = ?3, sort_field = ?4, source = ?5,
                                  updated_at = ?6, hlc = ?7
                 WHERE id = ?1",
            )?
            .execute(params![
                fact.as_bytes(),
                card_type.id(),
                fields_json,
                sort_field,
                source,
                self.now,
                hlc.to_string()
            ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "fact",
            fact,
            Op::Update,
            &serde_json::json!({ "card_type": card_type.id(), "fields": fields }),
        )?;
        Ok(())
    }

    /// Soft-delete (`deleted = true`) or restore a row (D-009: synced rows are never
    /// hard-deleted).
    pub fn set_deleted(&mut self, entity: Entity, id: Id, deleted: bool) -> Result<()> {
        let hlc = self.stamp();
        let sql = match entity {
            Entity::Fact => {
                "UPDATE facts SET deleted_at = ?2, updated_at = ?3, hlc = ?4 WHERE id = ?1"
            }
            Entity::Card => {
                "UPDATE cards SET deleted_at = ?2, updated_at = ?3, hlc = ?4 WHERE id = ?1"
            }
        };
        self.conn.prepare_cached(sql)?.execute(params![
            id.as_bytes(),
            deleted.then_some(self.now),
            self.now,
            hlc.to_string()
        ])?;
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            entity.name(),
            id,
            if deleted { Op::Delete } else { Op::Update },
            &serde_json::json!({ "deleted": deleted }),
        )?;
        Ok(())
    }

    /// Insert or update the anchor of a fact (SRC-004).
    pub fn upsert_anchor(&mut self, a: &AnchorRow<'_>) -> Result<Id> {
        let hlc = self.stamp();
        let existing: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT id FROM anchors WHERE fact_id = ?1 AND deleted_at IS NULL",
                [a.fact.as_bytes()],
                |r| r.get(0),
            )
            .optional()?;
        let (id, op) = match existing {
            Some(bytes) => {
                let id = Id::from_slice(&bytes)?;
                self.conn
                    .prepare_cached(
                        "UPDATE anchors SET source_id = ?2, repo = ?3, path = ?4, line_start = ?5,
                                            line_end = ?6, code_hash = ?7, stale_since = ?8,
                                            updated_at = ?9, hlc = ?10
                         WHERE id = ?1",
                    )?
                    .execute(params![
                        id.as_bytes(),
                        a.source.as_bytes(),
                        a.repo,
                        a.path,
                        a.line_start,
                        a.line_end,
                        a.code_hash,
                        a.stale_since,
                        self.now,
                        hlc.to_string()
                    ])?;
                (id, Op::Update)
            }
            None => {
                let id = Id::new();
                self.conn
                    .prepare_cached(
                        "INSERT INTO anchors (id, fact_id, source_id, repo, path, line_start,
                                              line_end, code_hash, stale_since, created_at,
                                              updated_at, hlc, origin_device)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, ?11, ?12)",
                    )?
                    .execute(params![
                        id.as_bytes(),
                        a.fact.as_bytes(),
                        a.source.as_bytes(),
                        a.repo,
                        a.path,
                        a.line_start,
                        a.line_end,
                        a.code_hash,
                        a.stale_since,
                        self.now,
                        hlc.to_string(),
                        self.device.as_bytes()
                    ])?;
                (id, Op::Insert)
            }
        };
        oplog::append(
            self.conn,
            &hlc,
            self.device,
            "anchor",
            id,
            op,
            &serde_json::json!({ "fact_id": a.fact.to_string(), "path": a.path,
                                 "stale": a.stale_since.is_some() }),
        )?;
        Ok(id)
    }
}

/// Rows that can be soft-deleted through [`Writer::set_deleted`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Entity {
    Fact,
    Card,
}

impl Entity {
    fn name(self) -> &'static str {
        match self {
            Entity::Fact => "fact",
            Entity::Card => "card",
        }
    }
}

/// An anchor as written (SRC-004).
pub(crate) struct AnchorRow<'s> {
    pub fact: Id,
    pub source: Id,
    pub repo: &'s str,
    pub path: &'s str,
    pub line_start: u32,
    pub line_end: u32,
    pub code_hash: &'s str,
    pub stale_since: Option<i64>,
}
