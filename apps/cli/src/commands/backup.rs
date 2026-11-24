// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay backup create|list|restore` (IO-004, NFR-REL-002) and the daily automatic
//! backup: after a command, [`spawn_if_due`] starts a detached `keepjay backup create
//! --auto` when one is due, so no command waits for it.

use std::io::{BufRead, IsTerminal, Write};

use clap::Subcommand;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use serde::Serialize;
use study_core::backup;
use study_core::domain::now_ms;
use study_core::{Backup, BackupKind, Paths};

use super::Ctx;
use crate::output::CliError;

/// Set to turn the automatic daily backup off (tests, or people who back up their own way).
pub const NO_AUTO_ENV: &str = "KEEPJAY_NO_AUTO_BACKUP";

#[derive(Subcommand)]
pub enum BackupCmd {
    /// Back up your cards and review history now
    Create {
        /// Run as the detached daily backup started after a command
        #[arg(long, hide = true)]
        auto: bool,
    },
    /// List backups, newest first
    List,
    /// Replace your data with a backup. Your current data is backed up first.
    Restore {
        /// The backup's id from `keepjay backup list` (the start of it is enough)
        id: String,
        /// Don't ask first (needed without a terminal)
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Serialize)]
struct Listing<'a> {
    dir: String,
    backups: &'a [Backup],
}

pub fn run(ctx: &mut Ctx, cmd: BackupCmd) -> Result<(), CliError> {
    match cmd {
        BackupCmd::Create { auto: true } => {
            let made = ctx.core.auto_backup(now_ms())?;
            ctx.out.emit(&made, || match &made {
                Some(b) => format!("Backed up to {}.", b.id),
                None => "No backup needed.".into(),
            })
        }
        BackupCmd::Create { auto: false } => {
            let b = ctx.core.create_backup(BackupKind::Manual)?;
            ctx.out.emit(&b, || {
                format!("Backed up to {} ({}).\n{}", b.id, size(b.bytes), b.path)
            })
        }
        BackupCmd::List => {
            let backups = ctx.core.backups()?;
            let listing = Listing {
                dir: ctx.core.paths().backups_dir().display().to_string(),
                backups: &backups,
            };
            ctx.out.emit(&listing, || describe(&listing))
        }
        BackupCmd::Restore { id, yes } => {
            let target = backup::find(&ctx.core.paths().backups_dir(), &id)?;
            if !yes {
                let interactive = std::io::stdin().is_terminal() && !ctx.out.json;
                if !interactive {
                    return Err(CliError::Usage(
                        "`backup restore` replaces your data; add --yes to confirm".into(),
                    ));
                }
                let mut err = std::io::stderr().lock();
                write!(
                    err,
                    "Replace your cards and review history with backup {} ({})?\n\
                     Your current data is backed up first, so you can undo this. \
                     Close any other Keepjay windows first.\nRestore? [y/N] ",
                    target.id,
                    when(target.created_at)
                )?;
                err.flush()?;
                let mut answer = String::new();
                std::io::stdin().lock().read_line(&mut answer)?;
                if !matches!(answer.trim(), "y" | "Y" | "yes") {
                    eprintln!("Nothing changed.");
                    return Ok(());
                }
            }
            let r = ctx.core.restore_backup(&target.id)?;
            ctx.out.emit(&r, || {
                format!(
                    "Restored {} ({}).\nYour previous data is in backup {}: `keepjay backup restore {}` undoes this.",
                    r.restored.id,
                    when(r.restored.created_at),
                    r.previous.id,
                    r.previous.id
                )
            })
        }
    }
}

fn describe(l: &Listing) -> String {
    if l.backups.is_empty() {
        return "No backups yet. Keepjay makes one each day you use it once you have cards; \
                make one now with `keepjay backup create`."
            .into();
    }
    let width = l.backups.iter().map(|b| b.id.len()).max().unwrap_or(0);
    let mut out = format!("{:width$}  {:13}  {:16}  SIZE\n", "ID", "KIND", "TAKEN");
    for b in l.backups {
        out.push_str(&format!(
            "{:width$}  {:13}  {:16}  {}\n",
            b.id,
            b.kind.as_str(),
            when(b.created_at),
            size(b.bytes)
        ));
    }
    out.push_str(&format!(
        "\nIn {}. Restore one with `keepjay backup restore ID`.",
        l.dir
    ));
    out
}

/// `2026-11-24 10:15`, in local time.
pub fn when(ms: i64) -> String {
    let ts = Timestamp::from_millisecond(ms).unwrap_or(Timestamp::UNIX_EPOCH);
    TimeZone::system()
        .to_datetime(ts)
        .strftime("%Y-%m-%d %H:%M")
        .to_string()
}

fn size(bytes: u64) -> String {
    match bytes {
        b if b < 1000 => format!("{b} B"),
        b if b < 1_000_000 => format!("{:.0} KB", b as f64 / 1e3),
        b if b < 1_000_000_000 => format!("{:.1} MB", b as f64 / 1e6),
        b => format!("{:.1} GB", b as f64 / 1e9),
    }
}

/// After a command: start the daily backup in the background if one is due.
pub fn spawn_if_due(paths: &Paths) {
    if std::env::var_os(NO_AUTO_ENV).is_some_and(|v| !v.is_empty()) {
        return;
    }
    if backup::auto_due(paths, now_ms()) && backup::claim_auto(paths) {
        super::spawn_detached(paths, &["backup", "create", "--auto"]);
    }
}
