// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay source add|list|rm`, `scrubjay scan` and `scrubjay stale list|keep`
//! (SRC-001…006; docs/specs/13-code-sources.md).

use std::path::PathBuf;

use clap::{Args, Subcommand};
use study_core::{NewSource, SourceKind};

use super::Ctx;
use crate::output::{CliError, truncate};

#[derive(Subcommand)]
pub enum SourceCmd {
    /// Register a folder or Git repository as a source of cards (SRC-001)
    Add(SourceAddArgs),
    /// List registered sources
    List,
    /// Unregister a source; cards already created from it are kept
    Rm {
        /// Source name or short id
        source: String,
    },
}

#[derive(Args)]
pub struct SourceAddArgs {
    /// Folder or repository to scan
    path: PathBuf,
    /// Name for the source (default: the folder's name)
    #[arg(long)]
    name: Option<String>,
    /// Deck for its cards, e.g. "Code::App" (default: the source name; created if missing)
    #[arg(short, long)]
    deck: Option<String>,
    /// Tag every card from this source (repeatable)
    #[arg(long = "tag", value_name = "TAG")]
    tags: Vec<String>,
    /// Don't write card ids back into your files (SRC-005)
    #[arg(long)]
    no_write_ids: bool,
}

#[derive(Args)]
pub struct ScanArgs {
    /// Only this source (name or short id); default: all
    source: Option<String>,
    /// Print nothing unless there are warnings (for Git hooks)
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Args)]
pub struct StaleArgs {
    #[command(subcommand)]
    cmd: Option<StaleCmd>,
}

#[derive(Subcommand)]
enum StaleCmd {
    /// List stale cards (the default)
    List {
        /// Only this source
        #[arg(long)]
        source: Option<String>,
    },
    /// The card is still right: clear the flag
    Keep {
        /// The card's id, as written in your file
        card: String,
    },
}

pub fn source(ctx: &mut Ctx, cmd: SourceCmd) -> Result<(), CliError> {
    match cmd {
        SourceCmd::Add(args) => {
            let ws = ctx.workspace_for_new()?;
            let s = ctx.core.add_source(
                ws,
                NewSource {
                    path: args.path,
                    name: args.name,
                    deck: args.deck,
                    tags: args.tags,
                    write_ids: !args.no_write_ids,
                },
            )?;
            ctx.out.emit(&s, || {
                let identity = match (&s.kind, &s.repo) {
                    (SourceKind::Repo, Some(root)) => {
                        format!(
                            "Git repository, root commit {}",
                            &root[..12.min(root.len())]
                        )
                    }
                    (SourceKind::Repo, None) => "Git repository with no commits yet".to_string(),
                    (SourceKind::Folder, _) => "folder".to_string(),
                };
                let ids = if s.write_ids {
                    "card ids will be written into your files"
                } else {
                    "card ids won't be written into your files"
                };
                format!(
                    "Added source '{}' ({}): {}\n  Path      {}\n  Deck      {}\n  Ids       {ids}",
                    s.name, s.short_id, identity, s.path, s.deck
                )
            })
        }
        SourceCmd::List => {
            let ws = ctx.workspace()?;
            let sources = ctx.core.list_sources(ws)?;
            ctx.out.emit(&sources, || {
                if sources.is_empty() {
                    return "No sources yet. Add one with `scrubjay source add <path>`."
                        .to_string();
                }
                let width = sources
                    .iter()
                    .map(|s| s.name.chars().count())
                    .max()
                    .unwrap_or(0);
                sources
                    .iter()
                    .map(|s| {
                        format!(
                            "{}  {:width$}  {:6}  {}  → {}",
                            s.short_id,
                            s.name,
                            s.kind.as_str(),
                            s.path,
                            s.deck
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        SourceCmd::Rm { source } => {
            let ws = ctx.workspace()?;
            let s = ctx.core.remove_source(ws, &source)?;
            ctx.out.emit(&s, || {
                format!(
                    "Removed source '{}'. Cards already created from it are kept.",
                    s.name
                )
            })
        }
    }
}

pub fn scan(ctx: &mut Ctx, args: ScanArgs) -> Result<(), CliError> {
    let ws = ctx.workspace()?;
    let reports = match args.source {
        Some(key) => vec![ctx.core.scan_source(ws, &key)?],
        None => ctx.core.scan_all(ws)?,
    };
    if args.quiet && !ctx.out.json {
        for r in &reports {
            for w in &r.warnings {
                eprintln!("scrubjay: {}:{}: {}", w.path, w.line, w.message);
            }
        }
        return Ok(());
    }
    ctx.out.emit(&reports, || scan_report(&reports))
}

pub fn stale(ctx: &mut Ctx, args: StaleArgs) -> Result<(), CliError> {
    let ws = ctx.workspace()?;
    match args.cmd.unwrap_or(StaleCmd::List { source: None }) {
        StaleCmd::List { source } => {
            let stale = ctx.core.list_stale(ws, source.as_deref())?;
            ctx.out.emit(&stale, || {
                if stale.is_empty() {
                    return "No stale cards: everything matches its code.".to_string();
                }
                let mut lines: Vec<String> = stale
                    .iter()
                    .map(|s| {
                        format!(
                            "{}  {}:{}  {}",
                            s.id,
                            s.path,
                            s.line,
                            truncate(&s.question, 60)
                        )
                    })
                    .collect();
                lines.push(
                    "Still right? `scrubjay stale keep <id>`. Otherwise edit or delete the card in its file, then `scrubjay scan`."
                        .into(),
                );
                lines.join("\n")
            })
        }
        StaleCmd::Keep { card } => {
            let s = ctx.core.keep_stale(ws, &card)?;
            ctx.out
                .emit(&s, || format!("Kept {} ({}:{}).", s.id, s.path, s.line))
        }
    }
}

fn scan_report(reports: &[study_core::ScanReport]) -> String {
    if reports.is_empty() {
        return "No sources yet. Add one with `scrubjay source add <path>`.".to_string();
    }
    let mut lines = Vec::new();
    for r in reports {
        let mut parts = Vec::new();
        for (n, what) in [
            (r.added, "new"),
            (r.updated, "updated"),
            (r.restored, "restored"),
            (r.retired, "retired"),
        ] {
            if n > 0 {
                parts.push(format!("{n} {what}"));
            }
        }
        let changes = if parts.is_empty() {
            "no changes".to_string()
        } else {
            parts.join(", ")
        };
        let plural = if r.cards_found == 1 { "" } else { "s" };
        lines.push(format!(
            "{}: {} card{plural} in {} files ({changes}), {} ms",
            r.source, r.cards_found, r.files_parsed, r.elapsed_ms
        ));
        if r.ids_written > 0 {
            lines.push(format!(
                "  Wrote {} card ids into your files.",
                r.ids_written
            ));
        }
        if r.became_stale > 0 {
            lines.push(format!(
                "  {} card{} now stale: the code changed. See `scrubjay stale`.",
                r.became_stale,
                if r.became_stale == 1 { " is" } else { "s are" }
            ));
        } else if r.stale > 0 {
            lines.push(format!("  {} stale (see `scrubjay stale`).", r.stale));
        }
        if r.skipped_large + r.skipped_binary > 0 {
            lines.push(format!(
                "  Skipped {} large and {} binary or non-UTF-8 files.",
                r.skipped_large, r.skipped_binary
            ));
        }
        for w in &r.warnings {
            lines.push(format!("  warning: {}:{}: {}", w.path, w.line, w.message));
        }
    }
    lines.join("\n")
}
