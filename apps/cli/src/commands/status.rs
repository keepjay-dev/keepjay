// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay status` (also the bare `scrubjay`): today's summary, or one line from a
//! template for prompts and status bars (CLI-021).

use clap::Args;
use serde::Serialize;
use study_core::ErrorKind;

use super::Ctx;
use super::review;
use crate::output::CliError;

#[derive(Args, Default)]
pub struct StatusArgs {
    /// One line from a template, for shell prompts and status bars (CLI-021).
    /// Fields: {due} {review} {learning} {new} {stale} {reviewed}; `{{` for a brace
    #[arg(long, value_name = "TEMPLATE")]
    format: Option<String>,
}

pub fn run(ctx: &mut Ctx, args: StatusArgs) -> Result<(), CliError> {
    let system = ctx.core.status()?;
    let today = match ctx.core.resolve_workspace(ctx.ws_name) {
        Ok(ws) => Some(ctx.core.today(ws, review::now_ms())?),
        Err(e) if e.kind() == ErrorKind::NotFound && ctx.ws_name.is_none() => None,
        Err(e) => return Err(e.into()),
    };
    if let Some(template) = args.format.filter(|_| !ctx.out.json) {
        // Nothing at all before `init`, so prompts stay clean.
        let line = match &today {
            Some(t) => status_line(&template, t)?,
            None => String::new(),
        };
        println!("{line}");
        return Ok(());
    }
    let view = StatusView { system, today };
    ctx.out.emit(&view, || view.human())
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
