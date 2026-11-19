// SPDX-License-Identifier: AGPL-3.0-or-later
//! Anki import against real packages exported by Anki 26.09 (IO-001). Fixtures are made by
//! `scripts/fixtures/make_anki_fixtures.py`; `expected.json` describes their content.

use std::path::{Path, PathBuf};

use study_core::{AnkiImportOptions, AnkiImportSummary, Core, Id, Paths};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/anki")
        .join(name)
}

fn import(core: &mut Core, ws: Id, file: &str) -> AnkiImportSummary {
    core.import_anki(ws, &fixture(file), &AnkiImportOptions::default())
        .unwrap_or_else(|e| panic!("{file}: {e}"))
}

fn setup() -> (tempfile::TempDir, Core, Id) {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    core.init("Uni").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    (dir, core, ws)
}

fn check_summary(s: &AnkiImportSummary, format: &str) {
    assert_eq!(s.format, format);
    assert_eq!(s.notes_imported, 8, "{s:#?}");
    assert_eq!(s.cards_imported, 11, "{s:#?}");
    // Basic: 1 real scheduler answer; cloze: 4 answers (the manual reschedule is ignored).
    assert_eq!(s.review_logs_imported, 5, "{s:#?}");
    assert_eq!(s.suspended, 1);
    assert_eq!(s.notes_skipped, 0, "{s:#?}");
    assert_eq!(s.cards_unmatched, 0, "{s:#?}");
    assert_eq!(
        s.decks_created,
        [
            "Biology",
            "Biology::Cells",
            "Geography",
            "Languages",
            "Languages::German"
        ],
        "filtered 'Cram' deck isn't created: its cards go home"
    );
    assert_eq!(s.media_files_not_imported, 1);
    assert_eq!(s.notes_with_tags_not_imported, 2);
    let label = |name: &str| {
        s.note_types
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.imported_as.clone())
            .unwrap_or_else(|| panic!("no report for {name}: {:#?}", s.note_types))
    };
    assert_eq!(label("Basic"), "basic");
    assert_eq!(label("Basic (and reversed card)"), "basic-reverse");
    assert_eq!(label("Basic (optional reversed card)"), "optional-reverse");
    assert_eq!(label("Basic (type in the answer)"), "type-in");
    assert_eq!(label("Cloze"), "cloze");
    assert!(label("Vocab (custom)").starts_with("basic (rendered"));
}

#[test]
fn imports_the_latest_format() {
    let (_d, mut core, ws) = setup();
    check_summary(&import(&mut core, ws, "sample-latest.apkg"), "latest");
}

#[test]
fn imports_the_legacy_format() {
    let (_d, mut core, ws) = setup();
    check_summary(&import(&mut core, ws, "sample-legacy.apkg"), "legacy");
}

#[test]
fn content_history_and_suspension_survive() {
    let (_d, mut core, ws) = setup();
    import(&mut core, ws, "sample-latest.apkg");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;

    // HTML stays lossless in storage but displays as text.
    let cells = core.find_deck(ws, "Biology::Cells").unwrap().unwrap();
    let cards = core.list_cards(cells).unwrap();
    assert_eq!(cards.len(), 3, "basic + two cloze cards");
    let cloze_reviewed = cards
        .iter()
        .find(|c| c.reps == 4)
        .expect("cloze card with history");
    assert_ne!(cloze_reviewed.state, "new");
    assert!(cloze_reviewed.stability.is_some());
    assert!(cloze_reviewed.last_review_at.unwrap() < now);

    // Type-in card: still new, rendered from plain fields.
    let q = core.next_card(ws, Some("Geography"), now).unwrap().unwrap();
    assert_eq!(q.question, "Capital of France?");
    assert_eq!(q.answer, "Paris");

    // Custom note type: rendered through its Anki template, HTML stripped for display.
    let q = core
        .next_card(ws, Some("Languages::German"), now)
        .unwrap()
        .unwrap();
    assert!(!q.question.contains('<'), "{}", q.question);

    // The suspended cloze card isn't offered.
    let counts = core.queue_counts(ws, Some("Biology::Cells"), now).unwrap();
    assert!(counts.new <= 1, "{counts:?}");
}

#[test]
fn reimporting_skips_everything() {
    let (_d, mut core, ws) = setup();
    import(&mut core, ws, "sample-latest.apkg");
    // Same notes from the other format: matched by Anki note guid.
    let again = import(&mut core, ws, "sample-legacy.apkg");
    assert_eq!(again.notes_imported, 0, "{again:#?}");
    assert_eq!(again.cards_imported, 0);
    assert_eq!(again.notes_already_imported, 8);
    assert!(again.decks_created.is_empty());
}

#[test]
fn a_corrupt_package_changes_nothing() {
    let (dir, mut core, ws) = setup();
    let bad = dir.path().join("broken.apkg");
    std::fs::write(&bad, b"not a zip").unwrap();
    let err = core
        .import_anki(ws, &bad, &AnkiImportOptions::default())
        .unwrap_err();
    assert_eq!(err.kind(), study_core::ErrorKind::Validation, "{err}");
    assert!(core.list_decks(ws).unwrap().is_empty());
    let missing = core.import_anki(
        ws,
        &dir.path().join("nope.apkg"),
        &AnkiImportOptions::default(),
    );
    assert_eq!(missing.unwrap_err().kind(), study_core::ErrorKind::NotFound);
}

#[test]
fn deck_prefix_nests_imported_decks() {
    let (_d, mut core, ws) = setup();
    let s = core
        .import_anki(
            ws,
            &fixture("sample-legacy.apkg"),
            &AnkiImportOptions {
                deck_prefix: Some("From Anki".into()),
            },
        )
        .unwrap();
    assert!(
        s.decks_created
            .contains(&"From Anki::Biology::Cells".to_string()),
        "{s:#?}"
    );
    assert!(core.find_deck(ws, "Biology").unwrap().is_none());
}
