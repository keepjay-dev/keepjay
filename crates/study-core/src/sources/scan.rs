// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scanning a source (SRC-004, SRC-005, SRC-006; docs/specs/13-code-sources.md §5–§6).
//!
//! Matches the cards found in the files against the source's facts, in one transaction:
//!
//! 1. **By id** (`@card(k22k1m8e)`): update the text if it changed, keeping history;
//!    restore the card if it had been retired; flag it stale if its code changed.
//! 2. **Without an id**, by file and first field (sources that don't write ids, or a
//!    failed write-back).
//! 3. **New cards** get a fact whose short id is the one in the file, or a new one, which is
//!    then written back into the file (after the database commit).
//! 4. **Facts no longer in the files** are retired (soft-deleted, history kept).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

use rusqlite::params;
use serde::Serialize;

use super::parse::{CardStyle, ParseWarning, ParsedCard};
use super::walk;
use super::writeback::{self, IdWrite};
use crate::Core;
use crate::cards::{CardType, DECK_SEPARATOR};
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::storage::oplog::{self, Op};
use crate::tags::Taggable;
use crate::write::{AnchorRow, Entity, Writer};

/// What a scan did (`keepjay scan`).
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct ScanReport {
    pub source: String,
    pub files_considered: u64,
    pub files_parsed: u64,
    pub skipped_large: u64,
    pub skipped_binary: u64,
    pub cards_found: u64,
    pub added: u64,
    pub updated: u64,
    pub unchanged: u64,
    pub restored: u64,
    pub retired: u64,
    /// Cards whose anchored code changed in this scan.
    pub became_stale: u64,
    /// Stale cards in this source after the scan.
    pub stale: u64,
    pub ids_written: u64,
    pub warnings: Vec<ParseWarning>,
    pub elapsed_ms: u64,
}

/// A card whose anchored code changed (SRC-006).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StaleCard {
    /// The fact's short id: the id written in the file.
    pub id: String,
    pub source: String,
    pub path: String,
    pub line: u32,
    pub question: String,
    pub stale_since: i64,
}

/// What `keepjay stale retire` would remove from a file (SRC-006).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RetirePlan {
    pub card: StaleCard,
    /// The lines removed from `card.path`, 1-based and inclusive.
    pub first_line: u32,
    pub last_line: u32,
    /// Those lines as they are now, without line endings.
    pub lines: Vec<String>,
}

/// A retired stale card, and the scan that followed.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Retired {
    #[serde(flatten)]
    pub plan: RetirePlan,
    pub scan: ScanReport,
}

/// A fact that came from a source, as stored.
struct Existing {
    id: Id,
    short: String,
    deck: Id,
    card_type: CardType,
    fields: BTreeMap<String, String>,
    deleted: bool,
    /// Belongs to another (usually removed) source; matched only by id.
    foreign: bool,
    anchor_path: Option<String>,
    code_hash: Option<String>,
    stale_since: Option<i64>,
    /// The anchor row as stored, to skip rewriting it when nothing changed.
    anchor: Option<AnchorState>,
}

/// The stored columns of an anchor that a scan sets.
#[derive(PartialEq)]
struct AnchorState {
    source: Vec<u8>,
    repo: String,
    path: String,
    line_start: u32,
    line_end: u32,
    code_hash: String,
    stale_since: Option<i64>,
}

impl AnchorState {
    fn of(source: Id, repo: &str, card: &ParsedCard, stale_since: Option<i64>) -> Self {
        AnchorState {
            source: source.as_bytes().to_vec(),
            repo: repo.to_string(),
            path: card.path.clone(),
            line_start: card.line_start,
            line_end: card.line_end,
            code_hash: card.code_hash.clone().unwrap_or_default(),
            stale_since,
        }
    }
}

impl Existing {
    fn first_field(&self) -> String {
        self.fields
            .get(self.card_type.fields()[0])
            .map(|v| v.trim().to_string())
            .unwrap_or_default()
    }
}

fn first_field(card_type: CardType, fields: &BTreeMap<String, String>) -> String {
    fields
        .get(card_type.fields()[0])
        .map(|v| v.trim().to_string())
        .unwrap_or_default()
}

