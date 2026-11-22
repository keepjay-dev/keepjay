// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scanning sources end to end (SRC-004, SRC-005, SRC-006).

use std::path::Path;

use study_core::{Core, Id, NewSource, Paths, Rating};

struct Env {
    _dir: tempfile::TempDir,
    root: std::path::PathBuf,
    core: Core,
    ws: Id,
}

fn setup(write_ids: bool) -> Env {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("app");
    std::fs::create_dir_all(root.join("src")).unwrap();
    let mut core = Core::open(Paths::new(dir.path().join("data"), dir.path())).unwrap();
    core.init("Dev").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    core.add_source(
        ws,
        NewSource {
            path: root.clone(),
            tags: vec!["repo::app".into()],
            write_ids,
            ..NewSource::default()
        },
    )
    .unwrap();
    Env {
        _dir: dir,
        root,
        core,
        ws,
    }
}

fn write(root: &Path, rel: &str, text: &str) {
    std::fs::write(root.join(rel), text).unwrap();
}

fn read(root: &Path, rel: &str) -> String {
    std::fs::read_to_string(root.join(rel)).unwrap()
}

const LIB: &str = "\
// @card What does open() do first?
// Creates the data directory.
fn open() {
    create_dir();
}
";

/// The id written into the first `@card` line of a file.
fn id_in(text: &str) -> String {
    let at = text.find("@card(").expect("an id was written") + 6;
    text[at..at + 8].to_string()
}

#[test]
fn new_cards_get_ids_written_and_rescans_are_idempotent() {
    let mut e = setup(true);
    write(&e.root, "src/lib.rs", LIB);
    write(
        &e.root,
        "notes.cards.md",
        "Q: What is FSRS?\nA: A spaced-repetition scheduler.\n",
    );

    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.cards_found, r.added, r.ids_written), (2, 2, 2), "{r:?}");
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
    let lib = read(&e.root, "src/lib.rs");
    let id = id_in(&lib);
    assert!(lib.starts_with(&format!("// @card({id}) What does open() do first?")));
    assert!(read(&e.root, "notes.cards.md").contains("<!-- id: "));

    // The card is reviewable, in the source's deck, with the source's tag.
    let decks = e.core.list_decks(e.ws).unwrap();
    let deck = decks.iter().find(|d| d.path == "app").unwrap();
    assert_eq!(deck.cards, 2);
    assert!(
        e.core
            .list_tags(e.ws)
            .unwrap()
            .iter()
            .any(|t| t.path == "repo::app")
    );

    let again = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!(
        (
            again.added,
            again.updated,
            again.unchanged,
            again.ids_written,
            again.retired
        ),
        (0, 0, 2, 0, 0),
        "{again:?}"
    );
}

#[test]
fn changed_code_makes_a_card_stale_and_keep_clears_it() {
    let mut e = setup(true);
    write(&e.root, "src/lib.rs", LIB);
    e.core.scan_source(e.ws, "app").unwrap();
    let id = id_in(&read(&e.root, "src/lib.rs"));

    // Whitespace-only changes don't count.
    let lib = read(&e.root, "src/lib.rs").replace("create_dir();", "create_dir();   ");
    write(&e.root, "src/lib.rs", &lib);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!(r.became_stale, 0);

    // Real code changes do.
    let lib = lib.replace("create_dir();   ", "load_device_id();\n    create_dir();");
    write(&e.root, "src/lib.rs", &lib);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.became_stale, r.stale), (1, 1), "{r:?}");
    let stale = e.core.list_stale(e.ws, None).unwrap();
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].id, id);
    assert_eq!(stale[0].path, "src/lib.rs");
    assert_eq!(stale[0].question, "What does open() do first?");
    let now = study_core::domain::now_ms();
    assert_eq!(
        e.core.today(e.ws, now).unwrap().stale,
        1,
        "status counts it"
    );

    // Still stale after another scan; cleared by `keep`.
    assert_eq!(e.core.scan_source(e.ws, "app").unwrap().stale, 1);
    e.core.keep_stale(e.ws, &id).unwrap();
    assert!(e.core.list_stale(e.ws, None).unwrap().is_empty());
    assert_eq!(e.core.scan_source(e.ws, "app").unwrap().stale, 0);

    // Editing the card text also confirms it.
    let lib = read(&e.root, "src/lib.rs").replace("create_dir();", "create_dirs();");
    write(&e.root, "src/lib.rs", &lib);
    assert_eq!(e.core.scan_source(e.ws, "app").unwrap().stale, 1);
    let lib = lib.replace(
        "Creates the data directory.",
        "Loads the device id, then creates the data dir.",
    );
    write(&e.root, "src/lib.rs", &lib);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.updated, r.stale), (1, 0), "{r:?}");
}

