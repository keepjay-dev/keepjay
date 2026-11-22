// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scrubjay desktop shell (Tauri 2). It's a thin adapter: every command delegates to
//! `study_core::Core` (D-002, docs/specs/08-architecture.md §6).
//!
//! `scrubjay-desktop --print-status` prints the core status as JSON and exits without
//! opening a window. CI uses it to prove the desktop app and the CLI share one database.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use serde::Serialize;
use study_core::{Core, Paths, Status, Workspace};

struct AppState(Mutex<Core>);

/// Errors cross the IPC boundary as `{ code, message }`.
#[derive(Debug, Serialize)]
struct CommandError {
    code: study_core::ErrorKind,
    message: String,
}

impl From<study_core::Error> for CommandError {
    fn from(e: study_core::Error) -> Self {
        CommandError {
            code: e.kind(),
            message: e.to_string(),
        }
    }
}

type CmdResult<T> = Result<T, CommandError>;

fn with_core<T>(
    state: &tauri::State<'_, AppState>,
    f: impl FnOnce(&mut Core) -> study_core::Result<T>,
) -> CmdResult<T> {
    let mut core = state.0.lock().map_err(|_| CommandError {
        code: study_core::ErrorKind::Unexpected,
        message: "core lock poisoned".into(),
    })?;
    Ok(f(&mut core)?)
}

#[tauri::command]
fn status(state: tauri::State<'_, AppState>) -> CmdResult<Status> {
    with_core(&state, |c| c.status())
}

#[tauri::command]
fn list_workspaces(state: tauri::State<'_, AppState>) -> CmdResult<Vec<Workspace>> {
    with_core(&state, |c| c.list_workspaces())
}

#[tauri::command]
fn create_workspace(state: tauri::State<'_, AppState>, name: String) -> CmdResult<Workspace> {
    with_core(&state, |c| c.create_workspace(&name))
}

/// Polled by the UI for live refresh (DESK-007): changes only when another process commits.
#[tauri::command]
fn data_version(state: tauri::State<'_, AppState>) -> CmdResult<i64> {
    with_core(&state, |c| c.data_version())
}

fn main() {
    let paths = Paths::resolve(None);
    let core = match Core::open(paths) {
        Ok(core) => core,
        Err(e) => {
            eprintln!("Scrubjay could not open its data: {e}");
            std::process::exit(1);
        }
    };

    if std::env::args().any(|a| a == "--print-status") {
        match core.status() {
            Ok(s) => println!(
                "{}",
                serde_json::to_string_pretty(&s).expect("serialisable")
            ),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
        return;
    }

    tauri::Builder::default()
        .manage(AppState(Mutex::new(core)))
        .invoke_handler(tauri::generate_handler![
            status,
            list_workspaces,
            create_workspace,
            data_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running Scrubjay");
}