impl Core {
    /// Scan one source and bring its cards up to date (SRC-004…006).
    pub fn scan_source(&mut self, workspace: Id, key: &str) -> Result<ScanReport> {
        let started = Instant::now();
        let source = self.find_source(workspace, key)?;
        let source_id: Id = source.id.parse()?;
        let root = PathBuf::from(&source.path);
        if !root.is_dir() {
            return Err(Error::NotFound(format!(
                "the folder for source '{}' is missing: {}",
                source.name, source.path
            )));
        }
        let found = walk::find_cards(&root)?;
        let source_key = format!("src:{}", source.id);
        let repo = source.repo.clone().unwrap_or_else(|| source.id.clone());

        let mut report = ScanReport {
            source: source.name.clone(),
            files_considered: found.files_considered,
            files_parsed: found.files_parsed,
            skipped_large: found.skipped_large,
            skipped_binary: found.skipped_binary,
            cards_found: found.cards.len() as u64,
            warnings: found.warnings,
            ..ScanReport::default()
        };

        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut existing = load_existing(&tx, workspace, &source_key)?;
        let wanted_ids: Vec<&str> = found.cards.iter().filter_map(|c| c.id.as_deref()).collect();
        let known: HashSet<String> = existing.iter().map(|e| e.short.clone()).collect();
        if wanted_ids.iter().any(|id| !known.contains(*id)) {
            existing.extend(load_foreign(&tx, workspace, &source_key)?);
        }
        let by_id: HashMap<String, usize> = existing
            .iter()
            .enumerate()
            .map(|(i, e)| (e.short.clone(), i))
            .collect();
        let by_text: HashMap<(String, String), usize> = existing
            .iter()
            .enumerate()
            .filter(|(_, e)| !e.foreign)
            .filter_map(|(i, e)| Some(((e.anchor_path.clone()?, e.first_field()), i)))
            .collect();
        let mut taken_shorts: Option<HashSet<String>> = None;

        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let now = w.now;
        let mut deck_ids: HashMap<String, Id> = HashMap::new();
        let mut used: HashSet<usize> = HashSet::new();
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut writes: Vec<IdWrite> = Vec::new();

        for card in &found.cards {
            if let Err(e) = card.card_type.card_ords(&card.fields) {
                report.warnings.push(warn(card, e.to_string()));
                continue;
            }
            let tags = card_tags(&source.tags, card, &mut report.warnings);

            // A repeated id (a copied card) is treated as a new card with a new id.
            let mut file_id = card.id.clone();
            if let Some(id) = &file_id
                && !seen_ids.insert(id.clone())
            {
                report.warnings.push(warn(
                    card,
                    format!("id {id} is used by another card too; this one gets a new id"),
                ));
                file_id = None;
            }

            let matched = match &file_id {
                Some(id) => by_id.get(id).copied(),
                None => by_text
                    .get(&(card.path.clone(), first_field(card.card_type, &card.fields)))
                    .copied(),
            }
            .filter(|i| !used.contains(i));

            let fact = match matched {
                Some(i) => {
                    used.insert(i);
                    let e = &existing[i];
                    if e.deleted {
                        w.set_deleted(Entity::Fact, e.id, false)?;
                        report.restored += 1;
                    }
                    let text_changed =
                        e.card_type != card.card_type || e.fields != card.fields || e.foreign;
                    if text_changed {
                        w.update_fact(e.id, card.card_type, &card.fields, Some(&source_key))?;
                        report.updated += 1;
                    } else if !e.deleted {
                        report.unchanged += 1;
                    }
                    if text_changed || e.deleted {
                        sync_cards(&mut w, e.id, e.deck, card.card_type, &card.fields)?;
                    }
                    let stale_since = if text_changed {
                        None // the card was edited: that confirms it
                    } else {
                        match (&e.code_hash, &card.code_hash) {
                            (Some(old), Some(new)) if old != new => {
                                if e.stale_since.is_none() {
                                    report.became_stale += 1;
                                }
                                e.stale_since.or(Some(now))
                            }
                            _ => e.stale_since,
                        }
                    };
                    // Unchanged anchors aren't rewritten, so a rescan adds nothing to
                    // the change log (D-009).
                    let anchor = AnchorState::of(source_id, &repo, card, stale_since);
                    if e.anchor.as_ref() != Some(&anchor) {
                        upsert_anchor(&mut w, e.id, source_id, &repo, card, stale_since)?;
                    }
                    if file_id.is_none() && source.write_ids {
                        writes.push(IdWrite::new(card, &e.short, card.id.clone()));
                    }
                    e.id
                }
                None => {
                    let shorts = match &mut taken_shorts {
                        Some(s) => s,
                        None => taken_shorts.insert(all_shorts(&tx, workspace)?),
                    };
                    let id = match &file_id {
                        Some(short) if !shorts.contains(short) => Id::with_short(short)?,
                        Some(short) => {
                            report.warnings.push(warn(
                                card,
                                format!("id {short} already belongs to another card here; this one gets a new id"),
                            ));
                            file_id = None;
                            Id::new()
                        }
                        None => Id::new(),
                    };
                    shorts.insert(id.short());
                    let deck_path = card.deck.clone().unwrap_or_else(|| source.deck.clone());
                    let deck = match deck_ids.get(&deck_path) {
                        Some(d) => *d,
                        None => {
                            let parts: Vec<&str> =
                                deck_path.split(DECK_SEPARATOR).map(str::trim).collect();
                            let (d, _) = w.ensure_deck_path(workspace, &parts)?;
                            deck_ids.insert(deck_path.clone(), d);
                            d
                        }
                    };
                    w.insert_fact_with_id(
                        id,
                        workspace,
                        deck,
                        card.card_type,
                        &card.fields,
                        Some(&source_key),
                    )?;
                    sync_cards(&mut w, id, deck, card.card_type, &card.fields)?;
                    upsert_anchor(&mut w, id, source_id, &repo, card, None)?;
                    report.added += 1;
                    if file_id.is_none() && (source.write_ids || card.id.is_some()) {
                        writes.push(IdWrite::new(card, &id.short(), card.id.clone()));
                    }
                    id
                }
            };
            w.tag_with(workspace, Taggable::Fact, fact, &tags)?;
        }

        // Facts of this source that are no longer in the files.
        for (i, e) in existing.iter().enumerate() {
            if e.foreign || e.deleted || used.contains(&i) {
                continue;
            }
            retire(&mut w, e.id)?;
            report.retired += 1;
        }
        let hlc = w.stamp();
        tx.execute(
            "UPDATE sources SET last_scan_at = ?2, updated_at = ?2, hlc = ?3 WHERE id = ?1",
            params![source_id.as_bytes(), now, hlc.to_string()],
        )?;
        oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "source",
            source_id,
            Op::Update,
            &serde_json::json!({ "last_scan_at": now }),
        )?;
        report.stale = tx.query_row(
            "SELECT COUNT(*) FROM anchors a JOIN facts f ON f.id = a.fact_id
             WHERE a.source_id = ?1 AND a.deleted_at IS NULL AND a.stale_since IS NOT NULL
               AND f.deleted_at IS NULL",
            [source_id.as_bytes()],
            |r| r.get(0),
        )?;
        tx.commit()?;

