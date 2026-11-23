// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay` — the Scrubjay command-line tool (docs/specs/06-cli.md).
//!
//! Commands so far: `status`, `init`, `workspace`, `deck`, `add card`, `tag`, `review`,
//! `while`, `import anki`, `source`, `scan`, `stale`, `doctor`.
//! Every command follows the JSON envelope (06 §6.1) and exit codes (06 §6.3).

mod review;
mod while_cmd;

use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use study_core::{CardType, Core, Error, NewFact, NewSource, Paths};

const SCHEMA: &str = "scrubjay.cli/1";

/// The workspace `init` creates when none is named.
const DEFAULT_WORKSPACE: &str = "My Studies";

#[derive(Parser)]
#[command(name = "scrubjay", version, about = "Scrubjay: keep what you learn while you code.", long_about = None)]
struct Cli {
    /// Output JSON (shorthand for --format json)
    #[arg(long, global = true)]
    json: bool,

    /// Override the data directory
    #[arg(long, global = true, value_name = "PATH", env = "SCRUBJAY_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Workspace to act on (default: the first one). For `init`, the name to create.
    #[arg(
        short = 'w',
        long,
        global = true,
        value_name = "NAME",
        env = "SCRUBJAY_WORKSPACE"
    )]
    workspace: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show today's summary
    Status {
        /// One line from a template, for shell prompts and status bars (CLI-021).
        /// Fields: {due} {review} {learning} {new} {stale} {reviewed}; `{{` for a brace
        #[arg(long, value_name = "TEMPLATE")]
        format: Option<String>,
    },
    /// First-run setup: create a workspace (named by --workspace) if none exists
    Init,
    /// Manage workspaces
    #[command(subcommand)]
    Workspace(WorkspaceCmd),
    /// Manage decks
    #[command(subcommand)]
    Deck(DeckCmd),
    /// Add things
    #[command(subcommand)]
    Add(AddCmd),
    /// Manage tags
    #[command(subcommand)]
    Tag(TagCmd),
    /// Review due cards (interactive)
    Review(review::ReviewArgs),
    /// Run a command and review while it runs, e.g. `scrubjay while -- cargo test`
    While(while_cmd::WhileArgs),
    /// Import from other apps
    #[command(subcommand)]
    Import(ImportCmd),
    /// Register folders and repositories to scan for cards
    #[command(subcommand)]
    Source(SourceCmd),
    /// Find @card comments and card files in your sources and update your cards
    Scan {
        /// Only this source (name or short id); default: all
        source: Option<String>,
        /// Print nothing unless there are warnings (for Git hooks)
        #[arg(short, long)]
        quiet: bool,
    },
    /// Cards whose code changed since you wrote them
    Stale(StaleArgs),
    /// Check the data directory and database health
    Doctor,
}

#[derive(Subcommand)]
enum WorkspaceCmd {
    /// List workspaces
    List,
    /// Create a workspace
    Create { name: String },
}

#[derive(Subcommand)]
enum DeckCmd {
    /// List decks with their card counts
    List,
    /// Create a deck; `Parent::Child` creates missing parents too
    Create { path: String },
}

#[derive(Subcommand)]
enum AddCmd {
    /// Add a card (a fact and the cards its card type generates)
    Card(AddCardArgs),
}

#[derive(Subcommand)]
enum TagCmd {
    /// List tags with how many facts carry each
    List,
}

