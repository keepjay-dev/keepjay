// SPDX-License-Identifier: AGPL-3.0-or-later
//! Local backups (IO-004, NFR-REL-002, NFR-REL-004; docs/specs/09-data-model.md §8).
//!
//! A backup is a consistent copy of the database (`VACUUM INTO`), checked with SQLite's
//! quick check and compressed with zstd: `backups/<UTC time>-<kind>.db.zst`. Kinds:
//!
//! - `auto`: at most one a day, made in the background after a command. Kept: the newest of
//!   each of the last 7 days that have one, and of each of the last 4 ISO weeks.
//! - `manual`: `keepjay backup create`. Never deleted automatically.
//! - `pre-migration` and `pre-restore`: taken before a schema migration or a restore. The
//!   newest 3 of each are kept.
//!
//! Media files aren't copied: they're content-addressed and never changed, so a restored
//! database still finds them.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use jiff::tz::TimeZone;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;

use crate::error::{Error, Result};
use crate::paths::Paths;
use crate::storage::SUPPORTED_SCHEMA_VERSION;

const HOUR_MS: i64 = 3_600_000;
const DAY_MS: i64 = 24 * HOUR_MS;
/// An automatic backup is due this long after the last one ("daily", with some slack so a
/// daily habit at slightly different times still gets one a day).
pub const AUTO_EVERY_MS: i64 = 20 * HOUR_MS;
/// After a failed or skipped attempt, wait this long before trying again.
const RETRY_AFTER_MS: i64 = HOUR_MS;
const KEEP_DAILY: usize = 7;
const KEEP_WEEKLY: usize = 4;
const KEEP_SAFETY: usize = 3;
/// Touched when an automatic backup is started, so other commands don't start another.
const ATTEMPT_MARKER: &str = ".last-auto-attempt";
const EXT: &str = ".db.zst";
/// Backups taken before migrations by 0.2.0-alpha.3 and earlier: `pre-migration-v<N>-<ms>.db`.
const LEGACY_EXT: &str = ".db";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackupKind {
    Auto,
    Manual,
    PreMigration,
    PreRestore,
}

impl BackupKind {
    const ALL: [BackupKind; 4] = [
        BackupKind::Auto,
        BackupKind::Manual,
        BackupKind::PreMigration,
        BackupKind::PreRestore,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BackupKind::Auto => "auto",
            BackupKind::Manual => "manual",
            BackupKind::PreMigration => "pre-migration",
            BackupKind::PreRestore => "pre-restore",
        }
    }
}

/// One backup file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Backup {
    /// The file name without its extension, e.g. `20261124-101503-auto`.
    pub id: String,
    pub kind: BackupKind,
    /// When it was taken (Unix ms, from its file name).
    pub created_at: i64,
    /// Size on disk.
    pub bytes: u64,
    pub path: String,
}

/// What `Core::restore_backup` did.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RestoreReport {
    pub restored: Backup,
    /// The data that was replaced, backed up first: restoring it undoes the restore.
    pub previous: Backup,
}

/// Every backup in `dir`, newest first. A missing folder has none.
pub fn list(dir: &Path) -> Result<Vec<Backup>> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some((id, kind, created_at)) = parse_name(name) else {
            continue;
        };
        let bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        out.push(Backup {
            id,
            kind,
            created_at,
            bytes,
            path: entry.path().display().to_string(),
        });
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(b.id.cmp(&a.id)));
    Ok(out)
}

/// The backup called `id`, or the only one whose id starts with it.
pub fn find(dir: &Path, id: &str) -> Result<Backup> {
    let all = list(dir)?;
    if let Some(b) = all.iter().find(|b| b.id == id) {
        return Ok(b.clone());
    }
    let matches: Vec<_> = all.iter().filter(|b| b.id.starts_with(id)).collect();
    match matches.as_slice() {
        [b] if !id.is_empty() => Ok((*b).clone()),
        [] | [_] => Err(Error::NotFound(format!(
            "no backup called '{id}'; see `keepjay backup list`"
        ))),
        _ => Err(Error::validation(format!(
            "'{id}' matches {} backups; give more of the id",
            matches.len()
        ))),
    }
}

