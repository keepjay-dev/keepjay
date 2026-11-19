// SPDX-License-Identifier: AGPL-3.0-or-later
//! Anki import (IO-001): `.apkg` / `.colpkg` in both the legacy (schema 11) and latest
//! (schema 18, Anki 2.1.50+) formats.
//!
//! - Notes become facts; stock note types map onto built-in card types by shape, others
//!   are rendered card by card through their own Anki templates.
//! - Field HTML is stored unchanged (lossless); plain-text views strip it.
//! - Review history is replayed through FSRS, so every card gets an FSRS-7 state derived
//!   from its full Anki history (SRS-006, D-009).
//! - Re-importing skips notes already imported (matched by Anki note guid).
//! - Everything happens in one transaction: an import fully succeeds or changes nothing.
//!
//! - Note tags go onto the facts (WS-003), matched case-insensitively with existing tags.
//!
//! - Media files go into the content-addressed store (D-026) before the transaction, so the
//!   database isn't locked while large files are copied. Fields keep referring to them by
//!   name; a name already used for other content becomes `name-<hash8>.ext` and the
//!   imported fields are rewritten to match, as Anki does. If the import fails, stored
//!   files stay as unreferenced content for later clean-up; the database is unchanged.

mod collection;
mod mapping;
mod package;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use serde::Serialize;

use crate::Core;
use crate::cards::{CardType, DECK_SEPARATOR, render};
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::media::{self, Attached};
use crate::srs::{Rating, review::replay_into};
use crate::storage::media::{MediaStore, Stored};
use crate::tags::{self, Taggable};
use crate::write::Writer;
use mapping::Mapping;

