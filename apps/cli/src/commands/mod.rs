// SPDX-License-Identifier: AGPL-3.0-or-later
//! One module per command group. Each owns its arguments, calls the core and words its
//! output; `main.rs` only parses and dispatches.

pub mod alias;
pub mod backup;
pub mod card_text;
pub mod cards;
pub mod doctor;
pub mod import;
pub mod review;
pub mod sources;
pub mod status;
pub mod telemetry;
pub mod while_cmd;
pub mod workspace;

use std::process::{Command, Stdio};

use study_core::{Core, ErrorKind, Id, Paths};

use crate::output::{CliError, Output};

/// Start `keepjay --data-dir D ARGS…` detached, with no input or output, for background work
/// after a command (the usage ping, the daily backup). Failures are ignored.
pub fn spawn_detached(paths: &Paths, args: &[&str]) {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let mut cmd = Command::new(exe);
    cmd.arg("--data-dir")
        .arg(&paths.data_dir)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let _ = cmd.spawn();
}

/// The workspace `init` creates when none is named.
pub const DEFAULT_WORKSPACE: &str = "My Studies";

/// What every command gets.
pub struct Ctx<'a> {
    pub core: &'a mut Core,
    pub out: &'a Output,
    /// `-w` / `KEEPJAY_WORKSPACE`, if given.
    pub ws_name: Option<&'a str>,
}

impl Ctx<'_> {
    /// The workspace to act on; an error if there's none yet.
    pub fn workspace(&self) -> Result<Id, CliError> {
        Ok(self.core.resolve_workspace(self.ws_name)?)
    }

    /// The workspace for a command that adds things (sources, decks, cards, imports). On a
    /// fresh install it creates the default workspace first, so `keepjay source add .`
    /// works straight after installing (J15) without a separate `init`.
    pub fn workspace_for_new(&mut self) -> Result<Id, CliError> {
        match self.core.resolve_workspace(self.ws_name) {
            Err(e)
                if e.kind() == ErrorKind::NotFound
                    && self.ws_name.is_none()
                    && self.core.list_workspaces()?.is_empty() =>
            {
                let ws = self.core.init(DEFAULT_WORKSPACE)?;
                eprintln!("Created your workspace '{}'.", ws.name);
                Ok(ws.id.parse()?)
            }
            other => Ok(other?),
        }
    }
}