/// Back up the database behind `conn` into `paths.backups_dir()`.
pub fn create(conn: &Connection, paths: &Paths, kind: BackupKind, now_ms: i64) -> Result<Backup> {
    let dir = paths.backups_dir();
    fs::create_dir_all(&dir)?;
    let stamp = stamp(now_ms);
    let name = (1..)
        .map(|n| match n {
            1 => format!("{stamp}-{}", kind.as_str()),
            n => format!("{stamp}-{}-{n}", kind.as_str()),
        })
        .find(|name| !dir.join(format!("{name}{EXT}")).exists())
        .expect("an unused name");
    let raw = dir.join(format!(".{name}.tmp.db"));
    let packed = dir.join(format!(".{name}.tmp.zst"));
    remove_if_exists(&raw)?;
    let result = (|| {
        conn.execute("VACUUM INTO ?1", [raw.to_string_lossy()])?;
        check(&raw, "quick_check")?;
        let data = fs::read(&raw)?;
        let compressed = ruzstd::encoding::compress_to_vec(
            &data[..],
            ruzstd::encoding::CompressionLevel::Fastest,
        );
        let mut file = File::create(&packed)?;
        file.write_all(&compressed)?;
        file.sync_all()?;
        drop(file);
        let target = dir.join(format!("{name}{EXT}"));
        fs::rename(&packed, &target)?;
        Ok(Backup {
            id: name.clone(),
            kind,
            created_at: now_ms - now_ms.rem_euclid(1000),
            bytes: compressed.len() as u64,
            path: target.display().to_string(),
        })
    })();
    let _ = fs::remove_file(&raw);
    let _ = fs::remove_file(&packed);
    result
}

/// Delete the backups the retention rules no longer keep (see the module docs).
pub fn prune(dir: &Path) -> Result<Vec<Backup>> {
    let all = list(dir)?;
    let doomed = plan_prune(&all);
    for b in &doomed {
        remove_if_exists(Path::new(&b.path))?;
    }
    Ok(doomed)
}

fn plan_prune(newest_first: &[Backup]) -> Vec<Backup> {
    let mut keep: HashSet<&str> = HashSet::new();
    let (mut days, mut weeks) = (HashSet::new(), HashSet::new());
    for b in newest_first.iter().filter(|b| b.kind == BackupKind::Auto) {
        let day = b.created_at.div_euclid(DAY_MS);
        if days.len() < KEEP_DAILY && days.insert(day) {
            keep.insert(&b.id);
        }
        let week = crate::telemetry::iso_week(b.created_at);
        if weeks.len() < KEEP_WEEKLY && weeks.insert(week) {
            keep.insert(&b.id);
        }
    }
    for kind in [BackupKind::PreMigration, BackupKind::PreRestore] {
        keep.extend(
            newest_first
                .iter()
                .filter(|b| b.kind == kind)
                .take(KEEP_SAFETY)
                .map(|b| b.id.as_str()),
        );
    }
    newest_first
        .iter()
        .filter(|b| b.kind != BackupKind::Manual && !keep.contains(b.id.as_str()))
        .cloned()
        .collect()
}

/// Whether an automatic backup should start now. Cheap: one `stat` and one directory read,
/// as it runs after every command.
pub fn auto_due(paths: &Paths, now_ms: i64) -> bool {
    let dir = paths.backups_dir();
    let attempted = fs::metadata(dir.join(ATTEMPT_MARKER))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64);
    // Either side of now: the file's time can be a little ahead of the caller's clock.
    if attempted.is_some_and(|t| (now_ms - t).abs() < RETRY_AFTER_MS) {
        return false;
    }
    match list(&dir) {
        Ok(all) => last_auto(&all).is_none_or(|t| now_ms - t >= AUTO_EVERY_MS),
        Err(_) => false,
    }
}

/// Record that an automatic backup is starting. False if that couldn't be recorded (then
/// don't start one: it would be retried after every command).
pub fn claim_auto(paths: &Paths) -> bool {
    let dir = paths.backups_dir();
    fs::create_dir_all(&dir).is_ok() && fs::write(dir.join(ATTEMPT_MARKER), b"").is_ok()
}

