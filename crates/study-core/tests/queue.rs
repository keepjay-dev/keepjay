// SPDX-License-Identifier: AGPL-3.0-or-later
//! Review queue order, deck scope, daily new-card limit and learn-ahead (SRS-004/005).

use std::collections::BTreeMap;

use study_core::{CardType, Core, Id, NewFact, Paths, QueueKind, Rating};

const MIN: i64 = 60_000;

fn setup() -> (tempfile::TempDir, Core, Id) {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    core.init("Uni").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    (dir, core, ws)
}

fn add(core: &mut Core, ws: Id, deck: &str, front: &str) -> Id {
    let fact = core
        .add_fact(
            ws,
            NewFact {
                deck: deck.into(),
                card_type: CardType::Basic,
                fields: BTreeMap::from([
                    ("Front".into(), front.into()),
                    ("Back".into(), format!("answer to {front}")),
                ]),
                allow_duplicate: false,
                tags: Vec::new(),
            },
        )
        .unwrap();
    fact.cards[0].id.parse().unwrap()
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

#[test]
fn new_cards_come_with_rendered_text() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio").unwrap();
    add(&mut core, ws, "Bio", "What produces ATP?");
    let q = core.next_card(ws, None, now()).unwrap().unwrap();
    assert_eq!(q.kind, QueueKind::New);
    assert_eq!(q.question, "What produces ATP?");
    assert_eq!(q.answer, "answer to What produces ATP?");
    assert_eq!(q.deck_path, "Bio");
}

#[test]
fn forgotten_cards_come_back_first_within_the_session() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio").unwrap();
    let first = add(&mut core, ws, "Bio", "one");
    add(&mut core, ws, "Bio", "two");
    let t = now();
    core.answer_card(first, Rating::Again, 2_000, t).unwrap();
    // Not shown before it's due: the other new card comes first...
    let soon = core.next_card(ws, None, t + 1_000).unwrap().unwrap();
    assert_eq!(soon.kind, QueueKind::New);
    let due = core.next_learning_due(ws, None).unwrap().unwrap();
    assert!(due > t && due - t <= 2 * MIN, "comes back within minutes");
    // ...and once due, the forgotten card jumps ahead of new cards.
    let later = core.next_card(ws, None, due).unwrap().unwrap();
    assert_eq!(later.kind, QueueKind::Learning);
    assert_eq!(later.card.id, first.to_string());
}

#[test]
fn a_lone_learning_card_is_not_shown_early() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio").unwrap();
    let only = add(&mut core, ws, "Bio", "one");
    let t = now();
    core.answer_card(only, Rating::Again, 2_000, t).unwrap();
    assert!(core.next_card(ws, None, t + 1_000).unwrap().is_none());
    assert_eq!(core.queue_counts(ws, None, t + 1_000).unwrap().learning, 1);
}

#[test]
fn deck_scope_includes_subdecks_only() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Bio::Cells").unwrap();
    core.create_deck(ws, "Chem").unwrap();
    add(&mut core, ws, "Bio::Cells", "cell");
    add(&mut core, ws, "Chem", "acid");
    let t = now();
    let bio = core.next_card(ws, Some("Bio"), t).unwrap().unwrap();
    assert_eq!(bio.question, "cell");
    assert_eq!(core.queue_counts(ws, Some("Bio"), t).unwrap().new, 1);
    assert_eq!(core.queue_counts(ws, None, t).unwrap().new, 2);
    assert!(core.next_card(ws, Some("Nope"), t).is_err());
}

#[test]
fn daily_new_card_limit_is_enforced() {
    let (_d, mut core, ws) = setup();
    core.create_deck(ws, "Vocab").unwrap();
    for i in 0..25 {
        add(&mut core, ws, "Vocab", &format!("word {i}"));
    }
    let t = now();
    assert_eq!(core.queue_counts(ws, None, t).unwrap().new, 20);
    let mut introduced = 0;
    while let Some(q) = core.next_card(ws, None, t).unwrap() {
        if q.kind != QueueKind::New {
            break;
        }
        // Easy pushes the card days ahead, so only new cards remain in the queue.
        let id: Id = q.card.id.parse().unwrap();
        core.answer_card(id, Rating::Easy, 1_000, t).unwrap();
        introduced += 1;
        assert!(introduced <= 20, "limit exceeded");
    }
    assert_eq!(introduced, 20);
    assert_eq!(core.queue_counts(ws, None, t).unwrap().new, 0);
}