        // Ids go into the files only once the database knows them (SRC-005).
        let (written, write_warnings) = writeback::apply(&root, writes);
        report.ids_written = written;
        report.warnings.extend(write_warnings);
        report.elapsed_ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Scan every source in the workspace, in name order.
    pub fn scan_all(&mut self, workspace: Id) -> Result<Vec<ScanReport>> {
        let names: Vec<String> = self
            .list_sources(workspace)?
            .into_iter()
            .map(|s| s.name)
            .collect();
        names
            .iter()
            .map(|name| self.scan_source(workspace, name))
            .collect()
    }

    /// Cards whose anchored code changed, oldest first (SRC-006).
    pub fn list_stale(&self, workspace: Id, source: Option<&str>) -> Result<Vec<StaleCard>> {
        let source_id = match source {
            Some(key) => Some(self.find_source(workspace, key)?.id.parse::<Id>()?),
            None => None,
        };
        let mut stmt = self.conn.prepare(
            "SELECT f.id, f.card_type, f.fields, s.name, a.path, a.line_start, a.stale_since
             FROM anchors a
             JOIN facts f ON f.id = a.fact_id
             JOIN sources s ON s.id = a.source_id
             WHERE f.workspace_id = ?1 AND a.stale_since IS NOT NULL
               AND a.deleted_at IS NULL AND f.deleted_at IS NULL AND s.deleted_at IS NULL
               AND (?2 IS NULL OR s.id = ?2)
             ORDER BY a.stale_since, a.path, a.line_start",
        )?;
        let rows = stmt.query_map(
            params![
                workspace.as_bytes(),
                source_id.as_ref().map(|i| i.as_bytes().to_vec())
            ],
            |r| {
                Ok((
                    r.get::<_, Vec<u8>>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, u32>(5)?,
                    r.get::<_, i64>(6)?,
                ))
            },
        )?;
        rows.map(|row| {
            let (id, card_type, fields, source, path, line, stale_since) = row?;
            let card_type = CardType::parse(&card_type)?;
            let fields: BTreeMap<String, String> = serde_json::from_str(&fields)
                .map_err(|e| Error::Unexpected(format!("invalid fact fields: {e}")))?;
            Ok(StaleCard {
                id: Id::from_slice(&id)?.short(),
                source,
                path,
                line,
                question: first_field(card_type, &fields),
                stale_since,
            })
        })
        .collect()
    }

