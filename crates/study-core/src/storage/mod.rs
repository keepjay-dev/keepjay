// SPDX-License-Identifier: AGPL-3.0-or-later
//! SQLite storage: connection setup, forward-only migrations, change log
//! (docs/specs/08-architecture.md §7–§8, 09-data-model.md §9).

pub mod device;
pub mod oplog;

use std::time::Duration;

use rusqlite::Connection;

use crate::error::{Error, Result};
use crate::paths::Paths;

/// Ordered, append-only list of migrations. Never edit a released entry; add a new one.
const MIGRATIONS: &[&str] = &[include_str!("migrations/0001_init.sql")];

/// Highest schema version this build understands.
pub const SUPPORTED_SCHEMA_VERSION: u32 = MIGRATIONS.len() as u32;

/// Open the profile database with the pragmas from 08 §7 and apply pending migrations.
pub fn open(paths: &Paths) -> Result<Connection> {
    let db_path = paths.db_path();
    let existed = db_path.exists();
    let mut conn = Connection::open(&db_path)?;
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    migrate(&mut conn, paths, existed)?;
    Ok(conn)
}

pub fn schema_version(conn: &Connection) -> Result<u32> {
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

fn migrate(conn: &mut Connection, paths: &Paths, existed: bool) -> Result<()> {
    let current = schema_version(conn)?;
    if current > SUPPORTED_SCHEMA_VERSION {
        return Err(Error::SchemaTooNew {
            db: current,
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }
    if current == SUPPORTED_SCHEMA_VERSION {
        return Ok(());
    }
    if existed && current > 0 {
        backup_before_migration(conn, paths, current)?; // NFR-REL-004
    }
    let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Exclusive)?;
    // Re-check inside the lock: another process may have migrated meanwhile.
    let current = schema_version(&tx)?;
    for (i, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", (i + 1) as u32)?;
    }
    tx.commit()?;
    Ok(())
}

fn backup_before_migration(conn: &Connection, paths: &Paths, from_version: u32) -> Result<()> {
    let dir = paths.backups_dir();
    std::fs::create_dir_all(&dir)?;
    let target = dir.join(format!(
        "pre-migration-v{from_version}-{}.db",
        crate::domain::now_ms()
    ));
    conn.execute("VACUUM INTO ?1", [target.to_string_lossy()])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_is_fully_migrated_and_in_wal_mode() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path(), dir.path());
        let conn = open(&paths).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), SUPPORTED_SCHEMA_VERSION);
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .unwrap();
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn refuses_newer_schema() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path(), dir.path());
        {
            let conn = open(&paths).unwrap();
            conn.pragma_update(None, "user_version", SUPPORTED_SCHEMA_VERSION + 1)
                .unwrap();
        }
        let err = open(&paths).unwrap_err();
        assert!(matches!(err, Error::SchemaTooNew { .. }), "{err}");
    }
}
