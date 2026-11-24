// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay doctor`: database health, plus the backups (IO-004) and `kj` alias (CLI-018)
//! checks.

use serde::Serialize;
use study_core::domain::now_ms;
use study_core::{Backup, Error};

use super::Ctx;
use super::{alias, backup};
use crate::output::{CliError, mark, warn_mark};

/// `doctor --json`: the database report, plus the backups and alias checks (additive,
/// 06 §6.1). Backup and alias problems are warnings: they don't make `doctor` fail.
#[derive(Serialize)]
struct DoctorView<'a> {
    #[serde(flatten)]
    report: &'a study_core::DoctorReport,
    backups: Backups,
    alias: &'a alias::AliasStatus,
}

#[derive(Serialize)]
struct Backups {
    count: usize,
    latest: Option<Backup>,
}

/// A week without a backup is worth a warning (the automatic one is daily).
const STALE_BACKUP_MS: i64 = 7 * 24 * 3_600_000;

impl Backups {
    fn ok(&self) -> bool {
        self.latest
            .as_ref()
            .is_some_and(|b| now_ms() - b.created_at < STALE_BACKUP_MS)
    }

    fn line(&self) -> String {
        match &self.latest {
            Some(b) => format!(
                "last backup {} ({} kept)",
                backup::when(b.created_at),
                self.count
            ),
            None => "no backups yet: `keepjay backup create` makes one".into(),
        }
    }
}

pub fn run(ctx: &mut Ctx) -> Result<(), CliError> {
    let r = ctx.core.doctor()?;
    let a = alias::status()?;
    let all = ctx.core.backups()?;
    let backups = Backups {
        count: all.len(),
        latest: all.into_iter().next(),
    };
    let view = DoctorView {
        report: &r,
        backups,
        alias: &a,
    };
    ctx.out.emit(&view, || {
        format!(
            "{}  integrity {}\n{}  journal mode {}\n{}  schema v{} (this build supports v{})\n{}  {}\n{}  {}",
            mark(r.integrity == "ok"),
            r.integrity,
            mark(r.journal_mode.eq_ignore_ascii_case("wal")),
            r.journal_mode,
            mark(r.schema_version == r.supported_schema_version),
            r.schema_version,
            r.supported_schema_version,
            if view.backups.ok() { mark(true) } else { warn_mark() },
            view.backups.line(),
            if a.ok() { mark(true) } else { warn_mark() },
            a.line()
        )
    })?;
    if r.ok {
        Ok(())
    } else {
        Err(Error::Unexpected("database health check failed".into()).into())
    }
}
