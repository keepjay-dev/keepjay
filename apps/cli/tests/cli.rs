// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end tests of the `scrubjay` binary: JSON envelope and exit codes (06 §6).

use std::process::{Command, Output};

fn scrubjay(dir: &std::path::Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_scrubjay"))
        .args(args)
        .env("SCRUBJAY_DATA_DIR", dir)
        .output()
        .expect("run scrubjay")
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
    let out = scrubjay(dir.path(), &["init", "--json"]);
    assert!(out.status.success(), "{out:?}");
    let v = json(&out);
    assert_eq!(v["schema"], "scrubjay.cli/1");
    assert_eq!(v["data"]["name"], "My Studies");

    let v = json(&scrubjay(dir.path(), &["status", "--json"]));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["workspaces"], 1);
}

#[test]
fn bare_command_runs_status() {
    let dir = tempfile::tempdir().unwrap();
    let out = scrubjay(dir.path(), &[]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("scrubjay init"));
}

#[test]
fn duplicate_workspace_is_a_conflict_with_exit_code_5() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        scrubjay(dir.path(), &["workspace", "create", "Uni"])
            .status
            .success()
    );
    let out = scrubjay(dir.path(), &["workspace", "create", "Uni", "--json"]);
    assert_eq!(out.status.code(), Some(5));
    let v = json(&out);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "conflict");
}

#[test]
fn empty_name_is_a_validation_error_with_exit_code_4() {
    let dir = tempfile::tempdir().unwrap();
    let out = scrubjay(dir.path(), &["workspace", "create", " "]);
    assert_eq!(out.status.code(), Some(4));
}

