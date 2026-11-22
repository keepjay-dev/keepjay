// SPDX-License-Identifier: AGPL-3.0-or-later
//! Decks, facts and cards through the public core API (SRS-001, SRS-002, SRS-007).

use std::collections::BTreeMap;

use study_core::{CardType, Core, ErrorKind, Id, NewFact, Paths};

fn setup() -> (tempfile::TempDir, Core, Id) {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    core.init("Uni").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    (dir, core, ws)
}

fn fields(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn fact(deck: &str, t: CardType, f: &[(&str, &str)]) -> NewFact {
    NewFact {
        deck: deck.into(),
        card_type: t,
        fields: fields(f),
        allow_duplicate: false,
        tags: Vec::new(),
    }
}

#[test]
fn deck_paths_create_missing_parents_and_list_sorted() {
    let (_d, mut core, ws) = setup();
    let cells = core.create_deck(ws, "Biology::Cells").unwrap();
    assert_eq!(cells.name, "Cells");
    assert_eq!(cells.path, "Biology::Cells");
    core.create_deck(ws, "Biology :: Genetics").unwrap(); // whitespace is trimmed
    core.create_deck(ws, "Art").unwrap();

    let paths: Vec<_> = core
        .list_decks(ws)
        .unwrap()
        .into_iter()
        .map(|d| d.path)
        .collect();
    assert_eq!(
        paths,
        ["Art", "Biology", "Biology::Cells", "Biology::Genetics"]
    );

    let err = core.create_deck(ws, "Biology::Cells").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Conflict);
    let err = core.create_deck(ws, "Biology::::Oops").unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Validation);
}

#[test]
fn facts_generate_cards_per_card_type() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio").unwrap();

    let basic = core
        .add_fact(
            ws,
            fact(
                "Bio",
                CardType::Basic,
                &[("Front", "ATP?"), ("Back", "Energy")],
            ),
        )
        .unwrap();
    assert_eq!(basic.cards.len(), 1);
    assert_eq!(basic.cards[0].state, "new");
    assert_eq!(basic.cards[0].due, None);

    let reverse = core
        .add_fact(
            ws,
            fact(
                "Bio",
                CardType::BasicReverse,
                &[("Front", "Cell"), ("Back", "Zelle")],
            ),
        )
        .unwrap();
    assert_eq!(
        reverse.cards.iter().map(|c| c.ord).collect::<Vec<_>>(),
        [0, 1]
    );

    let cloze = core
        .add_fact(
            ws,
            fact(
                "Bio",
                CardType::Cloze,
                &[("Text", "{{c1::Mitochondria}} produce {{c2::ATP}}")],
            ),
        )
        .unwrap();
    assert_eq!(cloze.cards.len(), 2);

    let deck = core.find_deck(ws, "Bio").unwrap().unwrap();
    assert_eq!(core.list_cards(deck).unwrap().len(), 5);
    assert_eq!(core.list_decks(ws).unwrap()[0].cards, 5);
}

#[test]
fn duplicates_are_rejected_unless_allowed() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio").unwrap();
    let qa = &[("Front", " What produces ATP? "), ("Back", "Mitochondria")];
    core.add_fact(ws, fact("Bio", CardType::Basic, qa)).unwrap();

    let err = core
        .add_fact(
            ws,
            fact("Bio", CardType::Basic, &[("Front", "What produces ATP?")]),
        )
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Conflict, "{err}");

    let mut allowed = fact("Bio", CardType::Basic, qa);
    allowed.allow_duplicate = true;
    core.add_fact(ws, allowed).unwrap();

    // Same front in a different card type or deck is not a duplicate.
    core.add_fact(ws, fact("Bio", CardType::TypeIn, qa))
        .unwrap();
}

#[test]
fn missing_deck_and_bad_fields_are_reported() {
    let (_d, mut core, ws) = setup();
    let err = core
        .add_fact(ws, fact("Nope", CardType::Basic, &[("Front", "Q")]))
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
    assert!(err.to_string().contains("scrubjay deck create"), "{err}");

    core.create_deck(ws, "Bio").unwrap();
    let err = core
        .add_fact(ws, fact("Bio", CardType::Cloze, &[("Text", "no deletion")]))
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Validation);
}

#[test]
fn writes_appear_in_the_change_log_for_other_processes() {
    let (dir, mut core, ws) = setup();
    let other = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    let since = other.last_change_seq().unwrap();

    core.create_deck(ws, "Bio").unwrap();
    core.add_fact(
        ws,
        fact(
            "Bio",
            CardType::BasicReverse,
            &[("Front", "Q"), ("Back", "A")],
        ),
    )
    .unwrap();

    let entities: Vec<_> = other
        .changes_since(since)
        .unwrap()
        .into_iter()
        .map(|c| c.entity)
        .collect();
    assert_eq!(entities, ["deck", "fact", "card", "card"]);
}

#[test]
fn workspaces_resolve_by_name_or_default() {
    let (_d, mut core, first) = setup();
    core.create_workspace("Japanese").unwrap();
    assert_eq!(core.resolve_workspace(None).unwrap(), first);
    assert_ne!(core.resolve_workspace(Some("Japanese")).unwrap(), first);
    let err = core.resolve_workspace(Some("Nope")).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotFound);
}