#[derive(Subcommand)]
enum SourceCmd {
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
struct StaleArgs {
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

#[derive(Args)]
struct SourceAddArgs {
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

#[derive(Subcommand)]
enum ImportCmd {
    /// Import an Anki package (.apkg / .colpkg) with its review history
    Anki {
        /// Path to the .apkg or .colpkg file
        file: PathBuf,
        /// Put imported decks under this deck, e.g. --into "From Anki"
        #[arg(long, value_name = "DECK")]
        into: Option<String>,
    },
}

#[derive(Args)]
struct AddCardArgs {
    /// First field (Front, or Text for cloze)
    first: Option<String>,
    /// Second field (Back, or Extra for cloze)
    second: Option<String>,
    /// Deck path, e.g. "Biology::Cells"
    #[arg(short, long)]
    deck: String,
    /// Card type: basic, basic-reverse, optional-reverse, cloze, type-in
    #[arg(short = 't', long = "type", default_value = "basic")]
    card_type: String,
    /// Set any field by name, e.g. --field "Add Reverse=y" (repeatable)
    #[arg(long = "field", value_name = "NAME=VALUE")]
    fields: Vec<String>,
    /// Add even if an identical first field exists in the deck
    #[arg(long)]
    allow_duplicate: bool,
    /// Tag the card, e.g. --tag bio::cells (repeatable)
    #[arg(long = "tag", value_name = "TAG")]
    tags: Vec<String>,
}

/// CLI-level errors: core errors plus usage problems found after parsing (exit 2).
pub enum CliError {
    Core(Error),
    Usage(String),
    /// The reader closed the pipe (e.g. `| head`): stop quietly, like other CLIs.
    BrokenPipe,
}

impl From<Error> for CliError {
    fn from(e: Error) -> Self {
        CliError::Core(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            CliError::BrokenPipe
        } else {
            CliError::Core(e.into())
        }
    }
}

impl CliError {
    /// (JSON error code, exit code) per 06 §6.3.
    fn codes(&self) -> (String, u8) {
        use study_core::ErrorKind as K;
        match self {
            CliError::Usage(_) => ("usage".into(), 2),
            CliError::BrokenPipe => ("broken_pipe".into(), 0),
            CliError::Core(e) => {
                let exit = match e.kind() {
                    K::Unexpected => 1,
                    K::NotFound => 3,
                    K::Validation => 4,
                    K::Conflict => 5,
                    K::VersionIncompatible => 6,
                };
                let code = serde_json::to_value(e.kind())
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string))
                    .unwrap_or_else(|| "unexpected".into());
                (code, exit)
            }
        }
    }

    fn message(&self) -> String {
        match self {
            CliError::Core(e) => e.to_string(),
            CliError::Usage(m) => m.clone(),
            CliError::BrokenPipe => String::new(),
        }
    }
}

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    schema: &'static str,
    ok: bool,
    data: T,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    schema: &'static str,
    ok: bool,
    error: ErrorBody,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json = cli.json;
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(CliError::BrokenPipe) => ExitCode::SUCCESS,
        Err(err) => {
            let (code, exit) = err.codes();
            eprintln!("scrubjay: {}", err.message());
            if json {
                let body = ErrorEnvelope {
                    schema: SCHEMA,
                    ok: false,
                    error: ErrorBody {
                        code,
                        message: err.message(),
                    },
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&body).expect("serialisable")
                );
            }
            ExitCode::from(exit)
        }
    }
}

