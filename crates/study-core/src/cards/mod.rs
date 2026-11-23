// SPDX-License-Identifier: AGPL-3.0-or-later
//! Decks, facts and cards (SRS-001, SRS-002, SRS-007; docs/specs/09-data-model.md §3).
//!
//! Scheduling (FSRS, review logs) arrives in `feat/core-fsrs`; here new cards are created
//! in state `new` with no due date.

pub mod card_type;
pub mod render;

use std::collections::{BTreeMap, HashMap};

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

pub use card_type::CardType;

use crate::Core;
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::write::Writer;

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
    /// `new`, `learning`, `review` or `relearning`.
    pub state: String,
    /// When the card is next due (UTC ms); `None` for new cards.
    pub due: Option<i64>,
    pub reps: u32,
    pub lapses: u32,
    /// FSRS memory state (cache of the review log, SRS-006).
    pub stability: Option<f64>,
    pub difficulty: Option<f64>,
    pub last_review_at: Option<i64>,
}

/// Columns read by [`card_from_row`], in order.
pub(crate) const CARD_COLUMNS: &str =
    "id, fact_id, ord, deck_id, state, due, reps, lapses, stability, difficulty, last_review_at";

fn id_column(r: &rusqlite::Row<'_>, i: usize) -> rusqlite::Result<Id> {
    let bytes: Vec<u8> = r.get(i)?;
    Id::from_slice(&bytes).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            i,
            rusqlite::types::Type::Blob,
            Box::new(std::io::Error::other(e.to_string())),
        )
    })
}

/// Build a [`Card`] from a row selected with [`CARD_COLUMNS`].
pub(crate) fn card_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Card> {
    let id = id_column(r, 0)?;
    Ok(Card {
        id: id.to_string(),
        short_id: id.short(),
        fact_id: id_column(r, 1)?.to_string(),
        ord: r.get(2)?,
        deck_id: id_column(r, 3)?.to_string(),
        state: r.get(4)?,
        due: r.get(5)?,
        reps: r.get(6)?,
        lapses: r.get(7)?,
        stability: r.get(8)?,
        difficulty: r.get(9)?,
        last_review_at: r.get(10)?,
    })
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Fact {
    pub id: String,
    pub short_id: String,
    pub deck_id: String,
    pub card_type: String,
    pub fields: BTreeMap<String, String>,
    /// Tag paths (WS-003), sorted.
    pub tags: Vec<String>,
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
    /// Tags to apply (WS-003), e.g. `bio::cells`; created on first use.
    pub tags: Vec<String>,
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
                "no workspace yet; run `keepjay init` first".into(),
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
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (leaf, _) = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        }
        .ensure_deck_path(workspace, &parts)?;
        tx.commit()?;
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
                "no deck '{}'; create it with `keepjay deck create \"{}\"`",
                new.deck, new.deck
            ))
        })?;
        let ords = new.card_type.card_ords(&new.fields)?;
        let mut tags = crate::tags::normalize_all(new.tags.iter().map(String::as_str))?;
        let first_field = new.card_type.fields()[0];
        let sort_field = new
            .fields
            .get(first_field)
            .map(|v| v.trim().to_string())
            .unwrap_or_default();

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

        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let fact_id = w.insert_fact(workspace, deck, new.card_type, &new.fields, None)?;
        w.tag_with(workspace, crate::tags::Taggable::Fact, fact_id, &tags)?;
        let mut cards = Vec::with_capacity(ords.len());
        for ord in ords {
            let card_id = w.insert_card(fact_id, deck, ord, false)?;
            cards.push(Card {
                id: card_id.to_string(),
                short_id: card_id.short(),
                fact_id: fact_id.to_string(),
                ord,
                deck_id: deck.to_string(),
                state: "new".into(),
                due: None,
                reps: 0,
                lapses: 0,
                stability: None,
                difficulty: None,
                last_review_at: None,
            });
        }
        tx.commit()?;
        if !tags.is_empty() {
            // Existing tags keep their first spelling.
            tags = self.fact_tags(fact_id)?;
        }

        Ok(Fact {
            id: fact_id.to_string(),
            short_id: fact_id.short(),
            deck_id: deck.to_string(),
            card_type: new.card_type.id().to_string(),
            fields: new.fields,
            tags,
            cards,
        })
    }

    /// Live cards directly in a deck, oldest first.
    pub fn list_cards(&self, deck: Id) -> Result<Vec<Card>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {CARD_COLUMNS} FROM cards
             WHERE deck_id = ?1 AND deleted_at IS NULL ORDER BY id"
        ))?;
        let cards = stmt
            .query_map([deck.as_bytes()], card_from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(cards)
    }
}