    /// Confirm that a stale card is still right (`keepjay stale keep`).
    pub fn keep_stale(&mut self, workspace: Id, card: &str) -> Result<StaleCard> {
        let card = card.trim();
        let stale = self
            .list_stale(workspace, None)?
            .into_iter()
            .find(|s| s.id == card)
            .ok_or_else(|| {
                Error::NotFound(format!("no stale card '{card}'; see `keepjay stale list`"))
            })?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let shorts = all_facts(&tx, workspace)?;
        let fact = shorts
            .into_iter()
            .find(|id| id.short() == stale.id)
            .ok_or_else(|| Error::Unexpected("stale card's fact not found".into()))?;
        let anchor: Vec<u8> = tx.query_row(
            "SELECT id FROM anchors WHERE fact_id = ?1 AND deleted_at IS NULL",
            [fact.as_bytes()],
            |r| r.get(0),
        )?;
        let anchor = Id::from_slice(&anchor)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let hlc = w.stamp();
        tx.execute(
            "UPDATE anchors SET stale_since = NULL, updated_at = ?2, hlc = ?3 WHERE id = ?1",
            params![anchor.as_bytes(), w.now, hlc.to_string()],
        )?;
        oplog::append(
            &tx,
            &hlc,
            self.device_id,
            "anchor",
            anchor,
            Op::Update,
            &serde_json::json!({ "stale": false }),
        )?;
        tx.commit()?;
        Ok(stale)
    }
}

impl Core {
    /// Which lines `retire_stale` would remove: the card's own text (its comment block, or
    /// its Markdown lines and id comment), never the code under it. Changes nothing.
    pub fn plan_retire(&self, workspace: Id, card: &str) -> Result<RetirePlan> {
        let card = card.trim();
        let stale = self
            .list_stale(workspace, None)?
            .into_iter()
            .find(|s| s.id == card)
            .ok_or_else(|| {
                Error::NotFound(format!("no stale card '{card}'; see `keepjay stale list`"))
            })?;
        let root = PathBuf::from(self.find_source(workspace, &stale.source)?.path);
        let parsed = walk::parse_file(&root, &stale.path)?;
        // By the id in the file, or, for sources without written ids, by line and text.
        let found = parsed
            .cards
            .iter()
            .find(|c| c.id.as_deref() == Some(stale.id.as_str()))
            .or_else(|| {
                parsed.cards.iter().find(|c| {
                    c.id.is_none()
                        && c.line_start == stale.line
                        && first_field(c.card_type, &c.fields) == stale.question
                })
            })
            .ok_or_else(|| {
                Error::NotFound(format!(
                    "card {} isn't in {} any more; run `keepjay scan`",
                    stale.id, stale.path
                ))
            })?;
        let text = std::fs::read_to_string(root.join(&stale.path))?;
        let lines: Vec<&str> = text.lines().collect();
        let first = found.line_start;
        let mut last = found.text_end.min(lines.len() as u32);
        // In Markdown, take one of the blank lines around the card with it.
        let blank = |n: u32| {
            lines
                .get(n as usize - 1)
                .is_some_and(|l| l.trim().is_empty())
        };
        if found.style == CardStyle::Markdown
            && (first == 1 || blank(first - 1))
            && (last as usize) < lines.len()
            && blank(last + 1)
        {
            last += 1;
        }
        Ok(RetirePlan {
            lines: lines[first as usize - 1..last as usize]
                .iter()
                .map(|l| l.to_string())
                .collect(),
            first_line: first,
            last_line: last,
            card: stale,
        })
    }