/// Runs the command; returns the process exit code (non-zero only from `while`).
fn run(cli: Cli) -> Result<u8, CliError> {
    let paths = Paths::resolve(cli.data_dir.as_deref());
    let mut core = Core::open(paths)?;
    let out = Output { json: cli.json };
    let ws_name = cli.workspace.as_deref();
    let command = cli.command.unwrap_or(Command::Status { format: None });
    if let Command::While(args) = &command {
        let ws = core.resolve_workspace(ws_name)?;
        return while_cmd::run(&mut core, ws, args, cli.json);
    }
    match command {
        Command::While(_) => unreachable!("handled above"),
        Command::Status { format } => {
            let system = core.status()?;
            let today = match core.resolve_workspace(ws_name) {
                Ok(ws) => Some(core.today(ws, review::now_ms())?),
                Err(e) if e.kind() == study_core::ErrorKind::NotFound && ws_name.is_none() => None,
                Err(e) => return Err(e.into()),
            };
            if let Some(template) = format.filter(|_| !cli.json) {
                // Nothing at all before `init`, so prompts stay clean.
                let line = match &today {
                    Some(t) => status_line(&template, t)?,
                    None => String::new(),
                };
                println!("{line}");
                return Ok(0);
            }
            let view = StatusView { system, today };
            out.emit(&view, || view.human())
        }
        Command::Init => {
            let ws = core.init(ws_name.unwrap_or(DEFAULT_WORKSPACE))?;
            out.emit(&ws, || {
                format!("Ready. Workspace '{}' ({})", ws.name, ws.short_id)
            })
        }
        Command::Workspace(WorkspaceCmd::List) => {
            let list = core.list_workspaces()?;
            out.emit(&list, || {
                if list.is_empty() {
                    "No workspaces yet. Run `scrubjay init`.".to_string()
                } else {
                    list.iter()
                        .map(|w| format!("{}  {}", w.short_id, w.name))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            })
        }
        Command::Workspace(WorkspaceCmd::Create { name }) => {
            let ws = core.create_workspace(&name)?;
            out.emit(&ws, || {
                format!("Created workspace '{}' ({})", ws.name, ws.short_id)
            })
        }
        Command::Deck(DeckCmd::List) => {
            let ws = core.resolve_workspace(ws_name)?;
            let decks = core.list_decks(ws)?;
            out.emit(&decks, || {
                if decks.is_empty() {
                    return "No decks yet. Create one with `scrubjay deck create <name>`."
                        .to_string();
                }
                decks
                    .iter()
                    .map(|d| {
                        let depth = d.path.matches("::").count();
                        format!(
                            "{}  {}{}  ({} cards)",
                            d.short_id,
                            "  ".repeat(depth),
                            d.name,
                            d.cards
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        Command::Deck(DeckCmd::Create { path }) => {
            let ws = workspace_for_new(&mut core, ws_name)?;
            let deck = core.create_deck(ws, &path)?;
            out.emit(&deck, || {
                format!("Created deck '{}' ({})", deck.path, deck.short_id)
            })
        }
        Command::Add(AddCmd::Card(args)) => {
            let ws = workspace_for_new(&mut core, ws_name)?;
            let new = new_fact(args)?;
            let deck = new.deck.clone();
            let fact = core.add_fact(ws, new)?;
            out.emit(&fact, || {
                let n = fact.cards.len();
                let ids: Vec<_> = fact.cards.iter().map(|c| c.short_id.as_str()).collect();
                let tags = if fact.tags.is_empty() {
                    String::new()
                } else {
                    format!(" (tags: {})", fact.tags.join(", "))
                };
                format!(
                    "Added {n} card{} to {deck}: {}{tags}",
                    if n == 1 { "" } else { "s" },
                    ids.join(", ")
                )
            })
        }
        Command::Tag(TagCmd::List) => {
            let ws = core.resolve_workspace(ws_name)?;
            let tags = core.list_tags(ws)?;
            out.emit(&tags, || {
                if tags.is_empty() {
                    return "No tags yet. Add one with `scrubjay add card … --tag <tag>`."
                        .to_string();
                }
                let width = tags
                    .iter()
                    .map(|t| t.path.chars().count())
                    .max()
                    .unwrap_or(0);
                tags.iter()
                    .map(|t| {
                        let plural = if t.facts == 1 { "" } else { "s" };
                        format!("{:width$}  {} fact{plural}", t.path, t.facts)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        Command::Review(args) => {
            let ws = core.resolve_workspace(ws_name)?;
            let summary = review::run(&mut core, ws, &args)?;
            if args.quick && summary.reviewed == 0 && !cli.json {
                return Ok(0); // silent when nothing was due (CLI-019)
            }
            out.emit(&summary, || summary.to_string())
        }
        Command::Import(ImportCmd::Anki { file, into }) => {
            let ws = workspace_for_new(&mut core, ws_name)?;
            let options = study_core::AnkiImportOptions { deck_prefix: into };
            let s = core.import_anki(ws, &file, &options)?;
            out.emit(&s, || import_report(&s))
        }
        Command::Source(SourceCmd::Add(args)) => {
            let ws = workspace_for_new(&mut core, ws_name)?;
            let s = core.add_source(
                ws,
                NewSource {
                    path: args.path,
                    name: args.name,
                    deck: args.deck,
                    tags: args.tags,
                    write_ids: !args.no_write_ids,
                },
            )?;
            out.emit(&s, || {
                let identity = match (&s.kind, &s.repo) {
                    (study_core::SourceKind::Repo, Some(root)) => {
                        format!(
                            "Git repository, root commit {}",
                            &root[..12.min(root.len())]
                        )
                    }
                    (study_core::SourceKind::Repo, None) => {
                        "Git repository with no commits yet".to_string()
                    }
                    (study_core::SourceKind::Folder, _) => "folder".to_string(),
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
        Command::Source(SourceCmd::List) => {
            let ws = core.resolve_workspace(ws_name)?;
            let sources = core.list_sources(ws)?;
            out.emit(&sources, || {
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
        Command::Source(SourceCmd::Rm { source }) => {
            let ws = core.resolve_workspace(ws_name)?;
            let s = core.remove_source(ws, &source)?;
            out.emit(&s, || {
                format!(
                    "Removed source '{}'. Cards already created from it are kept.",
                    s.name
                )
            })
        }
        Command::Scan { source, quiet } => {
            let ws = core.resolve_workspace(ws_name)?;
            let reports = match source {
                Some(key) => vec![core.scan_source(ws, &key)?],
                None => core.scan_all(ws)?,
            };
            if quiet && !out.json {
                for r in &reports {
                    for w in &r.warnings {
                        eprintln!("scrubjay: {}:{}: {}", w.path, w.line, w.message);
                    }
                }
                return Ok(0);
            }
            out.emit(&reports, || scan_report(&reports))
        }
        Command::Stale(StaleArgs { cmd }) => {
            let ws = core.resolve_workspace(ws_name)?;
            match cmd.unwrap_or(StaleCmd::List { source: None }) {
                StaleCmd::List { source } => {
                    let stale = core.list_stale(ws, source.as_deref())?;
                    out.emit(&stale, || {
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
                    let s = core.keep_stale(ws, &card)?;
                    out.emit(&s, || format!("Kept {} ({}:{}).", s.id, s.path, s.line))
                }
            }
        }
        Command::Doctor => {
            let r = core.doctor()?;
            out.emit(&r, || {
                format!(
                    "{}  integrity {}\n{}  journal mode {}\n{}  schema v{} (this build supports v{})",
                    mark(r.integrity == "ok"),
                    r.integrity,
                    mark(r.journal_mode.eq_ignore_ascii_case("wal")),
                    r.journal_mode,
                    mark(r.schema_version == r.supported_schema_version),
                    r.schema_version,
                    r.supported_schema_version
                )
            })?;
            if r.ok {
                Ok(())
            } else {
                Err(Error::Unexpected("database health check failed".into()).into())
            }
        }
    }
    .map(|()| 0)
}

/// The workspace for a command that adds things (sources, decks, cards, imports). On a
/// fresh install it creates the default workspace first, so `scrubjay source add .` works
/// straight after installing (J15) without a separate `init`.
fn workspace_for_new(core: &mut Core, name: Option<&str>) -> Result<study_core::Id, CliError> {
    match core.resolve_workspace(name) {
        Err(e)
            if e.kind() == study_core::ErrorKind::NotFound
                && name.is_none()
                && core.list_workspaces()?.is_empty() =>
        {
            let ws = core.init(DEFAULT_WORKSPACE)?;
            eprintln!("Created your workspace '{}'.", ws.name);
            Ok(ws.id.parse()?)
        }
        other => Ok(other?),
    }
}

/// Fill a `status --format` template (CLI-021).
fn status_line(template: &str, t: &study_core::Today) -> Result<String, CliError> {
    let mut out = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                out.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                out.push('}');
            }
            '{' => {
                let name: String = chars.by_ref().take_while(|&c| c != '}').collect();
                let value = match name.as_str() {
                    "due" => t.due.review + t.due.learning,
                    "review" => t.due.review,
                    "learning" => t.due.learning,
                    "new" => t.due.new,
                    "stale" => t.stale,
                    "reviewed" => t.reviewed,
                    _ => {
                        return Err(CliError::Usage(format!(
                            "unknown field {{{name}}} in --format; use {{due}}, {{review}}, \
                             {{learning}}, {{new}}, {{stale}} or {{reviewed}}"
                        )));
                    }
                };
                out.push_str(&value.to_string());
            }
            c => out.push(c),
        }
    }
    Ok(out)
}

/// `scrubjay status`: system info (unchanged fields) plus today's summary (additive).
#[derive(Serialize)]
struct StatusView {
    #[serde(flatten)]
    system: study_core::Status,
    today: Option<study_core::Today>,
}

impl StatusView {
    fn human(&self) -> String {
        let s = &self.system;
        let footer = format!("  Data      {} (schema v{})", s.data_dir, s.schema_version);
        let Some(t) = &self.today else {
            return format!(
                "Scrubjay {}\n  Next      run `scrubjay init` to create your first workspace\n{footer}",
                s.core_version
            );
        };
        let due = &t.due;
        let parts: Vec<String> = [
            (due.review, "due"),
            (due.learning, "learning"),
            (due.new, "new"),
        ]
        .iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, what)| format!("{n} {what}"))
        .collect();
        let cards = if parts.is_empty() {
            "all caught up".to_string()
        } else {
            parts.join(" · ")
        };
        let done = match (t.reviewed, t.answer_time_ms) {
            (0, _) => "nothing reviewed yet".to_string(),
            (n, ms) if ms < 60_000 => format!("{n} reviewed in under a minute"),
            (n, ms) => format!("{n} reviewed in {} min", (ms as f64 / 60_000.0).round()),
        };
        let now = review::now_ms();
        let learning_now = t.next_learning_due.is_some_and(|at| at <= now);
        let next = if due.review + due.new > 0 || learning_now {
            "scrubjay review".to_string()
        } else if let Some(at) = t.next_learning_due {
            let wait = (at - now).max(0) as f64 / 86_400_000.0;
            format!("next learning card in {}", review::fmt_interval(wait))
        } else {
            "add cards with `scrubjay add card`, or come back tomorrow".to_string()
        };
        format!(
            "Today · {}{:>pad$}\n  Cards     {cards}\n  Done      {done}\n  Next      {next}\n{footer}",
            t.workspace,
            t.label,
            pad = 44usize.saturating_sub(t.workspace.len())
        )
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

fn truncate(s: &str, max: usize) -> String {
    let one_line = s.lines().next().unwrap_or("");
    if one_line.chars().count() <= max && !s.contains('\n') {
        one_line.to_string()
    } else {
        let cut: String = one_line.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

fn import_report(s: &study_core::AnkiImportSummary) -> String {
    let mut lines = vec![format!(
        "Imported {} notes → {} cards with {} past reviews ({} Anki format).",
        s.notes_imported, s.cards_imported, s.review_logs_imported, s.format
    )];
    if s.notes_already_imported > 0 {
        lines.push(format!(
            "  Skipped {} notes already imported earlier.",
            s.notes_already_imported
        ));
    }
    if !s.decks_created.is_empty() {
        lines.push(format!("  Decks: {}", s.decks_created.join(", ")));
    }
    for t in &s.note_types {
        let plural = if t.notes == 1 { "" } else { "s" };
        lines.push(format!(
            "  {} ({} note{plural}) → {}",
            t.name, t.notes, t.imported_as
        ));
    }
    if s.suspended > 0 {
        lines.push(format!("  {} cards stay suspended.", s.suspended));
    }
    for (reason, n) in &s.skipped_reasons {
        lines.push(format!("  Skipped {n} notes: {reason}."));
    }
    if s.cards_reset_without_history > 0 {
        lines.push(format!(
            "  {} studied cards had no review history and start as new.",
            s.cards_reset_without_history
        ));
    }
    if s.notes_tagged > 0 {
        lines.push(format!(
            "  Tags on {} notes ({} new tags).",
            s.notes_tagged, s.tags_created
        ));
    }
    if s.tags_skipped > 0 {
        lines.push(format!(
            "  Skipped {} tags that aren't valid here.",
            s.tags_skipped
        ));
    }
    if s.media_files > 0 {
        let plural = if s.media_added == 1 { "" } else { "s" };
        let mut media = format!("  Media: {} file{plural} added", s.media_added);
        if s.media_already_present > 0 {
            media += &format!(", {} already here", s.media_already_present);
        }
        if s.media_renamed > 0 {
            media += &format!(
                ", {} renamed because the name was taken by another file",
                s.media_renamed
            );
        }
        lines.push(media + ".");
    }
    if s.media_skipped > 0 {
        lines.push(format!(
            "  Skipped {} media files (unsafe name, too large or damaged).",
            s.media_skipped
        ));
    }
    if s.media_missing > 0 {
        lines.push(format!(
            "  {} files the cards refer to aren't in the package (Anki: Tools → Check Media).",
            s.media_missing
        ));
    }
    lines.push("Next: scrubjay review".into());
    lines.join("\n")
}

/// Map positional values and `--field` flags onto the card type's fields.
fn new_fact(args: AddCardArgs) -> Result<NewFact, CliError> {
    let card_type = CardType::parse(&args.card_type)?;
    let names = card_type.fields();
    let mut fields = BTreeMap::new();
    for (name, value) in names.iter().zip([args.first, args.second]) {
        if let Some(v) = value {
            fields.insert(name.to_string(), v);
        }
    }
    for pair in args.fields {
        let (name, value) = pair
            .split_once('=')
            .ok_or_else(|| CliError::Usage(format!("--field expects NAME=VALUE, got '{pair}'")))?;
        fields.insert(name.trim().to_string(), value.to_string());
    }
    Ok(NewFact {
        deck: args.deck,
        card_type,
        fields,
        allow_duplicate: args.allow_duplicate,
        tags: args.tags,
    })
}

fn mark(ok: bool) -> &'static str {
    match (ok, std::io::stdout().is_terminal()) {
        (true, true) => "✓",
        (false, true) => "✗",
        (true, false) => "ok",
        (false, false) => "FAIL",
    }
}

struct Output {
    json: bool,
}

impl Output {
    fn emit<T: Serialize>(&self, data: &T, human: impl FnOnce() -> String) -> Result<(), CliError> {
        if self.json {
            let env = Envelope {
                schema: SCHEMA,
                ok: true,
                data,
                warnings: vec![],
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&env).expect("serialisable")
            );
        } else {
            println!("{}", human());
        }
        Ok(())
    }
}
