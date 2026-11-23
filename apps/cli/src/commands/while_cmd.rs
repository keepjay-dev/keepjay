// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay while -- <command…>`: review while a command runs (CLI-020; 06 §9 "While").
//!
//! The command's output goes to a temporary file so it doesn't scribble over the review
//! screen. When the command exits, the screen closes, the end of its output is printed,
//! and `scrubjay` exits with the command's exit code, so it composes in scripts
//! (`sj while -- make && deploy`). Quitting the review (`q`) keeps the command running in
//! the foreground and streams its output; Ctrl+C stops both.
//! Without a terminal, the command just runs with its usual output.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use clap::Args;
use serde::Serialize;
use study_core::{Core, Id};

use super::Ctx;
use super::review::{self, End, Session, Summary, Watch};
use crate::output::CliError;

/// Lines of the command's output shown when it finishes.
const TAIL_LINES: usize = 20;

#[derive(Args)]
pub struct WhileArgs {
    /// Only this deck and its subdecks
    #[arg(short, long)]
    deck: Option<String>,
    /// Stop reviewing after this many cards (the command keeps running)
    #[arg(long)]
    limit: Option<u32>,
    /// The command to run, after `--`
    #[arg(last = true, required = true, value_name = "COMMAND")]
    command: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WhileReport {
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    /// The last lines the command printed (stdout and stderr together).
    pub output_tail: Vec<String>,
    pub review: Summary,
}

/// `scrubjay while`: returns the exit code `scrubjay` should exit with (the command's).
pub fn command(ctx: &mut Ctx, args: WhileArgs) -> Result<u8, CliError> {
    let ws = ctx.workspace()?;
    run(ctx.core, ws, &args, ctx.out.json)
}

/// Runs the command; returns the exit code `scrubjay` should exit with.
pub fn run(core: &mut Core, ws: Id, args: &WhileArgs, json: bool) -> Result<u8, CliError> {
    let label = shell_words(&args.command);
    let interactive = review::interactive();
    let started = Instant::now();
    if !interactive && !json {
        // Nothing to draw on: behave like the bare command.
        let status = child_command(&args.command)
            .status()
            .map_err(|e| spawn_error(&label, e))?;
        return Ok(clamp(exit_code(status)));
    }

    let log = tempfile::Builder::new()
        .prefix("scrubjay-while-")
        .suffix(".log")
        .tempfile()?;
    let mut child = child_command(&args.command)
        .stdin(Stdio::null())
        .stdout(log.as_file().try_clone()?)
        .stderr(log.as_file().try_clone()?)
        .spawn()
        .map_err(|e| spawn_error(&label, e))?;
    // @card(jaa1tdnb) Why does `scrubjay while` read the log through `reopen()`, not a cloned handle?
    // A clone shares its file offset with the command's stdout, so seeking it to read
    // would move where the command writes.
    let mut reader = log.reopen()?;

    let (summary, end) = if interactive {
        let mut session = Session {
            deck: args.deck.clone(),
            limit: args.limit,
            deadline: None,
            wait_learning: true,
            watch: Some(Watch {
                label: label.clone(),
                child: &mut child,
                started,
            }),
        };
        review::tui(core, ws, &mut session)?
    } else {
        (Summary::default(), End::Done)
    };

    let status = match end {
        End::Interrupted => {
            let _ = child.kill();
            child.wait()?
        }
        End::Quit | End::Done if interactive && !json => {
            eprintln!(
                "{} Waiting for `{label}`…",
                if summary.reviewed > 0 {
                    summary.to_string()
                } else {
                    "Stopped reviewing.".to_string()
                }
            );
            follow(&mut reader, &mut child)?
        }
        _ => child.wait()?,
    };

    let code = match end {
        End::Interrupted => 130,
        _ => exit_code(status),
    };
    let followed = matches!(end, End::Quit | End::Done) && interactive && !json;
    let report = WhileReport {
        command: label.clone(),
        exit_code: code,
        duration_ms: started.elapsed().as_millis() as u64,
        output_tail: if followed {
            Vec::new() // already streamed
        } else {
            tail(&mut reader, TAIL_LINES)?
        },
        review: summary,
    };
    if json {
        crate::output::Output { json: true }.emit(&report, String::new)?;
    } else {
        let mut out = std::io::stdout().lock();
        for line in &report.output_tail {
            writeln!(out, "{line}")?;
        }
        let reviewed = match report.review.reviewed {
            0 => String::new(),
            1 => " · reviewed 1 card".to_string(),
            n => format!(" · reviewed {n} cards"),
        };
        eprintln!(
            "`{label}` {} after {}{reviewed}.",
            match end {
                End::Interrupted => "was stopped".to_string(),
                _ => format!("exited with code {code}"),
            },
            review::fmt_clock(started.elapsed())
        );
    }
    Ok(clamp(code))
}

fn child_command(argv: &[String]) -> Command {
    let mut c = Command::new(&argv[0]);
    c.args(&argv[1..]);
    c
}

fn spawn_error(label: &str, e: std::io::Error) -> CliError {
    CliError::Usage(format!("couldn't run `{label}`: {e}"))
}

/// The command as a user would type it (for messages, not for running).
fn shell_words(argv: &[String]) -> String {
    argv.iter()
        .map(|a| {
            if a.is_empty() || a.contains(char::is_whitespace) {
                format!("'{a}'")
            } else {
                a.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Exit code as a shell reports it: 128 + signal when killed by one.
fn exit_code(status: ExitStatus) -> i32 {
    if let Some(code) = status.code() {
        return code;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return 128 + sig;
        }
    }
    1
}

/// Process exit codes are a byte on Unix; Windows codes above 255 become 1.
fn clamp(code: i32) -> u8 {
    u8::try_from(code).unwrap_or(1)
}

/// The last `n` lines of the log (reads at most the final 64 KiB).
fn tail(file: &mut File, n: usize) -> std::io::Result<Vec<String>> {
    const WINDOW: u64 = 64 * 1024;
    let len = file.seek(SeekFrom::End(0))?;
    let start = len.saturating_sub(WINDOW);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<&str> = text.lines().collect();
    if start > 0 && !lines.is_empty() {
        lines.remove(0); // probably a partial line
    }
    let skip = lines.len().saturating_sub(n);
    Ok(lines[skip..].iter().map(|l| l.to_string()).collect())
}

/// Print the end of the output so far, then stream new output until the command exits.
fn follow(file: &mut File, child: &mut std::process::Child) -> std::io::Result<ExitStatus> {
    let mut out = std::io::stdout().lock();
    for line in tail(file, TAIL_LINES)? {
        writeln!(out, "{line}")?;
    }
    out.flush()?;
    let mut buf = [0u8; 8192];
    loop {
        let exited = child.try_wait()?;
        loop {
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            out.write_all(&buf[..n])?;
        }
        out.flush()?;
        if let Some(status) = exited {
            return Ok(status);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn tail_keeps_the_last_lines() {
        let mut f = tempfile::tempfile().unwrap();
        for i in 1..=30 {
            writeln!(f, "line {i}").unwrap();
        }
        let t = tail(&mut f, 3).unwrap();
        assert_eq!(t, ["line 28", "line 29", "line 30"]);
        let mut empty = tempfile::tempfile().unwrap();
        assert!(tail(&mut empty, 3).unwrap().is_empty());
    }

    #[test]
    fn labels_quote_arguments_with_spaces() {
        let argv: Vec<String> = ["sh", "-c", "exit 3"].map(String::from).into();
        assert_eq!(shell_words(&argv), "sh -c 'exit 3'");
    }
}
