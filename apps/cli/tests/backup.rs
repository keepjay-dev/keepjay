// SPDX-License-Identifier: AGPL-3.0-or-later
//! `keepjay backup` end to end (IO-004): create, list, restore, and the daily automatic
//! backup started in the background after a command.

use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

fn keepjay_env(dir: &Path, args: &[&str], auto: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_keepjay"));
    cmd.args(args)
        .env("KEEPJAY_DATA_DIR", dir)
        .env("KEEPJAY_NO_TELEMETRY", "1");
    if auto {
        cmd.env_remove("KEEPJAY_NO_AUTO_BACKUP");
    } else {
        cmd.env("KEEPJAY_NO_AUTO_BACKUP", "1");
    }
    cmd.output().expect("run keepjay")
}

fn keepjay(dir: &Path, args: &[&str]) -> Output {
    keepjay_env(dir, args, false)
}

fn json(out: &Output) -> serde_json::Value {
    assert!(out.status.success(), "{out:?}");
    serde_json::from_slice(&out.stdout).expect("JSON")
}

fn with_card() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        &["init"][..],
        &["deck", "create", "Rust"],
        &["add", "card", "first", "back", "-d", "Rust"],
    ] {
        assert!(keepjay(dir.path(), args).status.success());
    }
    dir
}

fn cards(dir: &Path) -> usize {
    let v = json(&keepjay(dir, &["status", "--json"]));
    v["data"]["today"]["due"]["new"].as_u64().unwrap() as usize
}

#[test]
fn create_list_and_restore_round_trip() {
    let dir = with_card();
    let made = json(&keepjay(dir.path(), &["backup", "create", "--json"]));
    let id = made["data"]["id"].as_str().unwrap().to_string();
    assert_eq!(made["data"]["kind"], "manual");
    assert!(id.ends_with("-manual"), "{id}");

    assert!(
        keepjay(dir.path(), &["add", "card", "second", "back", "-d", "Rust"])
            .status
            .success()
    );
    assert_eq!(cards(dir.path()), 2);

    let listed = json(&keepjay(dir.path(), &["backup", "list", "--json"]));
    assert_eq!(listed["data"]["backups"][0]["id"], id.as_str());

    let restored = json(&keepjay(
        dir.path(),
        &["backup", "restore", &id, "--yes", "--json"],
    ));
    assert_eq!(restored["data"]["restored"]["id"], id.as_str());
    assert_eq!(restored["data"]["previous"]["kind"], "pre-restore");
    assert_eq!(cards(dir.path()), 1);

    // Text output names the undo command.
    let previous = restored["data"]["previous"]["id"].as_str().unwrap();
    let out = keepjay(dir.path(), &["backup", "restore", previous, "--yes"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("keepjay backup restore"));
    assert_eq!(cards(dir.path()), 2);
}

#[test]
fn restore_needs_yes_without_a_terminal_and_a_known_id() {
    let dir = with_card();
    let id = json(&keepjay(dir.path(), &["backup", "create", "--json"]))["data"]["id"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        keepjay(dir.path(), &["backup", "restore", &id])
            .status
            .code(),
        Some(2)
    );
    assert_eq!(
        keepjay(dir.path(), &["backup", "restore", "nope", "--yes"])
            .status
            .code(),
        Some(3)
    );
}

#[test]
fn doctor_reports_the_latest_backup() {
    let dir = with_card();
    let v = json(&keepjay(dir.path(), &["doctor", "--json"]));
    assert_eq!(v["data"]["backups"]["count"], 0);
    keepjay(dir.path(), &["backup", "create"]);
    let v = json(&keepjay(dir.path(), &["doctor", "--json"]));
    assert_eq!(v["data"]["backups"]["count"], 1);
    assert_eq!(v["data"]["backups"]["latest"]["kind"], "manual");
}

#[test]
fn a_command_starts_the_daily_backup_in_the_background() {
    let dir = with_card();
    let out = keepjay_env(dir.path(), &["status"], true);
    assert!(out.status.success(), "{out:?}");
    let deadline = Instant::now() + Duration::from_secs(20);
    let backups = dir.path().join("backups");
    loop {
        let autos = std::fs::read_dir(&backups)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter(|e| e.file_name().to_string_lossy().ends_with("-auto.db.zst"))
                    .count()
            })
            .unwrap_or(0);
        if autos == 1 {
            break;
        }
        assert!(Instant::now() < deadline, "no automatic backup appeared");
        std::thread::sleep(Duration::from_millis(100));
    }
    // Another command the same day doesn't start a second one.
    keepjay_env(dir.path(), &["status"], true);
    std::thread::sleep(Duration::from_millis(500));
    let v = json(&keepjay(dir.path(), &["backup", "list", "--json"]));
    assert_eq!(v["data"]["backups"].as_array().unwrap().len(), 1);
}
