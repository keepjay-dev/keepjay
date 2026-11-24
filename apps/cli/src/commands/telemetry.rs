// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay telemetry`: the opt-in weekly usage ping (D-040, NFR-PRIV-002, docs/TELEMETRY.md).
//!
//! The ping is off unless the user turns it on, here or when `keepjay init` asks. After any
//! command, [`spawn_if_due`] starts a detached `keepjay telemetry send --background` when a
//! ping is due, so no command waits on the network.

use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use clap::Subcommand;
use serde::Serialize;
use study_core::domain::now_ms;
use study_core::telemetry::{self, Ping, Settings};
use study_core::{Error, Paths};

use super::Ctx;
use crate::output::CliError;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DOCS: &str = "https://github.com/keepjay-dev/keepjay/blob/main/docs/TELEMETRY.md";

#[derive(Subcommand)]
pub enum TelemetryCmd {
    /// Show whether the usage ping is on, and exactly what it would send
    Status,
    /// Turn the anonymous weekly usage ping on
    On,
    /// Turn it off and forget this installation's random id
    Off,
    /// Send a ping now, even if one was sent this week
    Send {
        /// Run silently as the detached sender started after a command
        #[arg(long, hide = true)]
        background: bool,
    },
}

#[derive(Serialize)]
struct Status {
    enabled: bool,
    asked: bool,
    disabled_by: Option<&'static str>,
    endpoint: String,
    last_sent_week: Option<String>,
    payload: Option<Ping>,
}

pub fn run(ctx: &mut Ctx, cmd: TelemetryCmd) -> Result<(), CliError> {
    let data_dir = ctx.core.paths().data_dir.clone();
    match cmd {
        TelemetryCmd::Status => {
            let s = Settings::load(&data_dir);
            let status = Status {
                enabled: s.is_on(),
                asked: s.enabled.is_some(),
                disabled_by: telemetry::env_override(),
                endpoint: telemetry::endpoint(),
                last_sent_week: s.last_sent_week.clone(),
                payload: ctx.core.usage_ping(VERSION, now_ms())?,
            };
            ctx.out.emit(&status, || describe(&status))
        }
        TelemetryCmd::On => {
            let mut s = Settings::load(&data_dir);
            s.set_enabled(true);
            s.save(&data_dir)?;
            let note = match telemetry::env_override() {
                Some(var) => format!("\nNothing is sent while {var} is set."),
                None => String::new(),
            };
            ctx.out.emit(&serde_json::json!({ "enabled": true }), || {
                format!(
                    "Usage ping on: once a week, Keepjay sends its version, your OS, a rough review count and whether you use sources. Never card content, names or paths.\nSee exactly what's sent: `keepjay telemetry status`. Turn it off: `keepjay telemetry off`.{note}"
                )
            })
        }
        TelemetryCmd::Off => {
            let mut s = Settings::load(&data_dir);
            s.set_enabled(false);
            s.save(&data_dir)?;
            ctx.out.emit(&serde_json::json!({ "enabled": false }), || {
                "Usage ping off. This installation's random id is deleted.".to_string()
            })
        }
        TelemetryCmd::Send { background } => {
            let result = send(ctx, &data_dir, background);
            if background {
                return Ok(()); // best effort: a detached sender never reports
            }
            let week = result?;
            ctx.out
                .emit(&serde_json::json!({ "sent": true, "week": week }), || {
                    format!("Sent the usage ping for {week}.")
                })
        }
    }
}

fn describe(s: &Status) -> String {
    let state = match (s.enabled, s.asked) {
        (true, _) => "on",
        (false, true) => "off",
        (false, false) => "off (you haven't been asked yet)",
    };
    let mut out = format!("Usage ping: {state}");
    if let Some(var) = s.disabled_by {
        out.push_str(&format!("\nNothing is sent while {var} is set."));
    }
    match &s.payload {
        Some(p) => {
            let json = serde_json::to_string_pretty(p).unwrap_or_default();
            out.push_str(&format!(
                "\nLast sent: {}\nEndpoint: {}\nOnce a week it sends exactly this:\n{json}",
                s.last_sent_week.as_deref().unwrap_or("never"),
                s.endpoint
            ));
        }
        None => out.push_str("\nTurn it on with `keepjay telemetry on`."),
    }
    out.push_str(&format!("\nDetails: {DOCS}"));
    out
}

