// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay doctor`: database health, plus the `kj` alias check (CLI-018).

use serde::Serialize;
use study_core::Error;

use super::Ctx;
use super::alias;
use crate::output::{CliError, mark, warn_mark};

/// `doctor --json`: the database report, plus the alias check (additive, 06 §6.1).
/// Alias problems are warnings: they don't make `doctor` fail.
#[derive(Serialize)]
struct DoctorView<'a> {
    #[serde(flatten)]
    report: &'a study_core::DoctorReport,
    alias: &'a alias::AliasStatus,
}

pub fn run(ctx: &mut Ctx) -> Result<(), CliError> {
    let r = ctx.core.doctor()?;
    let a = alias::status()?;
    let view = DoctorView {
        report: &r,
        alias: &a,
    };
    ctx.out.emit(&view, || {
        format!(
            "{}  integrity {}\n{}  journal mode {}\n{}  schema v{} (this build supports v{})\n{}  {}",
            mark(r.integrity == "ok"),
            r.integrity,
            mark(r.journal_mode.eq_ignore_ascii_case("wal")),
            r.journal_mode,
            mark(r.schema_version == r.supported_schema_version),
            r.schema_version,
            r.supported_schema_version,
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