#[test]
fn edits_and_moves_keep_review_history_and_removed_cards_retire_and_return() {
    let mut e = setup(true);
    write(&e.root, "src/lib.rs", LIB);
    e.core.scan_source(e.ws, "app").unwrap();
    let card = e
        .core
        .next_card(e.ws, None, study_core::domain::now_ms())
        .unwrap()
        .unwrap();
    e.core
        .answer_card(
            card.card.id.parse().unwrap(),
            Rating::Good,
            4000,
            study_core::domain::now_ms(),
        )
        .unwrap();
    let lib = read(&e.root, "src/lib.rs");

    // Move the card to another file and reword it: same fact, history kept.
    let moved = lib.replace(
        "What does open() do first?",
        "What does open() do before anything else?",
    );
    write(&e.root, "src/db.rs", &moved);
    std::fs::remove_file(e.root.join("src/lib.rs")).unwrap();
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.added, r.updated, r.retired), (0, 1, 0), "{r:?}");
    let after = e.core.card(card.card.id.parse().unwrap()).unwrap();
    assert_eq!(after.reps, 1, "review history survived the edit and move");

    // Delete it: retired. Put it back: restored with its history.
    std::fs::remove_file(e.root.join("src/db.rs")).unwrap();
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!(r.retired, 1);
    assert!(
        e.core
            .next_card(e.ws, None, study_core::domain::now_ms() + 365 * 86_400_000)
            .unwrap()
            .is_none(),
        "retired cards aren't reviewed"
    );
    write(&e.root, "src/db.rs", &moved);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.restored, r.added), (1, 0), "{r:?}");
    assert_eq!(e.core.card(card.card.id.parse().unwrap()).unwrap().reps, 1);
}

#[test]
fn ids_from_another_machine_are_kept_and_duplicates_get_new_ids() {
    let mut e = setup(true);
    // A teammate's clone: ids already in the file, unknown to this database.
    let text = "\
# @card(k22k1m8e) First?
# One.
x = 1

# @card(k22k1m8e) Copy-pasted second?
# Two.
y = 2
";
    write(&e.root, "src/a.py", text);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!(r.added, 2, "{r:?}");
    assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
    let out = read(&e.root, "src/a.py");
    assert!(
        out.starts_with("# @card(k22k1m8e) First?"),
        "first keeps its id:\n{out}"
    );
    let second = &out[out.find("Copy-pasted").unwrap() - 15..];
    assert!(
        !second.contains("k22k1m8e"),
        "the copy got a new id:\n{out}"
    );
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.added, r.unchanged, r.warnings.len()), (0, 2, 0), "{r:?}");
}

#[test]
fn sources_without_id_writing_match_cards_by_text() {
    let mut e = setup(false);
    write(&e.root, "src/lib.rs", LIB);
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.added, r.ids_written), (1, 0));
    assert_eq!(read(&e.root, "src/lib.rs"), LIB, "file untouched");
    let r = e.core.scan_source(e.ws, "app").unwrap();
    assert_eq!((r.added, r.unchanged), (0, 1), "{r:?}");
}

#[test]
fn cloze_numbers_added_and_removed_change_the_cards() {
    let mut e = setup(true);
    write(
        &e.root,
        "src/a.sql",
        "-- @card {{c1::VACUUM}} reclaims space\nVACUUM;\n",
    );
    e.core.scan_source(e.ws, "app").unwrap();
    let text = read(&e.root, "src/a.sql").replace(
        "reclaims space",
        "reclaims space; {{c2::ANALYZE}} updates stats",
    );
    write(&e.root, "src/a.sql", &text);
    e.core.scan_source(e.ws, "app").unwrap();
    let deck = e
        .core
        .list_decks(e.ws)
        .unwrap()
        .into_iter()
        .find(|d| d.path == "app")
        .unwrap();
    assert_eq!(deck.cards, 2);
    let text = text.replace("; {{c2::ANALYZE}} updates stats", "");
    write(&e.root, "src/a.sql", &text);
    e.core.scan_source(e.ws, "app").unwrap();
    let deck = e
        .core
        .list_decks(e.ws)
        .unwrap()
        .into_iter()
        .find(|d| d.path == "app")
        .unwrap();
    assert_eq!(deck.cards, 1);
}