/// Send one ping; returns the week it counts for.
fn send(ctx: &mut Ctx, data_dir: &Path, background: bool) -> Result<String, CliError> {
    if let Some(var) = telemetry::env_override() {
        return Err(usage(format!("the usage ping is disabled by {var}")));
    }
    let Some(ping) = ctx.core.usage_ping(VERSION, now_ms())? else {
        return Err(usage(
            "the usage ping is off; turn it on with `keepjay telemetry on`".into(),
        ));
    };
    let timeout = Duration::from_secs(if background { 10 } else { 20 });
    let body = serde_json::to_string(&ping).expect("ping serialises");
    let status = post(&telemetry::endpoint(), &body, timeout)
        .map_err(|e| CliError::Core(Error::Unexpected(format!("couldn't send the ping: {e}"))))?;
    if !(200..300).contains(&status) {
        return Err(CliError::Core(Error::Unexpected(format!(
            "the ping endpoint answered HTTP {status}"
        ))));
    }
    let week = telemetry::iso_week(now_ms());
    // Re-read: the user may have turned it off while the ping was in flight.
    let mut s = Settings::load(data_dir);
    if s.is_on() {
        s.last_sent_week = Some(week.clone());
        s.save(data_dir)?;
    }
    Ok(week)
}

fn usage(msg: String) -> CliError {
    CliError::Core(Error::validation(msg))
}

fn post(url: &str, body: &str, timeout: Duration) -> Result<u16, ureq::Error> {
    use ureq::tls::{RootCerts, TlsConfig, TlsProvider};
    let tls = TlsConfig::builder()
        .provider(TlsProvider::Rustls)
        .root_certs(RootCerts::WebPki)
        .unversioned_rustls_crypto_provider(Arc::new(rustls::crypto::ring::default_provider()))
        .build();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .tls_config(tls)
        .timeout_global(Some(timeout))
        .http_status_as_error(false)
        .user_agent(format!("keepjay/{VERSION}"))
        .build()
        .into();
    let response = agent
        .post(url)
        .header("content-type", "application/json")
        .send(body)?;
    Ok(response.status().as_u16())
}

/// After a command: if a ping is due, start a detached sender and return at once.
pub fn spawn_if_due(paths: &Paths) {
    if telemetry::env_override().is_some() {
        return;
    }
    let mut s = Settings::load(&paths.data_dir);
    let now = now_ms();
    if !s.due(now) {
        return;
    }
    // Claim the attempt first, so other commands don't start a second sender.
    s.last_attempt_ms = Some(now);
    if s.save(&paths.data_dir).is_err() {
        return;
    }
    super::spawn_detached(paths, &["telemetry", "send", "--background"]);
}

/// After `keepjay init` or `keepjay source add` on a terminal: ask once, defaulting to no
/// (D-040). Never in scripts, with `--json`, or once answered.
pub fn ask_once(ctx: &Ctx, json: bool) -> Result<(), CliError> {
    let paths = ctx.core.paths();
    let interactive = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let mut s = Settings::load(&paths.data_dir);
    if json || !interactive || s.enabled.is_some() || telemetry::env_override().is_some() {
        return Ok(());
    }
    let mut out = std::io::stdout();
    write!(
        out,
        "\nHelp keep Keepjay going? Once a week it can send an anonymous ping: its version, \
         your OS, a rough review count and whether you use sources. Never card content, names \
         or paths. Details: {DOCS}\nSend the weekly ping? [y/N] "
    )?;
    out.flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    let yes = matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes");
    s.set_enabled(yes);
    s.save(&paths.data_dir)?;
    writeln!(
        out,
        "{}",
        if yes {
            "Thanks: the usage ping is on. `keepjay telemetry off` turns it off."
        } else {
            "The usage ping stays off. `keepjay telemetry on` turns it on."
        }
    )?;
    Ok(())
}