    /// Retire a stale card: remove its text from the file, then scan the source, which
    /// retires the fact (its review history is kept) and updates the other cards' lines.
    /// The file holds the card (D-029), so retiring only in the database wouldn't last.
    pub fn retire_stale(&mut self, workspace: Id, card: &str) -> Result<Retired> {
        let plan = self.plan_retire(workspace, card)?;
        let root = PathBuf::from(self.find_source(workspace, &plan.card.source)?.path);
        let path = root.join(&plan.card.path);
        let text = std::fs::read_to_string(&path)?;
        let kept: String = text
            .split_inclusive('\n')
            .enumerate()
            .filter(|(i, _)| {
                let n = *i as u32 + 1;
                n < plan.first_line || n > plan.last_line
            })
            .map(|(_, l)| l)
            .collect();
        writeback::replace_file(&path, &kept)?;
        let scan = self.scan_source(workspace, &plan.card.source)?;
        Ok(Retired { plan, scan })
    }
}

fn warn(card: &ParsedCard, message: String) -> ParseWarning {
    ParseWarning {
        path: card.path.clone(),
        line: card.line_start,
        message,
    }
}

/// Source tags plus the card's own; invalid tags become warnings.
fn card_tags(
    source_tags: &[String],
    card: &ParsedCard,
    warnings: &mut Vec<ParseWarning>,
) -> Vec<String> {
    let mut out = source_tags.to_vec();
    for t in &card.tags {
        match crate::tags::normalize(t) {
            Ok(t) => out.push(t),
            Err(e) => warnings.push(warn(card, e.to_string())),
        }
    }
    crate::tags::normalize_all(out.iter().map(String::as_str)).unwrap_or(out)
}

fn upsert_anchor(
    w: &mut Writer<'_>,
    fact: Id,
    source: Id,
    repo: &str,
    card: &ParsedCard,
    stale_since: Option<i64>,
) -> Result<()> {
    w.upsert_anchor(&AnchorRow {
        fact,
        source,
        repo,
        path: &card.path,
        line_start: card.line_start,
        line_end: card.line_end,
        code_hash: card.code_hash.as_deref().unwrap_or(""),
        stale_since,
    })?;
    Ok(())
}

/// Make a fact's cards match what its card type generates: add missing ones, restore
/// retired ones, and retire ones no longer generated (e.g. a removed `{{c2::…}}`).
fn sync_cards(
    w: &mut Writer<'_>,
    fact: Id,
    deck: Id,
    card_type: CardType,
    fields: &BTreeMap<String, String>,
) -> Result<()> {
    let ords = card_type.card_ords(fields)?;
    let mut cards: Vec<(Id, u32, bool)> = Vec::new();
    {
        let mut stmt = w.conn.prepare_cached(
            "SELECT id, ord, deleted_at IS NOT NULL FROM cards WHERE fact_id = ?1",
        )?;
        let rows = stmt.query_map([fact.as_bytes()], |r| {
            Ok((
                r.get::<_, Vec<u8>>(0)?,
                r.get::<_, u32>(1)?,
                r.get::<_, bool>(2)?,
            ))
        })?;
        for row in rows {
            let (id, ord, deleted) = row?;
            cards.push((Id::from_slice(&id)?, ord, deleted));
        }
    }
    for ord in &ords {
        match cards.iter().find(|(_, o, _)| o == ord) {
            Some((id, _, true)) => w.set_deleted(Entity::Card, *id, false)?,
            Some(_) => {}
            None => {
                w.insert_card(fact, deck, *ord, false)?;
            }
        }
    }
    for (id, ord, deleted) in &cards {
        if !deleted && !ords.contains(ord) {
            w.set_deleted(Entity::Card, *id, true)?;
        }
    }
    Ok(())
}