#[derive(Debug, Clone, Default)]
pub struct AnkiImportOptions {
    /// Put imported decks under this deck path, e.g. `Imported` → `Imported::Biology`.
    pub deck_prefix: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct NoteTypeReport {
    pub name: String,
    pub imported_as: String,
    pub notes: u64,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct AnkiImportSummary {
    /// `latest` (collection.anki21b) or `legacy` (collection.anki21/anki2).
    pub format: String,
    pub notes_imported: u64,
    pub cards_imported: u64,
    pub review_logs_imported: u64,
    pub notes_already_imported: u64,
    pub notes_skipped: u64,
    pub skipped_reasons: BTreeMap<String, u64>,
    /// Studied in Anki but with no usable history (e.g. history was cleared): now new.
    pub cards_reset_without_history: u64,
    /// Anki cards that don't correspond to a card the built-in type generates.
    pub cards_unmatched: u64,
    pub suspended: u64,
    pub decks_created: Vec<String>,
    pub note_types: Vec<NoteTypeReport>,
    /// Media files in the package.
    pub media_files: u64,
    /// Files now available under their own name.
    pub media_added: u64,
    /// Files the workspace already had (same name, same content).
    pub media_already_present: u64,
    /// Files whose name was taken by other content, stored as `name-<hash8>.ext`.
    pub media_renamed: u64,
    /// Files that couldn't be imported (unsafe name, too large, corrupt).
    pub media_skipped: u64,
    /// Distinct names referenced by the package's notes that the workspace doesn't have.
    pub media_missing: u64,
    /// Package files no note refers to (Anki's `_` files for templates excluded).
    pub media_unused: u64,
    /// Imported notes that carry at least one tag.
    pub notes_tagged: u64,
    /// Tags that didn't exist in the workspace before.
    pub tags_created: u64,
    /// Anki tags that couldn't be used as tags here (e.g. over 200 characters).
    pub tags_skipped: u64,
}

impl AnkiImportSummary {
    fn skip(&mut self, reason: &str) {
        self.notes_skipped += 1;
        *self.skipped_reasons.entry(reason.to_string()).or_default() += 1;
    }

    fn count_tags(&mut self, tags: &[String], bad: u64) {
        self.notes_tagged += u64::from(!tags.is_empty());
        self.tags_skipped += bad;
    }
}

/// Anki tags as tag paths, leniently: empty levels (`a::::b`, `a::`) are dropped, as Anki
/// itself does. Returns the usable paths and how many tags had to be skipped.
fn anki_tags(raw: &[String]) -> (Vec<String>, u64) {
    let mut bad = 0;
    let cleaned: Vec<String> = raw
        .iter()
        .filter_map(|t| {
            let joined = t
                .split(tags::TAG_SEPARATOR)
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join(tags::TAG_SEPARATOR);
            match tags::normalize(&joined) {
                Ok(path) => Some(path),
                Err(_) => {
                    bad += 1;
                    None
                }
            }
        })
        .collect();
    let paths = tags::normalize_all(cleaned.iter().map(String::as_str))
        .expect("paths are already normalised");
    (paths, bad)
}

impl Core {
    /// Import an Anki package into a workspace (IO-001). See the module docs.
    pub fn import_anki(
        &mut self,
        workspace: Id,
        path: &Path,
        options: &AnkiImportOptions,
    ) -> Result<AnkiImportSummary> {
        let mut pkg = package::open(path)?;
        let col = collection::read(&pkg.db_path, pkg.format)?;
        let mut summary = AnkiImportSummary {
            format: match pkg.format {
                package::Format::Latest => "latest".into(),
                package::Format::Legacy => "legacy".into(),
            },
            media_files: pkg.media.len() as u64,
            ..Default::default()
        };
        let stored_media = store_media(&self.media_store(), &mut pkg, &mut summary)?;

        let already: HashSet<String> = self
            .conn
            .prepare(
                "SELECT source FROM facts WHERE workspace_id = ?1 AND source LIKE 'anki:%'
                   AND deleted_at IS NULL",
            )?
            .query_map([workspace.as_bytes()], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?;

        let prefix: Vec<String> = options
            .deck_prefix
            .as_deref()
            .map(|p| {
                p.split(DECK_SEPARATOR)
                    .map(|s| s.trim().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let mut decks: HashMap<i64, Id> = HashMap::new();
        let mut mappings: HashMap<i64, Mapping> = HashMap::new();
        let mut type_counts: HashMap<i64, u64> = HashMap::new();

        let mut renames: HashMap<String, String> = HashMap::new();
        let mut package_names: Vec<String> = Vec::with_capacity(stored_media.len());
        for (name, stored, ext) in &stored_media {
            match w.attach_media(workspace, name, stored, ext)? {
                Attached::Added => summary.media_added += 1,
                Attached::AlreadyPresent => summary.media_already_present += 1,
                Attached::Renamed(new) => {
                    summary.media_renamed += 1;
                    renames.insert(name.clone(), new);
                }
            }
            package_names.push(renames.get(name).unwrap_or(name).clone());
        }
        let mut referenced: HashSet<String> = HashSet::new();

        for note in &col.notes {
            let Some(nt) = col.note_types.get(&note.note_type) else {
                summary.skip("unknown note type");
                continue;
            };
            let Some(cards) = col.cards.get(&note.id).filter(|c| !c.is_empty()) else {
                summary.skip("note has no cards");
                continue;
            };
            let mapping = mappings
                .entry(note.note_type)
                .or_insert_with(|| mapping::map(nt))
                .clone();
            *type_counts.entry(note.note_type).or_default() += 1;
            if let Mapping::Skip(reason) = mapping {
                summary.skip(reason);
                continue;
            }
            let (tags, bad_tags) = anki_tags(&note.tags);
            let rewritten: Vec<String>;
            let note_fields: &[String] = if renames.is_empty() {
                &note.fields
            } else {
                rewritten = note
                    .fields
                    .iter()
                    .map(|f| media::rewrite_references(f, &renames))
                    .collect();
                &rewritten
            };
            for f in note_fields {
                referenced.extend(media::referenced_names(f));
            }

            let mut deck_of = |w: &mut Writer<'_>, created: &mut Vec<String>, anki_deck: i64| {
                resolve_deck(
                    w, workspace, &col.decks, &prefix, &mut decks, created, anki_deck,
                )
            };

            let answers_of = |card_id: i64| -> Vec<(i64, Rating, i64)> {
                col.answers
                    .get(&card_id)
                    .map(|list| {
                        list.iter()
                            .filter_map(|&(at, ease, ms)| {
                                Rating::from_number(ease).ok().map(|r| (at, r, ms))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };

            // Built-in mapping, unless the note can't form valid built-in cards.
            let builtin = match &mapping {
                Mapping::Builtin {
                    card_type,
                    cloze_text,
                } => {
                    let fields = mapping::builtin_fields(*card_type, *cloze_text, note_fields);
                    match card_type.card_ords(&fields) {
                        Ok(ords) => Some((*card_type, fields, ords)),
                        Err(_) if *card_type == CardType::Cloze => {
                            summary.skip("cloze note without deletions");
                            continue;
                        }
                        Err(_) => None, // e.g. empty front: fall back to rendering
                    }
                }
                _ => None,
            };

            if let Some((card_type, fields, ords)) = builtin {
                let source = format!("anki:{}", note.guid);
                if already.contains(&source) {
                    summary.notes_already_imported += 1;
                    continue;
                }
                let fact_deck = deck_of(&mut w, &mut summary.decks_created, cards[0].deck)?;
                let fact =
                    w.insert_fact(workspace, fact_deck, card_type, &fields, Some(&source))?;
                summary.tags_created += w.tag_with(workspace, Taggable::Fact, fact, &tags)?;
                for ord in &ords {
                    let anki = cards.iter().find(|c| c.ord == *ord);
                    let deck = match anki {
                        Some(c) => deck_of(&mut w, &mut summary.decks_created, c.deck)?,
                        None => fact_deck,
                    };
                    let suspended = anki.is_some_and(|c| c.suspended);
                    let card = w.insert_card(fact, deck, *ord, suspended)?;
                    summary.cards_imported += 1;
                    summary.suspended += suspended as u64;
                    if let Some(c) = anki {
                        let answers = answers_of(c.id);
                        if answers.is_empty() && c.studied {
                            summary.cards_reset_without_history += 1;
                        }
                        let (reps, _) = replay_into(&mut w, card, &answers)?;
                        summary.review_logs_imported += u64::from(reps);
                    }
                }
                summary.cards_unmatched +=
                    cards.iter().filter(|c| !ords.contains(&c.ord)).count() as u64;
                summary.notes_imported += 1;
                summary.count_tags(&tags, bad_tags);
            } else {
                // Rendered: each Anki card becomes its own Basic fact.
                let mut imported_any = false;
                for c in cards {
                    let source = format!("anki:{}:{}", note.guid, c.ord);
                    if already.contains(&source) {
                        continue;
                    }
                    let Some((question, answer)) = mapping::render_card(nt, c.ord, note_fields)
                    else {
                        summary.cards_unmatched += 1;
                        continue;
                    };
                    if render::html_to_text(&question).trim().is_empty() {
                        summary.cards_unmatched += 1;
                        continue;
                    }
                    let fields = BTreeMap::from([
                        ("Front".to_string(), question),
                        ("Back".to_string(), answer),
                    ]);
                    let deck = deck_of(&mut w, &mut summary.decks_created, c.deck)?;
                    let fact =
                        w.insert_fact(workspace, deck, CardType::Basic, &fields, Some(&source))?;
                    summary.tags_created += w.tag_with(workspace, Taggable::Fact, fact, &tags)?;
                    let card = w.insert_card(fact, deck, 0, c.suspended)?;
                    summary.cards_imported += 1;
                    summary.suspended += c.suspended as u64;
                    let answers = answers_of(c.id);
                    if answers.is_empty() && c.studied {
                        summary.cards_reset_without_history += 1;
                    }
                    let (reps, _) = replay_into(&mut w, card, &answers)?;
                    summary.review_logs_imported += u64::from(reps);
                    imported_any = true;
                }
                if imported_any {
                    summary.notes_imported += 1;
                    summary.count_tags(&tags, bad_tags);
                } else if cards
                    .iter()
                    .any(|c| already.contains(&format!("anki:{}:{}", note.guid, c.ord)))
                {
                    summary.notes_already_imported += 1;
                } else {
                    summary.skip("no card could be rendered");
                }
            }
        }
        for name in &referenced {
            summary.media_missing += u64::from(!w.has_media(workspace, name)?);
        }
        summary.media_unused = package_names
            .iter()
            .filter(|n| !n.starts_with('_') && !referenced.contains(*n))
            .count() as u64;
        tx.commit()?;

        let mut reports: Vec<NoteTypeReport> = type_counts
            .into_iter()
            .map(|(id, notes)| NoteTypeReport {
                name: col.note_types[&id].name.clone(),
                imported_as: mappings[&id].label(),
                notes,
            })
            .collect();
        reports.sort_by(|a, b| b.notes.cmp(&a.notes).then(a.name.cmp(&b.name)));
        summary.note_types = reports;
        summary.decks_created.sort();
        Ok(summary)
    }
}

/// The most media a package may declare in total (files are capped at
/// [`media::MAX_MEDIA_FILE`]). Real collections stay far below; the limits stop zip bombs
/// from filling the disk.
const MAX_MEDIA_TOTAL: u64 = 64 << 30;

/// Copy the package's media into the store. Returns (name, stored file, extension) for each
/// file that made it; the rest are counted as skipped.
fn store_media(
    store: &MediaStore,
    pkg: &mut package::Package,
    summary: &mut AnkiImportSummary,
) -> Result<Vec<(String, Stored, String)>> {
    let declared: u64 = pkg.media.iter().map(|m| m.size).sum();
    if declared > MAX_MEDIA_TOTAL {
        return Err(Error::validation(format!(
            "the package declares {} GiB of media; the limit is {} GiB",
            declared >> 30,
            MAX_MEDIA_TOTAL >> 30
        )));
    }
    let entries = pkg.media.clone();
    let mut out = Vec::with_capacity(entries.len());
    for m in entries {
        if !media::valid_name(&m.name) || m.size > media::MAX_MEDIA_FILE {
            summary.media_skipped += 1;
            continue;
        }
        let ext = media::ext_of(&m.name);
        let stored = match pkg.media_reader(&m.entry) {
            Ok(reader) => store.put(reader, &ext, m.size),
            Err(e) => Err(e),
        };
        match stored {
            Ok(s) if s.size == m.size => out.push((m.name, s, ext)),
            Ok(_) => summary.media_skipped += 1, // shorter than declared: corrupt
            Err(e) if e.kind() == crate::error::ErrorKind::Validation => summary.media_skipped += 1,
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

/// Our deck for an Anki deck id, creating the path (under the optional prefix) on first use.
fn resolve_deck(
    w: &mut Writer<'_>,
    workspace: Id,
    anki_decks: &HashMap<i64, String>,
    prefix: &[String],
    cache: &mut HashMap<i64, Id>,
    created: &mut Vec<String>,
    anki_deck: i64,
) -> Result<Id> {
    if let Some(id) = cache.get(&anki_deck) {
        return Ok(*id);
    }
    let name = anki_decks
        .get(&anki_deck)
        .map(String::as_str)
        .unwrap_or("Imported");
    let mut parts: Vec<&str> = prefix.iter().map(String::as_str).collect();
    parts.extend(
        name.split(DECK_SEPARATOR)
            .map(str::trim)
            .filter(|p| !p.is_empty()),
    );
    if parts.is_empty() {
        parts.push("Imported");
    }
    let (id, new) = w.ensure_deck_path(workspace, &parts)?;
    created.extend(new);
    cache.insert(anki_deck, id);
    Ok(id)
}
