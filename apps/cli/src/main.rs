// SPDX-License-Identifier: AGPL-3.0-or-later
//! `addisco` — the Addisco command-line tool (docs/specs/06-cli.md).
//!
//! M0.3 scaffold: `status`, `init`, `workspace list|create`, `doctor`, with the JSON
//! envelope (06 §6.1) and exit codes (06 §6.3) that every later command will follow.

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde::Serialize;
use study_core::{Core, Error, ErrorKind, Paths};

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

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show today's summary (scaffold: data location and counts)
    Status,
    /// First-run setup: create a workspace if none exists
    Init {
        /// Name of the first workspace
        #[arg(long, default_value = "My Studies")]
        workspace: String,
    },
    /// Manage workspaces
    #[command(subcommand)]
    Workspace(WorkspaceCmd),
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

#[derive(Serialize)]
struct Envelope<T: Serialize> {
    schema: &'static str,
    ok: bool,
    data: T,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct ErrorBody {
    code: ErrorKind,
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
        Err(err) => {
            eprintln!("addisco: {err}");
            if json {
                let body = ErrorEnvelope {
                    schema: SCHEMA,
                    ok: false,
                    error: ErrorBody {
                        code: err.kind(),
                        message: err.to_string(),
                    },
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&body).expect("serialisable")
                );
            }
            ExitCode::from(exit_code(err.kind()))
        }
    }
}

/// 06 §6.3. Usage errors (2) and Ctrl-C (130) are produced by clap and the OS.
fn exit_code(kind: ErrorKind) -> u8 {
    match kind {
        ErrorKind::Unexpected => 1,
        ErrorKind::NotFound => 3,
        ErrorKind::Validation => 4,
        ErrorKind::Conflict => 5,
        ErrorKind::VersionIncompatible => 6,
    }
}

fn run(cli: Cli) -> Result<(), Error> {
    let paths = Paths::resolve(cli.data_dir.as_deref());
    let mut core = Core::open(paths)?;
    let out = Output { json: cli.json };
    match cli.command.unwrap_or(Command::Status) {
        Command::Status => {
            let s = core.status()?;
            out.emit(&s, || {
                let hint = if s.workspaces == 0 {
                    "\n  Next      run `addisco init` to create your first workspace"
                } else {
                    ""
                };
                format!(
                    "Addisco {}\n  Data      {}\n  Schema    v{}\n  Workspaces {}{hint}",
                    s.core_version, s.data_dir, s.schema_version, s.workspaces
                )
            })
        }
        Command::Init { workspace } => {
            let ws = core.init(&workspace)?;
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
                Err(Error::Unexpected("database health check failed".into()))
            }
        }
    }
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
    fn emit<T: Serialize>(&self, data: &T, human: impl FnOnce() -> String) -> Result<(), Error> {
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
