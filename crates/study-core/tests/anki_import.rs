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
    // "bio cells" on the Basic note, "de" on the reversed one.
    assert_eq!(s.notes_tagged, 2, "{s:#?}");
    assert_eq!(s.tags_created, 3, "{s:#?}");
    assert_eq!(s.tags_skipped, 0);
    assert_eq!(s.media_files, 1);
    assert_eq!(s.media_added, 1, "{s:#?}");
    assert_eq!(
        (
            s.media_skipped,
            s.media_missing,
            s.media_unused,
            s.media_renamed
        ),
        (0, 0, 0, 0),
        "{s:#?}"
    );
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
fn note_tags_land_on_facts_and_reuse_existing_tags() {
    let (_d, mut core, ws) = setup();
    // An existing tag with different case is reused, keeping its spelling (like Anki).
    core.create_deck(ws, "Mine").unwrap();
    core.add_fact(
        ws,
        study_core::NewFact {
            deck: "Mine".into(),
            card_type: study_core::CardType::Basic,
            fields: [("Front", "q"), ("Back", "a")]
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            allow_duplicate: false,
            tags: vec!["Bio".into()],
        },
    )
    .unwrap();

    let s = import(&mut core, ws, "sample-legacy.apkg");
    assert_eq!(
        s.tags_created, 2,
        "cells and de are new; Bio exists: {s:#?}"
    );

    let cells = core.find_deck(ws, "Biology::Cells").unwrap().unwrap();
    let basic = core
        .list_cards(cells)
        .unwrap()
        .into_iter()
        .find(|c| c.reps == 1)
        .expect("the reviewed Basic card");
    let fact: Id = basic.fact_id.parse().unwrap();
    assert_eq!(core.fact_tags(fact).unwrap(), ["Bio", "cells"]);

    let tags: Vec<(String, u64)> = core
        .list_tags(ws)
        .unwrap()
        .into_iter()
        .map(|t| (t.path, t.facts))
        .collect();
    assert_eq!(
        tags,
        [("Bio".into(), 2), ("cells".into(), 1), ("de".into(), 1)]
    );

    // Re-importing adds no tags.
    let again = import(&mut core, ws, "sample-latest.apkg");
    assert_eq!((again.tags_created, again.notes_tagged), (0, 0));
    assert_eq!(core.list_tags(ws).unwrap().len(), 3);
}

/// The stored content of the fixture's `cell.png` (scripts/fixtures/make_anki_fixtures.py).
const CELL_PNG: &[u8] = b"\x89PNG fixture";

fn basic_back_field(dir: &Path) -> String {
    let db = rusqlite::Connection::open(dir.join("study.db")).unwrap();
    db.query_row(
        "SELECT json_extract(fields, '$.Back') FROM facts WHERE source LIKE 'anki:%'
           AND json_extract(fields, '$.Back') LIKE 'Mitochondria%'",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn media_is_stored_by_content_and_found_by_name() {
    for file in ["sample-latest.apkg", "sample-legacy.apkg"] {
        let (dir, mut core, ws) = setup();
        import(&mut core, ws, file);
        let path = core.media_file(ws, "cell.png").unwrap().expect(file);
        assert!(path.starts_with(dir.path().join("media")), "{path:?}");
        assert!(path.to_string_lossy().ends_with(".png"));
        assert_eq!(std::fs::read(&path).unwrap(), CELL_PNG, "{file}");
        assert_eq!(
            basic_back_field(dir.path()),
            r#"Mitochondria<br><img src="cell.png">"#
        );
        assert!(core.media_file(ws, "nope.png").unwrap().is_none());

        // Same file again (other format): nothing new, still one file on disk.
        let other = if file.contains("latest") {
            "sample-legacy.apkg"
        } else {
            "sample-latest.apkg"
        };
        let again = import(&mut core, ws, other);
        assert_eq!(
            (again.media_added, again.media_already_present),
            (0, 1),
            "{again:#?}"
        );
        let files = walk(&dir.path().join("media"));
        assert_eq!(files.len(), 1, "{files:?}");
    }
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            out.extend(walk(&p));
        } else {
            out.push(p);
        }
    }
    out
}

#[test]
fn a_taken_media_name_gets_a_hashed_name_and_fields_follow() {
    let (dir, mut core, ws) = setup();
    let mine = core
        .add_media(ws, "cell.png", &b"my own picture"[..])
        .unwrap();
    assert_eq!(mine.name, "cell.png");

    let s = import(&mut core, ws, "sample-latest.apkg");
    assert_eq!((s.media_added, s.media_renamed), (0, 1), "{s:#?}");
    assert_eq!(s.media_missing, 0, "{s:#?}");

    let back = basic_back_field(dir.path());
    let renamed = back
        .strip_prefix(r#"Mitochondria<br><img src=""#)
        .and_then(|r| r.strip_suffix(r#"">"#))
        .unwrap_or_else(|| panic!("{back}"));
    assert!(
        renamed.starts_with("cell-") && renamed.ends_with(".png"),
        "{renamed}"
    );
    let theirs = core.media_file(ws, renamed).unwrap().unwrap();
    assert_eq!(std::fs::read(theirs).unwrap(), CELL_PNG);
    let ours = core.media_file(ws, "cell.png").unwrap().unwrap();
    assert_eq!(std::fs::read(ours).unwrap(), b"my own picture");
}

#[test]
fn unsafe_media_names_are_skipped_and_reported_missing() {
    let (dir, mut core, ws) = setup();
    // Rewrite the legacy package's media map so the file claims a path-like name.
    let src = fixture("sample-legacy.apkg");
    let evil = dir.path().join("evil.apkg");
    let mut zin = zip::ZipArchive::new(std::fs::File::open(&src).unwrap()).unwrap();
    let mut zout = zip::ZipWriter::new(std::fs::File::create(&evil).unwrap());
    for i in 0..zin.len() {
        let mut f = zin.by_index(i).unwrap();
        let name = f.name().unwrap().to_string();
        zout.start_file(name.as_str(), zip::write::SimpleFileOptions::default())
            .unwrap();
        if name == "media" {
            use std::io::Write;
            zout.write_all(br#"{"0": "../../cell.png"}"#).unwrap();
        } else {
            std::io::copy(&mut f, &mut zout).unwrap();
        }
    }
    zout.finish().unwrap();

    let s = import(&mut core, ws, evil.to_str().unwrap());
    assert_eq!(
        (s.media_files, s.media_skipped, s.media_added),
        (1, 1, 0),
        "{s:#?}"
    );
    assert_eq!(
        s.media_missing, 1,
        "cell.png is referenced but wasn't imported"
    );
    assert!(!dir.path().join("cell.png").exists());
    assert!(!dir.path().join("media").exists() || walk(&dir.path().join("media")).is_empty());
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
