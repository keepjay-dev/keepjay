// SPDX-License-Identifier: AGPL-3.0-or-later
//! SQLite storage: connection setup, forward-only migrations, change log
//! (docs/specs/08-architecture.md §7–§8, 09-data-model.md §9).

pub mod device;
pub mod media;
pub mod oplog;

use std::time::Duration;

use rusqlite::Connection;

use crate::error::{Error, Result};
use crate::paths::Paths;

/// Ordered, append-only list of migrations. Never edit a released entry; add a new one.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/0001_init.sql"),
    include_str!("migrations/0002_cards.sql"),
    include_str!("migrations/0003_reviews.sql"),
    include_str!("migrations/0004_fact_source.sql"),
    include_str!("migrations/0005_tags.sql"),
    include_str!("migrations/0006_media.sql"),
    include_str!("migrations/0007_sources.sql"),
    include_str!("migrations/0008_review_undos.sql"),
];

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

// @card(xfs9x4xm) Why does `migrate` read the schema version again after taking the EXCLUSIVE lock?
// Another process may have migrated the database in the meantime; checking inside the
// lock means no migration runs twice.
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
        backup_before_migration(conn, paths)?; // NFR-REL-004
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

fn backup_before_migration(conn: &Connection, paths: &Paths) -> Result<()> {
    use crate::backup::{self, BackupKind};
    backup::create(
        conn,
        paths,
        BackupKind::PreMigration,
        crate::domain::now_ms(),
    )?;
    backup::prune(&paths.backups_dir())?;
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
    fn upgrading_an_existing_database_takes_a_backup_first() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path(), dir.path());
        {
            // Simulate a database created by a build that only knew migration 1.
            let conn = Connection::open(paths.db_path()).unwrap();
            conn.execute_batch(MIGRATIONS[0]).unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
        }
        let conn = open(&paths).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), SUPPORTED_SCHEMA_VERSION);
        let backups = crate::backup::list(&paths.backups_dir()).unwrap();
        assert_eq!(backups.len(), 1, "{backups:?}");
        assert_eq!(backups[0].kind, crate::BackupKind::PreMigration);
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
