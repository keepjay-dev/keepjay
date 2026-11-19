// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end tests of the `addisco` binary: JSON envelope and exit codes (06 §6).

use std::process::{Command, Output};

fn addisco(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_addisco"))
        .args(args)
        .env("ADDISCO_DATA_DIR", dir)
        .output()
        .expect("run addisco")
}

fn json(out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "invalid JSON ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

#[test]
fn init_then_status_reports_one_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let out = addisco(dir.path(), &["init", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let v = json(&out);
    assert_eq!(v["schema"], "addisco.cli/1");
    assert_eq!(v["data"]["name"], "My Studies");

    let v = json(&addisco(dir.path(), &["status", "--json"]));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["workspaces"], 1);
}

#[test]
fn bare_command_runs_status() {
    let dir = tempfile::tempdir().unwrap();
    let out = addisco(dir.path(), &[]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("addisco init"));
}

#[test]
fn duplicate_workspace_is_a_conflict_with_exit_code_5() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        addisco(dir.path(), &["workspace", "create", "Uni"])
            .status
            .success()
    );
    let out = addisco(dir.path(), &["workspace", "create", "Uni", "--json"]);
    assert_eq!(out.status.code(), Some(5));
    let v = json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "conflict");
}

#[test]
fn empty_name_is_a_validation_error_with_exit_code_4() {
    let dir = tempfile::tempdir().unwrap();
    let out = addisco(dir.path(), &["workspace", "create", " "]);
    assert_eq!(out.status.code(), Some(4));
}

#[test]
fn unknown_flag_is_a_usage_error_with_exit_code_2() {
    let dir = tempfile::tempdir().unwrap();
    let out = addisco(dir.path(), &["--definitely-not-a-flag"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn doctor_passes_on_fresh_database() {
    let dir = tempfile::tempdir().unwrap();
    let out = addisco(dir.path(), &["doctor", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(json(&out)["data"]["ok"], true);
}