pub(crate) fn last_auto(newest_first: &[Backup]) -> Option<i64> {
    newest_first
        .iter()
        .find(|b| b.kind == BackupKind::Auto)
        .map(|b| b.created_at)
}

/// Unpack `backup` next to the database and check it fully. Returns the unpacked file.
pub(crate) fn stage(paths: &Paths, backup: &Backup) -> Result<PathBuf> {
    let staged = paths.data_dir.join(".restore.tmp.db");
    remove_if_exists(&staged)?;
    let result = (|| {
        let source = Path::new(&backup.path);
        if backup.path.ends_with(EXT) {
            let damaged = |e: &dyn std::fmt::Display| {
                Error::validation(format!("backup {} is damaged: {e}", backup.id))
            };
            let file = BufReader::new(File::open(source)?);
            let mut decoder =
                ruzstd::decoding::StreamingDecoder::new(file).map_err(|e| damaged(&e))?;
            let mut out = File::create(&staged)?;
            io::copy(&mut decoder, &mut out).map_err(|e| damaged(&e))?;
            out.sync_all()?;
        } else {
            fs::copy(source, &staged)?;
        }
        let version = check(&staged, "integrity_check").map_err(|e| match e {
            Error::Validation(_) | Error::Db(_) => Error::validation(format!(
                "backup {} failed its integrity check, so it wasn't restored",
                backup.id
            )),
            other => other,
        })?;
        if version > SUPPORTED_SCHEMA_VERSION {
            return Err(Error::SchemaTooNew {
                db: version,
                supported: SUPPORTED_SCHEMA_VERSION,
            });
        }
        Ok(staged.clone())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&staged);
    }
    result
}

/// Put a staged backup in place of the database. Every connection to it must be closed.
pub(crate) fn swap_in(paths: &Paths, staged: &Path) -> Result<()> {
    let db = paths.db_path();
    for suffix in ["-wal", "-shm"] {
        let mut side = db.clone().into_os_string();
        side.push(suffix);
        remove_if_exists(Path::new(&side)).map_err(|_| {
            Error::conflict("the database is in use; close other Keepjay windows and try again")
        })?;
    }
    fs::rename(staged, &db).map_err(|e| match e.kind() {
        io::ErrorKind::PermissionDenied => {
            Error::conflict("the database is in use; close other Keepjay windows and try again")
        }
        _ => e.into(),
    })?;
    Ok(())
}

/// Run SQLite's `quick_check` or `integrity_check` on a database file; returns its schema
/// version.
fn check(path: &Path, pragma: &str) -> Result<u32> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let result: String = conn.query_row(&format!("PRAGMA {pragma}"), [], |r| r.get(0))?;
    if result != "ok" {
        return Err(Error::validation(format!("{pragma} failed: {result}")));
    }
    Ok(conn.query_row("PRAGMA user_version", [], |r| r.get(0))?)
}

/// `20261124-101503` (UTC).
fn stamp(now_ms: i64) -> String {
    let ts = Timestamp::from_millisecond(now_ms).unwrap_or(Timestamp::UNIX_EPOCH);
    ts.strftime("%Y%m%d-%H%M%S").to_string()
}

/// `(id, kind, created_at)` for a backup's file name; `None` for anything else.
fn parse_name(name: &str) -> Option<(String, BackupKind, i64)> {
    if name.starts_with('.') {
        return None;
    }
    if let Some(id) = name.strip_suffix(EXT) {
        let (stamp, rest) = (id.get(..15)?, id.get(15..)?.strip_prefix('-')?);
        let kind = BackupKind::ALL.into_iter().find(|k| {
            rest.strip_prefix(k.as_str()).is_some_and(|tail| {
                tail.is_empty()
                    || tail
                        .strip_prefix('-')
                        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
            })
        })?;
        return Some((id.to_string(), kind, parse_stamp(stamp)?));
    }
    let id = name.strip_suffix(LEGACY_EXT)?;
    let (_, ms) = id.strip_prefix("pre-migration-v")?.split_once('-')?;
    Some((id.to_string(), BackupKind::PreMigration, ms.parse().ok()?))
}

