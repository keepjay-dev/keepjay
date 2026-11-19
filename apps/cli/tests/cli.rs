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

fn addisco_stdin(dir: &std::path::Path, args: &[&str], input: &str) -> Output {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_addisco"))
        .args(args)
        .env("ADDISCO_DATA_DIR", dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn addisco");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn with_deck() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    assert!(addisco(dir.path(), &["init"]).status.success());
    assert!(
        addisco(dir.path(), &["deck", "create", "Bio::Cells"])
            .status
            .success()
    );
    dir
}

#[test]
fn decks_and_cards_round_trip_as_json() {
    let dir = with_deck();
    let v = json(&addisco(dir.path(), &["deck", "list", "--json"]));
    let paths: Vec<_> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(paths, ["Bio", "Bio::Cells"]);

    let v = json(&addisco(
        dir.path(),
        &["add", "card", "Q", "A", "-d", "Bio::Cells", "--json"],
    ));
    assert_eq!(v["data"]["cards"].as_array().unwrap().len(), 1);
    let v = json(&addisco(
        dir.path(),
        &[
            "add",
            "card",
            "{{c1::a}} {{c2::b}}",
            "-d",
            "Bio",
            "-t",
            "cloze",
            "--json",
        ],
    ));
    assert_eq!(v["data"]["cards"].as_array().unwrap().len(), 2);
    let v = json(&addisco(
        dir.path(),
        &[
            "add",
            "card",
            "Q2",
            "A2",
            "-d",
            "Bio",
            "-t",
            "optional-reverse",
            "--field",
            "Add Reverse=y",
            "--json",
        ],
    ));
    assert_eq!(v["data"]["cards"].as_array().unwrap().len(), 2);
}

#[test]
fn add_card_errors_map_to_exit_codes() {
    let dir = with_deck();
    let code = |args: &[&str]| addisco(dir.path(), args).status.code();
    assert_eq!(code(&["add", "card", "Q", "-d", "Nope"]), Some(3));
    assert_eq!(
        code(&["add", "card", "Q", "-d", "Bio", "-t", "nope"]),
        Some(4)
    );
    assert_eq!(
        code(&["add", "card", "Q", "-d", "Bio", "--field", "oops"]),
        Some(2)
    );
    assert_eq!(code(&["add", "card", "Q", "-d", "Bio"]), Some(0));
    assert_eq!(
        code(&["add", "card", "Q", "-d", "Bio"]),
        Some(5),
        "duplicate"
    );
    assert_eq!(
        code(&["add", "card", "Q", "-d", "Bio", "--allow-duplicate"]),
        Some(0)
    );
    assert_eq!(code(&["deck", "list", "-w", "Nope"]), Some(3));
}

#[test]
fn plain_review_answers_from_stdin() {
    let dir = with_deck();
    addisco(
        dir.path(),
        &[
            "add",
            "card",
            "What produces ATP?",
            "Mitochondria",
            "-d",
            "Bio",
        ],
    );
    let out = addisco_stdin(dir.path(), &["review", "--plain"], "\n3\n");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("Q: What produces ATP?"), "{text}");
    assert!(text.contains("A: Mitochondria"), "{text}");
    assert!(text.contains("Reviewed 1 card"), "{text}");

    // Answered "Good": nothing is due now.
    let out = addisco_stdin(dir.path(), &["review", "--plain"], "");
    assert!(String::from_utf8_lossy(&out.stdout).contains("Nothing to review"));
}

#[test]
fn plain_review_reprompts_on_invalid_rating_and_supports_json() {
    let dir = with_deck();
    addisco(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let out = addisco_stdin(
        dir.path(),
        &["review", "--plain", "--json"],
        "\nmeh\ngood\n",
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Type 1–4"), "{text}");
    let summary = &text[text.find("{\n").unwrap()..];
    let v: serde_json::Value = serde_json::from_str(summary).unwrap();
    assert_eq!(v["data"]["reviewed"], 1);
    assert_eq!(v["data"]["good"], 1);
}

#[test]
fn interactive_review_without_a_terminal_is_a_usage_error() {
    let dir = with_deck();
    let out = addisco_stdin(dir.path(), &["review"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--plain"));
}

#[test]
fn status_shows_today_and_points_to_the_next_action() {
    let dir = with_deck();
    addisco(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let text = String::from_utf8_lossy(&addisco(dir.path(), &["status"]).stdout).to_string();
    assert!(text.contains("Today · My Studies"), "{text}");
    assert!(text.contains("1 new"), "{text}");
    assert!(text.contains("Next      addisco review"), "{text}");

    addisco_stdin(dir.path(), &["review", "--plain"], "\n3\n");
    let v = json(&addisco(dir.path(), &["status", "--json"]));
    // Existing fields stay at the top level (additive change, 06 §6.1).
    assert_eq!(v["data"]["workspaces"], 1);
    assert_eq!(v["data"]["today"]["reviewed"], 1);
    assert_eq!(v["data"]["today"]["due"]["new"], 0);
    let text = String::from_utf8_lossy(&addisco(dir.path(), &["status"]).stdout).to_string();
    assert!(text.contains("all caught up"), "{text}");
}

fn anki_fixture(name: &str) -> String {
    format!(
        "{}/../../crates/study-core/tests/fixtures/anki/{name}",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn import_anki_reports_and_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    addisco(dir.path(), &["init"]);
    let file = anki_fixture("sample-latest.apkg");
    let out = addisco(dir.path(), &["import", "anki", &file]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(
        text.contains("Imported 8 notes → 11 cards with 5 past reviews"),
        "{text}"
    );
    assert!(
        text.contains("Vocab (custom) (2 notes) → basic (rendered"),
        "{text}"
    );

    let v = json(&addisco(dir.path(), &["import", "anki", &file, "--json"]));
    assert_eq!(v["data"]["notes_already_imported"], 8);

    let missing = addisco(dir.path(), &["import", "anki", "/nope.apkg"]);
    assert_eq!(missing.status.code(), Some(3));
}
