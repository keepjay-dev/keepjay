// SPDX-License-Identifier: AGPL-3.0-or-later
//! The JSON envelope (06 §6.1), exit codes (06 §6.3) and small text helpers every command
//! shares.

use std::io::IsTerminal;

use serde::Serialize;
use study_core::Error;

pub const SCHEMA: &str = "keepjay.cli/1";

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
    pub fn codes(&self) -> (String, u8) {
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

    pub fn message(&self) -> String {
        match self {
            CliError::Core(e) => e.to_string(),
            CliError::Usage(m) => m.clone(),
            CliError::BrokenPipe => String::new(),
        }
    }

    /// Print the error to stderr, and as a JSON envelope on stdout with `--json`. Returns
    /// the exit code.
    pub fn report(&self, json: bool) -> u8 {
        let (code, exit) = self.codes();
        eprintln!("keepjay: {}", self.message());
        if json {
            let body = ErrorEnvelope {
                schema: SCHEMA,
                ok: false,
                error: ErrorBody {
                    code,
                    message: self.message(),
                },
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&body).expect("serialisable")
            );
        }
        exit
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

/// Where a command's result goes: a JSON envelope with `--json`, else human text.
pub struct Output {
    pub json: bool,
}

impl Output {
    pub fn emit<T: Serialize>(
        &self,
        data: &T,
        human: impl FnOnce() -> String,
    ) -> Result<(), CliError> {
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

/// A pass/fail mark: symbols on a terminal, words otherwise.
pub fn mark(ok: bool) -> &'static str {
    match (ok, std::io::stdout().is_terminal()) {
        (true, true) => "✓",
        (false, true) => "✗",
        (true, false) => "ok",
        (false, false) => "FAIL",
    }
}

/// A warning mark, for checks that don't fail a command.
pub fn warn_mark() -> &'static str {
    if std::io::stdout().is_terminal() {
        "!"
    } else {
        "WARN"
    }
}

/// The first line of `s`, cut to `max` characters with an ellipsis.
pub fn truncate(s: &str, max: usize) -> String {
    let one_line = s.lines().next().unwrap_or("");
    if one_line.chars().count() <= max && !s.contains('\n') {
        one_line.to_string()
    } else {
        let cut: String = one_line.chars().take(max.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}