fn parse_stamp(s: &str) -> Option<i64> {
    let num = |range: std::ops::Range<usize>| s.get(range)?.parse::<i32>().ok();
    if s.as_bytes().get(8) != Some(&b'-') {
        return None;
    }
    let date = jiff::civil::Date::new(
        i16::try_from(num(0..4)?).ok()?,
        i8::try_from(num(4..6)?).ok()?,
        i8::try_from(num(6..8)?).ok()?,
    )
    .ok()?;
    let time = jiff::civil::Time::new(
        i8::try_from(num(9..11)?).ok()?,
        i8::try_from(num(11..13)?).ok()?,
        i8::try_from(num(13..15)?).ok()?,
        0,
    )
    .ok()?;
    let ts = TimeZone::UTC.to_timestamp(date.to_datetime(time)).ok()?;
    Some(ts.as_millisecond())
}

fn remove_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monday 2026-11-23 12:00 UTC.
    const MON: i64 = 1_795_435_200_000;

    fn auto(ms: i64) -> Backup {
        backup(BackupKind::Auto, ms)
    }

    fn backup(kind: BackupKind, ms: i64) -> Backup {
        Backup {
            id: format!("{}-{}", stamp(ms), kind.as_str()),
            kind,
            created_at: ms,
            bytes: 1,
            path: String::new(),
        }
    }

    fn newest_first(mut v: Vec<Backup>) -> Vec<Backup> {
        v.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        v
    }

    #[test]
    fn names_round_trip() {
        assert_eq!(stamp(MON), "20261123-120000");
        assert_eq!(
            parse_name("20261123-120000-auto.db.zst"),
            Some(("20261123-120000-auto".into(), BackupKind::Auto, MON))
        );
        assert_eq!(
            parse_name("20261123-120000-pre-migration-2.db.zst"),
            Some((
                "20261123-120000-pre-migration-2".into(),
                BackupKind::PreMigration,
                MON
            ))
        );
        assert_eq!(
            parse_name("pre-migration-v6-1795435200000.db"),
            Some((
                "pre-migration-v6-1795435200000".into(),
                BackupKind::PreMigration,
                MON
            ))
        );
        for junk in [
            ".last-auto-attempt",
            ".20261123-120000-auto.tmp.zst",
            "20261123-120000-autox.db.zst",
            "20261123-120000-auto-.db.zst",
            "notes.txt",
            "20261399-120000-auto.db.zst",
        ] {
            assert_eq!(parse_name(junk), None, "{junk}");
        }
    }

    #[test]
    fn keeps_seven_days_and_four_weeks_of_automatic_backups() {
        // Two a day for 60 days.
        let all = newest_first(
            (0..60)
                .flat_map(|d| [auto(MON - d * DAY_MS), auto(MON - d * DAY_MS - HOUR_MS)])
                .collect(),
        );
        let doomed: HashSet<_> = plan_prune(&all).into_iter().map(|b| b.id).collect();
        let kept: Vec<_> = all.iter().filter(|b| !doomed.contains(&b.id)).collect();
        // The newest of each of the last 7 days...
        for d in 0..7 {
            assert!(
                kept.iter().any(|b| b.created_at == MON - d * DAY_MS),
                "day {d}"
            );
        }
        // ...and of the 3 weeks before this one (Sundays, the newest in each week).
        for w in 1..4 {
            let sunday = MON - (1 + 7 * (w - 1)) * DAY_MS;
            assert!(kept.iter().any(|b| b.created_at == sunday), "week {w}");
        }
        assert_eq!(kept.len(), 7 + 2, "{kept:#?}"); // two of the weeks' Sundays are in the 7 days
    }

    #[test]
    fn never_prunes_manual_backups_and_keeps_three_of_each_safety_kind() {
        let mut all = Vec::new();
        for i in 0..5 {
            all.push(backup(BackupKind::Manual, MON - i * 30 * DAY_MS));
            all.push(backup(BackupKind::PreMigration, MON - i * DAY_MS - 1));
            all.push(backup(BackupKind::PreRestore, MON - i * DAY_MS - 2));
        }
        let all = newest_first(all);
        let doomed = plan_prune(&all);
        assert_eq!(doomed.len(), 4);
        assert!(doomed.iter().all(|b| b.kind != BackupKind::Manual));
        assert!(doomed.iter().all(|b| b.created_at < MON - 2 * DAY_MS));
    }

    fn core_with_card(dir: &Path) -> crate::Core {
        let mut core = crate::Core::open(Paths::new(dir, dir)).unwrap();
        let ws = core.init("W").unwrap();
        let ws: crate::Id = ws.id.parse().unwrap();
        core.create_deck(ws, "D").unwrap();
        add_card(&mut core, ws, "first");
        core
    }

    fn add_card(core: &mut crate::Core, ws: crate::Id, front: &str) {
        let fields = [("Front", front), ("Back", "back")]
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .into();
        let new = crate::NewFact {
            deck: "D".into(),
            card_type: crate::CardType::Basic,
            fields,
            allow_duplicate: false,
            tags: Vec::new(),
        };
        core.add_fact(ws, new).unwrap();
    }

    fn card_count(core: &crate::Core) -> i64 {
        core.conn
            .query_row("SELECT COUNT(*) FROM cards", [], |r| r.get(0))
            .unwrap()
    }

    #[test]
    fn restore_brings_back_the_data_and_backs_up_what_it_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let mut core = core_with_card(dir.path());
        let ws = core.resolve_workspace(None).unwrap();
        let first = core.create_backup(BackupKind::Manual).unwrap();
        add_card(&mut core, ws, "second");
        assert_eq!(card_count(&core), 2);

        let report = core.restore_backup(&first.id[..10]).unwrap();
        assert_eq!(report.restored.id, first.id);
        assert_eq!(report.previous.kind, BackupKind::PreRestore);
        assert_eq!(card_count(&core), 1);
        // The restored database works: it can be written to.
        add_card(&mut core, ws, "third");
        assert_eq!(card_count(&core), 2);

        // Undo: restoring the pre-restore backup brings the second card back.
        core.restore_backup(&report.previous.id).unwrap();
        assert_eq!(card_count(&core), 2);
        let fronts: Vec<String> = core
            .conn
            .prepare("SELECT fields FROM facts")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|f| f.unwrap())
            .collect();
        assert!(fronts.iter().any(|f| f.contains("second")), "{fronts:?}");
    }

    #[test]
    fn a_damaged_backup_is_refused_and_nothing_changes() {
        let dir = tempfile::tempdir().unwrap();
        let mut core = core_with_card(dir.path());
        let b = core.create_backup(BackupKind::Manual).unwrap();
        fs::write(&b.path, b"not zstd at all").unwrap();
        let err = core.restore_backup(&b.id).unwrap_err();
        assert!(matches!(err, Error::Validation(_)), "{err}");
        assert_eq!(card_count(&core), 1);
        assert!(!dir.path().join(".restore.tmp.db").exists());
    }

    #[test]
    fn automatic_backups_skip_an_empty_database_and_run_once_a_day() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path(), dir.path());
        let core = crate::Core::open(paths.clone()).unwrap();
        assert_eq!(core.auto_backup(MON).unwrap(), None);
        drop(core);

        let core = core_with_card(dir.path());
        assert!(auto_due(&paths, MON));
        let made = core.auto_backup(MON).unwrap().expect("a backup");
        assert_eq!(made.kind, BackupKind::Auto);
        assert!(!auto_due(&paths, MON + HOUR_MS));
        assert_eq!(core.auto_backup(MON + HOUR_MS).unwrap(), None);
        assert!(auto_due(&paths, MON + AUTO_EVERY_MS));

        // A claimed attempt holds off other commands for an hour, then allows a retry.
        // The marker's time is the real clock's, so this part uses it, with no backups.
        fs::remove_file(&made.path).unwrap();
        let now = crate::domain::now_ms();
        assert!(auto_due(&paths, now));
        assert!(claim_auto(&paths));
        assert!(!auto_due(&paths, now));
        assert!(auto_due(&paths, now + 2 * HOUR_MS));
    }
}
