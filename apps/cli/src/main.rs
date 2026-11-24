// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay` — the Keepjay command-line tool (docs/specs/06-cli.md).
//!
//! Commands so far: `status`, `init`, `workspace`, `deck`, `add card`, `tag`, `review`,
//! `while`, `import anki`, `source`, `scan`, `stale`, `alias`, `backup`, `doctor`, `telemetry`. Each lives in a
//! module under `commands/`; this file parses the command line and dispatches.
//! Every command follows the JSON envelope (06 §6.1) and exit codes (06 §6.3).

mod commands;
mod output;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use study_core::{Core, Paths};

use commands::{
    Ctx, alias, backup, cards, doctor, import, review, sources, status, telemetry, while_cmd,
    workspace,
};
use output::{CliError, Output};

#[derive(Parser)]
#[command(name = "keepjay", version, about = "Keepjay: keep what you learn while you code.", long_about = None)]
struct Cli {
    /// Output JSON (shorthand for --format json)
    #[arg(long, global = true)]
    json: bool,

    /// Don't use colour (as does setting NO_COLOR)
    #[arg(long, global = true)]
    no_color: bool,

    /// Override the data directory
    #[arg(long, global = true, value_name = "PATH", env = "KEEPJAY_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Workspace to act on (default: the first one). For `init`, the name to create.
    #[arg(
        short = 'w',
        long,
        global = true,
        value_name = "NAME",
        env = "KEEPJAY_WORKSPACE"
    )]
    workspace: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show today's summary
    Status(status::StatusArgs),
    /// First-run setup: create a workspace (named by --workspace) if none exists
    Init,
    /// Manage workspaces
    #[command(subcommand)]
    Workspace(workspace::WorkspaceCmd),
    /// Manage decks
    #[command(subcommand)]
    Deck(cards::DeckCmd),
    /// Add things
    #[command(subcommand)]
    Add(cards::AddCmd),
    /// Manage tags
    #[command(subcommand)]
    Tag(cards::TagCmd),
    /// Review due cards (interactive)
    Review(review::ReviewArgs),
    /// Run a command and review while it runs, e.g. `keepjay while -- cargo test`
    While(while_cmd::WhileArgs),
    /// Import from other apps
    #[command(subcommand)]
    Import(import::ImportCmd),
    /// Register folders and repositories to scan for cards
    #[command(subcommand)]
    Source(sources::SourceCmd),
    /// Find @card comments and card files in your sources and update your cards
    Scan(sources::ScanArgs),
    /// Cards whose code changed since you wrote them
    Stale(sources::StaleArgs),
    /// Install or remove the `kj` short alias (CLI-018)
    Alias(alias::AliasArgs),
    /// Back up your data, list backups, or restore one
    #[command(subcommand)]
    Backup(backup::BackupCmd),
    /// Check the data directory and database health
    Doctor,
    /// The opt-in anonymous weekly usage ping (off unless you turn it on)
    #[command(subcommand)]
    Telemetry(telemetry::TelemetryCmd),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    if cli.no_color {
        output::disable_color();
    }
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(CliError::BrokenPipe) => ExitCode::SUCCESS,
        Err(err) => ExitCode::from(err.report(json)),
    }
}

/// Runs the command; returns the process exit code (non-zero only from `while`).
fn run(cli: Cli) -> Result<u8, CliError> {
    let mut core = Core::open(Paths::resolve(cli.data_dir.as_deref()))?;
    let out = Output { json: cli.json };
    // The pre-rename variable (D-037) still works; `--workspace` and KEEPJAY_WORKSPACE win.
    let ws_name = cli.workspace.clone().or_else(|| {
        std::env::var("SCRUBJAY_WORKSPACE")
            .ok()
            .filter(|v| !v.is_empty())
    });
    let mut ctx = Ctx {
        core: &mut core,
        out: &out,
        ws_name: ws_name.as_deref(),
    };
    let ctx = &mut ctx;
    let command = cli.command.unwrap_or(Command::Status(Default::default()));
    // Background work never starts more background work: the ping's own commands never
    // start a ping (D-040), and backup commands never start the daily backup.
    let background = !matches!(command, Command::Telemetry(_) | Command::Backup(_));
    let code = dispatch(ctx, command, cli.json)?;
    if background {
        telemetry::spawn_if_due(core.paths());
        backup::spawn_if_due(core.paths());
    }
    Ok(code)
}

fn dispatch(ctx: &mut Ctx, command: Command, json: bool) -> Result<u8, CliError> {
    match command {
        Command::While(args) => return while_cmd::command(ctx, args),
        Command::Status(args) => status::run(ctx, args),
        Command::Init => workspace::init(ctx).and_then(|()| telemetry::ask_once(ctx, json)),
        Command::Workspace(cmd) => workspace::run(ctx, cmd),
        Command::Deck(cmd) => cards::deck(ctx, cmd),
        Command::Add(cmd) => cards::add(ctx, cmd),
        Command::Tag(cmd) => cards::tag(ctx, cmd),
        Command::Review(args) => review::command(ctx, args),
        Command::Import(cmd) => import::run(ctx, cmd),
        Command::Source(cmd) => {
            // `source add` is every new user's first step, so it asks too (D-040).
            let ask = matches!(cmd, sources::SourceCmd::Add(_));
            sources::source(ctx, cmd).and_then(|()| match ask {
                true => telemetry::ask_once(ctx, json),
                false => Ok(()),
            })
        }
        Command::Scan(args) => sources::scan(ctx, args),
        Command::Stale(args) => sources::stale(ctx, args),
        Command::Alias(args) => alias::run(ctx, args),
        Command::Backup(cmd) => backup::run(ctx, cmd),
        Command::Doctor => doctor::run(ctx),
        Command::Telemetry(cmd) => telemetry::run(ctx, cmd),
    }?;
    Ok(0)
}
