// SPDX-License-Identifier: AGPL-3.0-or-later
//! Decks, facts and cards (SRS-001, SRS-002, SRS-007; docs/specs/09-data-model.md §3).
//!
//! Scheduling (FSRS, review logs) arrives in `feat/core-fsrs`; here new cards are created
//! in state `new` with no due date.

pub mod card_type;

use std::collections::{BTreeMap, HashMap};

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

pub use card_type::CardType;

use crate::Core;
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::storage::oplog::{self, Op};

/// Raw deck row: id, parent id, name, direct card count.
type DeckRow = (Vec<u8>, Option<Vec<u8>>, String, u64);

/// Separator for deck paths, e.g. `Biology::Cells` (as in Anki).
pub const DECK_SEPARATOR: &str = "::";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Deck {
    pub id: String,
    pub short_id: String,
    /// Leaf name, e.g. `Cells`.
    pub name: String,
    /// Full path, e.g. `Biology::Cells`.
    pub path: String,
    pub parent_id: Option<String>,
    /// Live cards directly in this deck (not counting subdecks).
    pub cards: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Card {
    pub id: String,
    pub short_id: String,
    pub fact_id: String,
    pub ord: u32,
    pub deck_id: String,
    pub state: String,
    pub due: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fact {
    pub id: String,
    pub short_id: String,
    pub deck_id: String,
    pub card_type: String,
    pub fields: BTreeMap<String, String>,
    pub cards: Vec<Card>,
}

/// Input for [`Core::add_fact`].
#[derive(Debug, Clone)]
pub struct NewFact {
    /// Deck path, e.g. `Biology::Cells`. The deck must exist.
    pub deck: String,
    pub card_type: CardType,
    pub fields: BTreeMap<String, String>,
    /// Skip duplicate detection (SRS-007).
    pub allow_duplicate: bool,
}

/// Split and validate a deck path.
fn deck_parts(path: &str) -> Result<Vec<&str>> {
    let parts: Vec<&str> = path.split(DECK_SEPARATOR).map(str::trim).collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(Error::validation(format!(
            "invalid deck path '{path}': names must not be empty (use 'Parent{DECK_SEPARATOR}Child')"
        )));
    }
    Ok(parts)
}

impl Core {
    /// Workspace by name, or the first workspace when `name` is `None` (WS-004).
    pub fn resolve_workspace(&self, name: Option<&str>) -> Result<Id> {
        let found: Option<Vec<u8>> = match name {
            Some(n) => self
                .conn
                .query_row(
                    "SELECT id FROM workspaces WHERE name = ?1 AND deleted_at IS NULL",
                    [n.trim()],
                    |r| r.get(0),
                )
                .optional()?,
            None => self
                .conn
                .query_row(
                    "SELECT id FROM workspaces WHERE deleted_at IS NULL ORDER BY sort LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .optional()?,
        };
        match (found, name) {
            (Some(bytes), _) => Id::from_slice(&bytes),
            (None, Some(n)) => Err(Error::NotFound(format!("no workspace named '{n}'"))),
            (None, None) => Err(Error::NotFound(
                "no workspace yet; run `addisco init` first".into(),
            )),
        }
    }

    /// Find a deck by path in a workspace.
    pub fn find_deck(&self, workspace: Id, path: &str) -> Result<Option<Id>> {
        let mut parent: Option<Id> = None;
        for part in deck_parts(path)? {
            let id: Option<Vec<u8>> = self
                .conn
                .query_row(
                    "SELECT id FROM decks
                     WHERE workspace_id = ?1 AND parent_id IS ?2 AND name = ?3
                       AND deleted_at IS NULL",
                    params![
                        workspace.as_bytes(),
                        parent.as_ref().map(|p| p.as_bytes().to_vec()),
                        part
                    ],
                    |r| r.get(0),
                )
                .optional()?;
            match id {
                Some(bytes) => parent = Some(Id::from_slice(&bytes)?),
                None => return Ok(None),
            }
        }
        Ok(parent)
    }

    /// Create a deck (SRS-001). Missing parents in the path are created too, like
    /// `mkdir -p`. It's a conflict if the full path already exists.
    pub fn create_deck(&mut self, workspace: Id, path: &str) -> Result<Deck> {
        let parts = deck_parts(path)?;
        if self.find_deck(workspace, path)?.is_some() {
            return Err(Error::conflict(format!("deck '{path}' already exists")));
        }
        let now = domain::now_ms();
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut parent: Option<Id> = None;
        for part in &parts {
            let existing: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT id FROM decks
                     WHERE workspace_id = ?1 AND parent_id IS ?2 AND name = ?3
                       AND deleted_at IS NULL",
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
                    let hlc = self.clock.tick(now, self.device_id);
                    tx.execute(
                        "INSERT INTO decks (id, workspace_id, parent_id, name, sort,
                                            created_at, updated_at, hlc, origin_device)
                         VALUES (?1, ?2, ?3, ?4,
                                 (SELECT COALESCE(MAX(sort), 0) + 1 FROM decks), ?5, ?5, ?6, ?7)",
                        params![
                            id.as_bytes(),
                            workspace.as_bytes(),
                            parent.as_ref().map(|p| p.as_bytes().to_vec()),
                            part,
                            now,
                            hlc.to_string(),
                            self.device_id.as_bytes()
                        ],
                    )?;
                    oplog::append(
                        &tx,
                        &hlc,
                        self.device_id,
                        "deck",
                        id,
                        Op::Insert,
                        &serde_json::json!({ "name": part }),
                    )?;
                    id
                }
            };
            parent = Some(id);
        }
        tx.commit()?;
        let leaf = parent.expect("deck path has at least one part");
        self.list_decks(workspace)?
            .into_iter()
            .find(|d| d.id == leaf.to_string())
            .ok_or_else(|| Error::Unexpected("created deck not found".into()))
    }

    /// All live decks in a workspace, sorted by path, with direct card counts.
    pub fn list_decks(&self, workspace: Id) -> Result<Vec<Deck>> {
        let mut stmt = self.conn.prepare(
            "SELECT d.id, d.parent_id, d.name,
                    (SELECT COUNT(*) FROM cards c WHERE c.deck_id = d.id AND c.deleted_at IS NULL)
             FROM decks d WHERE d.workspace_id = ?1 AND d.deleted_at IS NULL",
        )?;
        let rows: Vec<DeckRow> = stmt
            .query_map([workspace.as_bytes()], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })?
            .collect::<rusqlite::Result<_>>()?;
        let by_id: HashMap<Vec<u8>, (Option<Vec<u8>>, String)> = rows
            .iter()
            .map(|(id, parent, name, _)| (id.clone(), (parent.clone(), name.clone())))
            .collect();
        let path_of = |id: &Vec<u8>| -> String {
            let mut names = Vec::new();
            let mut cur = Some(id.clone());
            // Bounded walk guards against a corrupt cycle.
            for _ in 0..64 {
                let Some(c) = cur else { break };
                let Some((parent, name)) = by_id.get(&c) else {
                    break;
                };
                names.push(name.clone());
                cur = parent.clone();
            }
            names.reverse();
            names.join(DECK_SEPARATOR)
        };
        let mut decks = rows
            .iter()
            .map(|(id, parent, name, cards)| {
                let deck_id = Id::from_slice(id)?;
                Ok(Deck {
                    id: deck_id.to_string(),
                    short_id: deck_id.short(),
                    name: name.clone(),
                    path: path_of(id),
                    parent_id: parent
                        .as_ref()
                        .map(|p| Id::from_slice(p).map(|i| i.to_string()))
                        .transpose()?,
                    cards: *cards,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        decks.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(decks)
    }

    /// Add a fact and generate its cards (SRS-002), with duplicate detection (SRS-007).
    pub fn add_fact(&mut self, workspace: Id, new: NewFact) -> Result<Fact> {
        let deck = self.find_deck(workspace, &new.deck)?.ok_or_else(|| {
            Error::NotFound(format!(
                "no deck '{}'; create it with `addisco deck create \"{}\"`",
                new.deck, new.deck
            ))
        })?;
        let ords = new.card_type.card_ords(&new.fields)?;
        let first_field = new.card_type.fields()[0];
        let sort_field = new
            .fields
            .get(first_field)
            .map(|v| v.trim().to_string())
            .unwrap_or_default();

        let now = domain::now_ms();
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        if !new.allow_duplicate {
            let dup: Option<i64> = tx
                .query_row(
                    "SELECT 1 FROM facts WHERE deck_id = ?1 AND card_type = ?2
                       AND sort_field = ?3 AND deleted_at IS NULL",
                    params![deck.as_bytes(), new.card_type.id(), sort_field],
                    |r| r.get(0),
                )
                .optional()?;
            if dup.is_some() {
                return Err(Error::conflict(format!(
                    "duplicate: '{}' already has a card with {first_field} \"{sort_field}\" (allow it with --allow-duplicate)",
                    new.deck
                )));
            }
        }

        let fact_id = Id::new();
        let fields_json = serde_json::to_string(&new.fields)
            .map_err(|e| Error::Unexpected(format!("could not encode fields: {e}")))?;
        let hlc = self.clock.tick(now, self.device_id);
        tx.execute(
            "INSERT INTO facts (id, workspace_id, deck_id, card_type, fields, sort_field,
                                created_at, updated_at, hlc, origin_device)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7, ?8, ?9)",
            params![
                fact_id.as_bytes(),
                workspace.as_bytes(),
                deck.as_bytes(),
                new.card_type.id(),
                fields_json,
                sort_field,
                now,
                hlc.to_string(),
                self.device_id.as_bytes()
            ],
        )?;
        oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "fact",
            fact_id,
            Op::Insert,
            &serde_json::json!({ "card_type": new.card_type.id(), "fields": new.fields }),
        )?;

        let mut cards = Vec::with_capacity(ords.len());
        for ord in ords {
            let card_id = Id::new();
            let hlc = self.clock.tick(now, self.device_id);
            tx.execute(
                "INSERT INTO cards (id, fact_id, ord, deck_id, created_at, updated_at,
                                    hlc, origin_device)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7)",
                params![
                    card_id.as_bytes(),
                    fact_id.as_bytes(),
                    ord,
                    deck.as_bytes(),
                    now,
                    hlc.to_string(),
                    self.device_id.as_bytes()
                ],
            )?;
            oplog::append(
                &tx,
                &hlc,
                self.device_id,
                "card",
                card_id,
                Op::Insert,
                &serde_json::json!({ "fact_id": fact_id.to_string(), "ord": ord }),
            )?;
            cards.push(Card {
                id: card_id.to_string(),
                short_id: card_id.short(),
                fact_id: fact_id.to_string(),
                ord,
                deck_id: deck.to_string(),
                state: "new".into(),
                due: None,
            });
        }
        tx.commit()?;

        Ok(Fact {
            id: fact_id.to_string(),
            short_id: fact_id.short(),
            deck_id: deck.to_string(),
            card_type: new.card_type.id().to_string(),
            fields: new.fields,
            cards,
        })
    }

    /// Live cards directly in a deck, oldest first.
    pub fn list_cards(&self, deck: Id) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, fact_id, ord, deck_id, state, due FROM cards
             WHERE deck_id = ?1 AND deleted_at IS NULL ORDER BY id",
        )?;
        let rows = stmt.query_map([deck.as_bytes()], |r| {
            Ok((
                r.get::<_, Vec<u8>>(0)?,
                r.get::<_, Vec<u8>>(1)?,
                r.get::<_, u32>(2)?,
                r.get::<_, Vec<u8>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<i64>>(5)?,
            ))
        })?;
        rows.map(|row| {
            let (id, fact, ord, deck, state, due) = row?;
            let id = Id::from_slice(&id)?;
            Ok(Card {
                id: id.to_string(),
                short_id: id.short(),
                fact_id: Id::from_slice(&fact)?.to_string(),
                ord,
                deck_id: Id::from_slice(&deck)?.to_string(),
                state,
                due,
            })
        })
        .collect()
    }
}