#[test]
fn unknown_flag_is_a_usage_error_with_exit_code_2() {
    let dir = tempfile::tempdir().unwrap();
    let out = scrubjay(dir.path(), &["--definitely-not-a-flag"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn doctor_passes_on_fresh_database() {
    let dir = tempfile::tempdir().unwrap();
    let out = scrubjay(dir.path(), &["doctor", "--json"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(json(&out)["data"]["ok"], true);
}

fn scrubjay_stdin(dir: &std::path::Path, args: &[&str], input: &str) -> Output {
    use std::io::Write;
    let mut child = Command::new(env!("CARGO_BIN_EXE_scrubjay"))
        .args(args)
        .env("SCRUBJAY_DATA_DIR", dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn scrubjay");
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
    assert!(scrubjay(dir.path(), &["init"]).status.success());
    assert!(
        scrubjay(dir.path(), &["deck", "create", "Bio::Cells"])
            .status
            .success()
    );
    dir
}

#[test]
fn decks_and_cards_round_trip_as_json() {
    let dir = with_deck();
    let v = json(&scrubjay(dir.path(), &["deck", "list", "--json"]));
    let paths: Vec<_> = v["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["path"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(paths, ["Bio", "Bio::Cells"]);

    let v = json(&scrubjay(
        dir.path(),
        &["add", "card", "Q", "A", "-d", "Bio::Cells", "--json"],
    ));
    assert_eq!(v["data"]["cards"].as_array().unwrap().len(), 1);
    let v = json(&scrubjay(
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
    let v = json(&scrubjay(
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
    let code = |args: &[&str]| scrubjay(dir.path(), args).status.code();
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
    scrubjay(
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
    let out = scrubjay_stdin(dir.path(), &["review", "--plain"], "\n3\n");
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("Q: What produces ATP?"), "{text}");
    assert!(text.contains("A: Mitochondria"), "{text}");
    assert!(text.contains("Reviewed 1 card"), "{text}");

    // Answered "Good": nothing is due now.
    let out = scrubjay_stdin(dir.path(), &["review", "--plain"], "");
    assert!(String::from_utf8_lossy(&out.stdout).contains("Nothing to review"));
}

#[test]
fn plain_review_reprompts_on_invalid_rating_and_supports_json() {
    let dir = with_deck();
    scrubjay(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let out = scrubjay_stdin(
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
    let out = scrubjay_stdin(dir.path(), &["review"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--plain"));
}

#[test]
fn status_shows_today_and_points_to_the_next_action() {
    let dir = with_deck();
    scrubjay(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let text = String::from_utf8_lossy(&scrubjay(dir.path(), &["status"]).stdout).to_string();
    assert!(text.contains("Today · My Studies"), "{text}");
    assert!(text.contains("1 new"), "{text}");
    assert!(text.contains("Next      scrubjay review"), "{text}");

    scrubjay_stdin(dir.path(), &["review", "--plain"], "\n3\n");
    let v = json(&scrubjay(dir.path(), &["status", "--json"]));
    // Existing fields stay at the top level (additive change, 06 §6.1).
    assert_eq!(v["data"]["workspaces"], 1);
    assert_eq!(v["data"]["today"]["reviewed"], 1);
    assert_eq!(v["data"]["today"]["due"]["new"], 0);
    let text = String::from_utf8_lossy(&scrubjay(dir.path(), &["status"]).stdout).to_string();
    assert!(text.contains("all caught up"), "{text}");
}

#[test]
fn tags_on_cards_and_tag_list() {
    let dir = with_deck();
    let out = scrubjay(
        dir.path(),
        &[
            "add",
            "card",
            "ATP?",
            "Mitochondria",
            "-d",
            "Bio::Cells",
            "--tag",
            "bio::cells",
            "--tag",
            "exam",
        ],
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("(tags: bio::cells, exam)"), "{text}");

    let v = json(&scrubjay(
        dir.path(),
        &[
            "add",
            "card",
            "DNA?",
            "Nucleus",
            "-d",
            "Bio::Cells",
            "--tag",
            "EXAM",
            "--json",
        ],
    ));
    assert_eq!(
        v["data"]["tags"],
        serde_json::json!(["exam"]),
        "existing spelling wins"
    );

    let v = json(&scrubjay(dir.path(), &["tag", "list", "--json"]));
    assert_eq!(
        v["data"],
        serde_json::json!([{"path": "bio::cells", "facts": 1}, {"path": "exam", "facts": 2}])
    );
    let text = String::from_utf8_lossy(&scrubjay(dir.path(), &["tag", "list"]).stdout).to_string();
    assert!(text.contains("exam        2 facts"), "{text}");

    let bad = scrubjay(
        dir.path(),
        &[
            "add",
            "card",
            "x",
            "y",
            "-d",
            "Bio::Cells",
            "--tag",
            "two words",
        ],
    );
    assert_eq!(
        bad.status.code(),
        Some(4),
        "invalid tag is a validation error"
    );
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
    scrubjay(dir.path(), &["init"]);
    let file = anki_fixture("sample-latest.apkg");
    let out = scrubjay(dir.path(), &["import", "anki", &file]);
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
    assert!(text.contains("Tags on 2 notes (3 new tags)."), "{text}");
    assert!(text.contains("Media: 1 file added."), "{text}");

    let v = json(&scrubjay(dir.path(), &["import", "anki", &file, "--json"]));
    assert_eq!(v["data"]["notes_already_imported"], 8);
    assert_eq!(v["data"]["media_already_present"], 1);

    let missing = scrubjay(dir.path(), &["import", "anki", "/nope.apkg"]);
    assert_eq!(missing.status.code(), Some(3));
}

#[test]
fn source_add_list_and_rm() {
    let dir = tempfile::tempdir().unwrap();
    assert!(scrubjay(dir.path(), &["init"]).status.success());
    let project = dir.path().join("my-app");
    std::fs::create_dir(&project).unwrap();
    let project = project.to_str().unwrap();

    let out = scrubjay(
        dir.path(),
        &["source", "add", project, "--tag", "lang::rust", "--json"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let added = json(&out);
    assert_eq!(added["data"]["name"], "my-app");
    assert_eq!(added["data"]["deck"], "my-app");
    assert_eq!(added["data"]["write_ids"], true);
    assert_eq!(added["data"]["tags"][0], "lang::rust");

    // Same folder again: conflict (exit 5).
    let again = scrubjay(dir.path(), &["source", "add", project, "--name", "x"]);
    assert_eq!(again.status.code(), Some(5));

    // Not a folder: validation (exit 4).
    let missing = dir.path().join("missing");
    let bad = scrubjay(dir.path(), &["source", "add", missing.to_str().unwrap()]);
    assert_eq!(bad.status.code(), Some(4));

    let list = json(&scrubjay(dir.path(), &["source", "list", "--json"]));
    assert_eq!(list["data"].as_array().unwrap().len(), 1);

    let rm = scrubjay(dir.path(), &["source", "rm", "my-app"]);
    assert!(rm.status.success());
    assert!(String::from_utf8_lossy(&rm.stdout).contains("kept"));
    let unknown = scrubjay(dir.path(), &["source", "rm", "my-app"]);
    assert_eq!(unknown.status.code(), Some(3));
    let list = json(&scrubjay(dir.path(), &["source", "list", "--json"]));
    assert_eq!(list["data"].as_array().unwrap().len(), 0);
}

#[test]
fn scan_writes_ids_and_stale_lists_changed_code() {
    let dir = tempfile::tempdir().unwrap();
    assert!(scrubjay(dir.path(), &["init"]).status.success());
    let project = dir.path().join("app");
    std::fs::create_dir(&project).unwrap();
    let file = project.join("lib.rs");
    std::fs::write(
        &file,
        "// @card What does f return?\n// One.\nfn f() -> u8 { 1 }\n",
    )
    .unwrap();
    let project = project.to_str().unwrap();
    assert!(
        scrubjay(dir.path(), &["source", "add", project])
            .status
            .success()
    );

    let scan = json(&scrubjay(dir.path(), &["scan", "--json"]));
    assert_eq!(scan["data"][0]["added"], 1);
    assert_eq!(scan["data"][0]["ids_written"], 1);
    let text = std::fs::read_to_string(&file).unwrap();
    let id = &text[text.find("@card(").unwrap() + 6..][..8];

    std::fs::write(&file, text.replace("{ 1 }", "{ 2 }")).unwrap();
    let human = scrubjay(dir.path(), &["scan", "app"]);
    assert!(String::from_utf8_lossy(&human.stdout).contains("now stale"));
    let stale = json(&scrubjay(dir.path(), &["stale", "--json"]));
    assert_eq!(stale["data"][0]["id"], id);

    assert!(
        scrubjay(dir.path(), &["stale", "keep", id])
            .status
            .success()
    );
    let stale = json(&scrubjay(dir.path(), &["stale", "list", "--json"]));
    assert_eq!(stale["data"].as_array().unwrap().len(), 0);
    assert_eq!(
        scrubjay(dir.path(), &["stale", "keep", id]).status.code(),
        Some(3)
    );

    // --quiet prints nothing on success.
    let quiet = scrubjay(dir.path(), &["scan", "--quiet"]);
    assert!(quiet.status.success() && quiet.stdout.is_empty());
}

#[test]
fn quick_review_is_silent_when_nothing_is_due() {
    let dir = with_deck();
    let out = scrubjay_stdin(dir.path(), &["review", "--quick"], "");
    assert!(out.status.success(), "{out:?}");
    assert!(out.stdout.is_empty(), "{out:?}");
    // -n and --time only make sense with --quick.
    let out = scrubjay(dir.path(), &["review", "-n", "3"]);
    assert_eq!(out.status.code(), Some(2));
    let out = scrubjay(dir.path(), &["review", "--quick", "--time", "soon"]);
    assert_eq!(out.status.code(), Some(2));
    // With cards due it still needs a terminal (or --plain).
    scrubjay(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let out = scrubjay_stdin(dir.path(), &["review", "--quick"], "");
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn plain_quick_review_stops_after_n_cards() {
    let dir = with_deck();
    for q in ["one", "two", "three"] {
        scrubjay(dir.path(), &["add", "card", q, "A", "-d", "Bio"]);
    }
    let out = scrubjay_stdin(
        dir.path(),
        &["review", "--quick", "-n", "2", "--plain", "--json"],
        "\n3\n\n3\n\n3\n",
    );
    assert!(out.status.success(), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout);
    let v: serde_json::Value = serde_json::from_str(&text[text.find("{\n").unwrap()..]).unwrap();
    assert_eq!(v["data"]["reviewed"], 2);
}

#[test]
fn status_format_fills_a_template() {
    let dir = tempfile::tempdir().unwrap();
    let out = scrubjay(dir.path(), &["status", "--format", "{due}d"]);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "\n",
        "empty before init"
    );

    let dir = with_deck();
    scrubjay(dir.path(), &["add", "card", "Q", "A", "-d", "Bio"]);
    let out = scrubjay(
        dir.path(),
        &["status", "--format", "{due}d {new}n {stale}s {{x}}"],
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "0d 1n 0s {x}\n");
    let out = scrubjay(dir.path(), &["status", "--format", "{target}"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("{stale}"));
}

#[cfg(unix)]
fn shell(script: &str) -> [&str; 3] {
    ["sh", "-c", script]
}

#[cfg(windows)]
fn shell(script: &str) -> [&str; 3] {
    ["cmd", "/C", script]
}

#[test]
fn while_passes_the_command_exit_code_through() {
    let dir = with_deck();
    let [a, b, c] = shell("echo hello && exit 3");
    let out = scrubjay(dir.path(), &["while", "--", a, b, c]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stdout).contains("hello"));

    let out = scrubjay(dir.path(), &["while", "--json", "--", a, b, c]);
    assert_eq!(out.status.code(), Some(3), "{out:?}");
    let v = json(&out);
    assert_eq!(v["data"]["exit_code"], 3);
    assert_eq!(
        v["data"]["output_tail"][0].as_str().unwrap().trim(),
        "hello"
    );
    assert_eq!(v["data"]["review"]["reviewed"], 0);

    // A command that can't start is a usage error; a missing `--` too.
    let out = scrubjay(dir.path(), &["while", "--", "scrubjay-no-such-command"]);
    assert_eq!(out.status.code(), Some(2));
    let out = scrubjay(dir.path(), &["while"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn adding_things_on_a_fresh_install_creates_the_workspace() {
    // J15: `scrubjay source add .` works straight after installing.
    let dir = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    let path = repo.path().to_str().unwrap();
    let out = scrubjay(dir.path(), &["source", "add", path, "--name", "app"]);
    assert!(out.status.success(), "{out:?}");
    assert!(String::from_utf8_lossy(&out.stderr).contains("Created your workspace 'My Studies'"));
    let out = scrubjay(dir.path(), &["deck", "create", "Rust"]);
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty(), "only once: {out:?}");
    let v = json(&scrubjay(dir.path(), &["workspace", "list", "--json"]));
    assert_eq!(v["data"].as_array().unwrap().len(), 1);
    // A named workspace that doesn't exist is still an error, not a new workspace.
    let out = scrubjay(dir.path(), &["deck", "create", "X", "-w", "Typo"]);
    assert_eq!(out.status.code(), Some(3));
    // Read-only commands don't create anything.
    let fresh = tempfile::tempdir().unwrap();
    assert_eq!(
        scrubjay(fresh.path(), &["deck", "list"]).status.code(),
        Some(3)
    );
}
