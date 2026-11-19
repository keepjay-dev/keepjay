// SPDX-License-Identifier: AGPL-3.0-or-later
//! Reading an Anki collection (schema 11 or 18) into plain structs.
//!
//! Schema 11 keeps note types and decks as JSON in `col`; schema 18 has `notetypes`,
//! `fields`, `templates` and `decks` tables with protobuf config blobs. Deck names use
//! `::` (11) or `\x1f` (18) between levels. Cards in a filtered deck point back to their
//! home deck through `odid`.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::package::Format;
use crate::error::{Error, Result};
use crate::io::proto;

#[derive(Debug, Clone)]
pub(crate) struct NoteType {
    pub name: String,
    pub cloze: bool,
    pub fields: Vec<String>,
    /// (question format, answer format) per template, by ordinal.
    pub templates: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub(crate) struct Note {
    pub id: i64,
    pub guid: String,
    pub note_type: i64,
    pub fields: Vec<String>,
    pub has_tags: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AnkiCard {
    pub id: i64,
    pub ord: u32,
    /// Home deck (the original deck for cards sitting in a filtered deck).
    pub deck: i64,
    pub suspended: bool,
    /// Anki considered it studied (`type != 0`).
    pub studied: bool,
}

/// One answer from `revlog`: (time ms, ease 1–4, answer time ms). Manual entries
/// (ease 0: rescheduled, set due date) are dropped.
pub(crate) type Answer = (i64, u8, i64);

pub(crate) struct Collection {
    pub note_types: HashMap<i64, NoteType>,
    pub decks: HashMap<i64, String>,
    pub notes: Vec<Note>,
    pub cards: HashMap<i64, Vec<AnkiCard>>,
    pub answers: HashMap<i64, Vec<Answer>>,
}

pub(crate) fn read(db_path: &Path, format: Format) -> Result<Collection> {
    let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    // Schema 18 tables declare `COLLATE unicase`; queries fail without it.
    conn.create_collation("unicase", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))?;
    let (note_types, decks) = match format {
        Format::Latest => (note_types_18(&conn)?, decks_18(&conn)?),
        Format::Legacy => legacy_types_and_decks(&conn)?,
    };

    let notes = conn
        .prepare("SELECT id, guid, mid, flds, tags FROM notes ORDER BY id")?
        .query_map([], |r| {
            let flds: String = r.get(3)?;
            let tags: String = r.get(4)?;
            Ok(Note {
                id: r.get(0)?,
                guid: r.get(1)?,
                note_type: r.get(2)?,
                fields: flds.split('\x1f').map(str::to_string).collect(),
                has_tags: !tags.trim().is_empty(),
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut cards: HashMap<i64, Vec<AnkiCard>> = HashMap::new();
    let mut stmt =
        conn.prepare("SELECT id, nid, ord, did, odid, queue, type FROM cards ORDER BY nid, ord")?;
    let rows = stmt.query_map([], |r| {
        let (did, odid): (i64, i64) = (r.get(3)?, r.get(4)?);
        let (queue, kind): (i64, i64) = (r.get(5)?, r.get(6)?);
        Ok((
            r.get::<_, i64>(1)?,
            AnkiCard {
                id: r.get(0)?,
                ord: r.get(2)?,
                deck: if odid != 0 { odid } else { did },
                suspended: queue == -1,
                studied: kind != 0,
            },
        ))
    })?;
    for row in rows {
        let (nid, card) = row?;
        cards.entry(nid).or_default().push(card);
    }

    let mut answers: HashMap<i64, Vec<Answer>> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT cid, id, ease, time FROM revlog WHERE ease BETWEEN 1 AND 4 ORDER BY cid, id",
    )?;
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?)))
    })?;
    for row in rows {
        let (cid, answer) = row?;
        answers.entry(cid).or_default().push(answer);
    }

    Ok(Collection {
        note_types,
        decks,
        notes,
        cards,
        answers,
    })
}

fn note_types_18(conn: &Connection) -> Result<HashMap<i64, NoteType>> {
    let mut types: HashMap<i64, NoteType> = HashMap::new();
    let mut stmt = conn.prepare("SELECT id, name, config FROM notetypes")?;
    for row in stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, Vec<u8>>(2)?,
        ))
    })? {
        let (id, name, config) = row?;
        // Notetype.Config: kind = 1 (0 normal, 1 cloze).
        let cloze = proto::varint_field(&config, 1) == Some(1);
        types.insert(
            id,
            NoteType {
                name,
                cloze,
                fields: Vec::new(),
                templates: Vec::new(),
            },
        );
    }
    let mut stmt = conn.prepare("SELECT ntid, name FROM fields ORDER BY ntid, ord")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (ntid, name) = row?;
        if let Some(t) = types.get_mut(&ntid) {
            t.fields.push(name);
        }
    }
    let mut stmt = conn.prepare("SELECT ntid, config FROM templates ORDER BY ntid, ord")?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?)))? {
        let (ntid, config) = row?;
        // Template.Config: q_format = 1, a_format = 2.
        let q = proto::string_field(&config, 1).unwrap_or_default();
        let a = proto::string_field(&config, 2).unwrap_or_default();
        if let Some(t) = types.get_mut(&ntid) {
            t.templates.push((q, a));
        }
    }
    Ok(types)
}

fn decks_18(conn: &Connection) -> Result<HashMap<i64, String>> {
    let mut stmt = conn.prepare("SELECT id, name FROM decks")?;
    let decks = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?.replace('\x1f', "::"),
            ))
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(decks)
}

type TypesAndDecks = (HashMap<i64, NoteType>, HashMap<i64, String>);

fn legacy_types_and_decks(conn: &Connection) -> Result<TypesAndDecks> {
    let (models, decks): (String, String) =
        conn.query_row("SELECT models, decks FROM col", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })?;
    let bad = |what: &str, e: serde_json::Error| {
        Error::validation(format!("Anki collection has unreadable {what}: {e}"))
    };
    let models: HashMap<String, serde_json::Value> =
        serde_json::from_str(&models).map_err(|e| bad("note types", e))?;
    let decks: HashMap<String, serde_json::Value> =
        serde_json::from_str(&decks).map_err(|e| bad("decks", e))?;

    let text = |v: &serde_json::Value, k: &str| v[k].as_str().unwrap_or_default().to_string();
    let mut types = HashMap::new();
    for (id, m) in models {
        let Ok(id) = id.parse::<i64>() else { continue };
        let mut fields: Vec<(i64, String)> = m["flds"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|f| (f["ord"].as_i64().unwrap_or(0), text(f, "name")))
                    .collect()
            })
            .unwrap_or_default();
        fields.sort();
        let mut templates: Vec<(i64, (String, String))> = m["tmpls"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|t| {
                        (
                            t["ord"].as_i64().unwrap_or(0),
                            (text(t, "qfmt"), text(t, "afmt")),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        templates.sort_by_key(|(ord, _)| *ord);
        types.insert(
            id,
            NoteType {
                name: text(&m, "name"),
                cloze: m["type"].as_i64() == Some(1),
                fields: fields.into_iter().map(|(_, n)| n).collect(),
                templates: templates.into_iter().map(|(_, t)| t).collect(),
            },
        );
    }
    let decks = decks
        .into_iter()
        .filter_map(|(id, d)| Some((id.parse::<i64>().ok()?, text(&d, "name"))))
        .collect();
    Ok((types, decks))
}
