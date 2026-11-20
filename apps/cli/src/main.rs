// SPDX-License-Identifier: AGPL-3.0-or-later
//! `addisco` — the Addisco command-line tool (docs/specs/06-cli.md).
//!
//! Commands so far: `status`, `init`, `workspace`, `deck`, `add card`, `review`, `doctor`.
//! Every command follows the JSON envelope (06 §6.1) and exit codes (06 §6.3).

mod review;

use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use study_core::{CardType, Core, Error, NewFact, Paths};

const SCHEMA: &str = "addisco.cli/1";

#[derive(Parser)]
#[command(name = "addisco", version, about = "Addisco: study, your way.", long_about = None)]
struct Cli {
    /// Output JSON (shorthand for --format json)
    #[arg(long, global = true)]
    json: bool,

    /// Override the data directory
    #[arg(long, global = true, value_name = "PATH", env = "ADDISCO_DATA_DIR")]
    data_dir: Option<PathBuf>,

    /// Workspace to act on (default: the first one). For `init`, the name to create.
    #[arg(
        short = 'w',
        long,
        global = true,
        value_name = "NAME",
        env = "ADDISCO_WORKSPACE"
    )]
    workspace: Option<String>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show today's summary
    Status,
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
    /// Import from other apps
    #[command(subcommand)]
    Import(ImportCmd),
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
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::BrokenPipe) => ExitCode::SUCCESS,
        Err(err) => {
            let (code, exit) = err.codes();
            eprintln!("addisco: {}", err.message());
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

fn run(cli: Cli) -> Result<(), CliError> {
    let paths = Paths::resolve(cli.data_dir.as_deref());
    let mut core = Core::open(paths)?;
    let out = Output { json: cli.json };
    let ws_name = cli.workspace.as_deref();
    match cli.command.unwrap_or(Command::Status) {
        Command::Status => {
            let system = core.status()?;
            let today = match core.resolve_workspace(ws_name) {
                Ok(ws) => Some(core.today(ws, review::now_ms())?),
                Err(e) if e.kind() == study_core::ErrorKind::NotFound && ws_name.is_none() => None,
                Err(e) => return Err(e.into()),
            };
            let view = StatusView { system, today };
            out.emit(&view, || view.human())
        }
        Command::Init => {
            let ws = core.init(ws_name.unwrap_or("My Studies"))?;
            out.emit(&ws, || {
                format!("Ready. Workspace '{}' ({})", ws.name, ws.short_id)
            })
        }
        Command::Workspace(WorkspaceCmd::List) => {
            let list = core.list_workspaces()?;
            out.emit(&list, || {
                if list.is_empty() {
                    "No workspaces yet. Run `addisco init`.".to_string()
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
                    return "No decks yet. Create one with `addisco deck create <name>`."
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
            let ws = core.resolve_workspace(ws_name)?;
            let deck = core.create_deck(ws, &path)?;
            out.emit(&deck, || {
                format!("Created deck '{}' ({})", deck.path, deck.short_id)
            })
        }
        Command::Add(AddCmd::Card(args)) => {
            let ws = core.resolve_workspace(ws_name)?;
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
                    return "No tags yet. Add one with `addisco add card … --tag <tag>`."
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
            out.emit(&summary, || summary.to_string())
        }
        Command::Import(ImportCmd::Anki { file, into }) => {
            let ws = core.resolve_workspace(ws_name)?;
            let options = study_core::AnkiImportOptions { deck_prefix: into };
            let s = core.import_anki(ws, &file, &options)?;
            out.emit(&s, || import_report(&s))
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
}

/// `addisco status`: system info (unchanged fields) plus today's summary (additive).
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
                "Addisco {}\n  Next      run `addisco init` to create your first workspace\n{footer}",
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
            "addisco review".to_string()
        } else if let Some(at) = t.next_learning_due {
            let wait = (at - now).max(0) as f64 / 86_400_000.0;
            format!("next learning card in {}", review::fmt_interval(wait))
        } else {
            "add cards with `addisco add card`, or come back tomorrow".to_string()
        };
        format!(
            "Today · {}{:>pad$}\n  Cards     {cards}\n  Done      {done}\n  Next      {next}\n{footer}",
            t.workspace,
            t.label,
            pad = 44usize.saturating_sub(t.workspace.len())
        )
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
    lines.push("Next: addisco review".into());
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
