// SPDX-License-Identifier: AGPL-3.0-or-later
//! `scrubjay init` and `scrubjay workspace list|create`.

use clap::Subcommand;

use super::{Ctx, DEFAULT_WORKSPACE};
use crate::output::CliError;

#[derive(Subcommand)]
pub enum WorkspaceCmd {
    /// List workspaces
    List,
    /// Create a workspace
    Create { name: String },
}

pub fn init(ctx: &mut Ctx) -> Result<(), CliError> {
    let ws = ctx.core.init(ctx.ws_name.unwrap_or(DEFAULT_WORKSPACE))?;
    ctx.out.emit(&ws, || {
        format!("Ready. Workspace '{}' ({})", ws.name, ws.short_id)
    })
}

pub fn run(ctx: &mut Ctx, cmd: WorkspaceCmd) -> Result<(), CliError> {
    match cmd {
        WorkspaceCmd::List => {
            let list = ctx.core.list_workspaces()?;
            ctx.out.emit(&list, || {
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
        WorkspaceCmd::Create { name } => {
            let ws = ctx.core.create_workspace(&name)?;
            ctx.out.emit(&ws, || {
                format!("Created workspace '{}' ({})", ws.name, ws.short_id)
            })
        }
    }
}