/// Soft-delete a fact and its cards; review history and the anchor are kept.
fn retire(w: &mut Writer<'_>, fact: Id) -> Result<()> {
    let live: Vec<Vec<u8>> = {
        let mut stmt = w
            .conn
            .prepare_cached("SELECT id FROM cards WHERE fact_id = ?1 AND deleted_at IS NULL")?;
        stmt.query_map([fact.as_bytes()], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    for card in live {
        w.set_deleted(Entity::Card, Id::from_slice(&card)?, true)?;
    }
    w.set_deleted(Entity::Fact, fact, true)
}

const EXISTING_SQL: &str =
    "SELECT f.id, f.deck_id, f.card_type, f.fields, f.deleted_at IS NOT NULL,
                                   a.path, a.code_hash, a.stale_since,
                                   a.source_id, a.repo, a.line_start, a.line_end
                            FROM facts f
                            LEFT JOIN anchors a ON a.fact_id = f.id AND a.deleted_at IS NULL
                            WHERE f.workspace_id = ?1 AND ";

fn read_existing(
    conn: &rusqlite::Connection,
    sql: &str,
    workspace: Id,
    source_key: &str,
    foreign: bool,
) -> Result<Vec<Existing>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map(params![workspace.as_bytes(), source_key], |r| {
        Ok((
            r.get::<_, Vec<u8>>(0)?,
            r.get::<_, Vec<u8>>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, bool>(4)?,
            r.get::<_, Option<String>>(5)?,
            r.get::<_, Option<String>>(6)?,
            r.get::<_, Option<i64>>(7)?,
            (
                r.get::<_, Option<Vec<u8>>>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, Option<u32>>(10)?,
                r.get::<_, Option<u32>>(11)?,
            ),
        ))
    })?;
    rows.map(|row| {
        let (id, deck, card_type, fields, deleted, path, hash, stale, rest) = row?;
        let id = Id::from_slice(&id)?;
        let anchor = match (&path, rest) {
            (Some(path), (Some(source), Some(repo), Some(line_start), Some(line_end))) => {
                Some(AnchorState {
                    source,
                    repo,
                    path: path.clone(),
                    line_start,
                    line_end,
                    code_hash: hash.clone().unwrap_or_default(),
                    stale_since: stale,
                })
            }
            _ => None,
        };
        Ok(Existing {
            short: id.short(),
            id,
            deck: Id::from_slice(&deck)?,
            card_type: CardType::parse(&card_type)?,
            fields: serde_json::from_str(&fields)
                .map_err(|e| Error::Unexpected(format!("invalid fact fields: {e}")))?,
            deleted,
            foreign,
            anchor_path: path,
            code_hash: hash.filter(|h| !h.is_empty()),
            stale_since: stale,
            anchor,
        })
    })
    .collect()
}

fn load_existing(
    conn: &rusqlite::Connection,
    workspace: Id,
    source_key: &str,
) -> Result<Vec<Existing>> {
    read_existing(
        conn,
        &format!("{EXISTING_SQL} f.source = ?2"),
        workspace,
        source_key,
        false,
    )
}

/// Live facts of other sources, so a card can move between sources by its id (e.g. a
/// source removed and added again).
fn load_foreign(
    conn: &rusqlite::Connection,
    workspace: Id,
    source_key: &str,
) -> Result<Vec<Existing>> {
    read_existing(
        conn,
        &format!(
            "{EXISTING_SQL} f.source LIKE 'src:%' AND f.source != ?2 AND f.deleted_at IS NULL"
        ),
        workspace,
        source_key,
        true,
    )
}

fn all_facts(conn: &rusqlite::Connection, workspace: Id) -> Result<Vec<Id>> {
    let mut stmt = conn.prepare("SELECT id FROM facts WHERE workspace_id = ?1")?;
    stmt.query_map([workspace.as_bytes()], |r| r.get::<_, Vec<u8>>(0))?
        .map(|b| Id::from_slice(&b?))
        .collect()
}

/// Short ids of every fact in the workspace, to keep ids from files unique.
fn all_shorts(conn: &rusqlite::Connection, workspace: Id) -> Result<HashSet<String>> {
    Ok(all_facts(conn, workspace)?.iter().map(Id::short).collect())
}
