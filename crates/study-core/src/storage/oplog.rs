// SPDX-License-Identifier: AGPL-3.0-or-later
//! Append-only change log (D-009; docs/specs/08-architecture.md §8–§9).

use rusqlite::{Connection, params};
use serde::Serialize;

use crate::domain::HlcStamp;
use crate::domain::Id;
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Op {
    Insert,
    Update,
    Delete,
}

impl Op {
    fn as_str(self) -> &'static str {
        match self {
            Op::Insert => "insert",
            Op::Update => "update",
            Op::Delete => "delete",
        }
    }
}

/// One committed change, as seen by other processes.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Change {
    pub seq: i64,
    pub hlc: String,
    pub entity: String,
    pub entity_id: String,
    pub op: String,
}

/// Record a change. Must be called inside the same transaction as the write itself.
pub fn append(
    conn: &Connection,
    hlc: &HlcStamp,
    device: Id,
    entity: &str,
    entity_id: Id,
    op: Op,
    fields: &serde_json::Value,
) -> Result<()> {
    conn.execute(
        "INSERT INTO oplog (hlc, device_id, entity, entity_id, op, fields)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            hlc.to_string(),
            device.as_bytes(),
            entity,
            entity_id.as_bytes(),
            op.as_str(),
            fields.to_string()
        ],
    )?;
    Ok(())
}

pub fn since(conn: &Connection, since_seq: i64) -> Result<Vec<Change>> {
    let mut stmt = conn
        .prepare("SELECT seq, hlc, entity, entity_id, op FROM oplog WHERE seq > ?1 ORDER BY seq")?;
    let rows = stmt.query_map([since_seq], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, Vec<u8>>(3)?,
            r.get::<_, String>(4)?,
        ))
    })?;
    rows.map(|row| {
        let (seq, hlc, entity, id, op) = row?;
        Ok(Change {
            seq,
            hlc,
            entity,
            entity_id: Id::from_slice(&id)?.to_string(),
            op,
        })
    })
    .collect()
}
