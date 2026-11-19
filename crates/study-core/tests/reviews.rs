// SPDX-License-Identifier: AGPL-3.0-or-later
//! Answering cards with FSRS, the review log, replay and due counts (SRS-003/005/006).

use std::collections::BTreeMap;

use study_core::{Card, CardType, Core, Id, NewFact, Paths, Rating};

const T0: i64 = 1_763_000_000_000; // fixed clock for deterministic tests
const HOUR: i64 = 3_600_000;
const DAY: i64 = 24 * HOUR;

fn setup() -> (tempfile::TempDir, Core, Id, Id) {
    let dir = tempfile::tempdir().unwrap();
    let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    core.init("Uni").unwrap();
    let ws = core.resolve_workspace(None).unwrap();
    core.create_deck(ws, "Bio").unwrap();
    let fact = core
        .add_fact(
            ws,
            NewFact {
                deck: "Bio".into(),
                card_type: CardType::Basic,
                fields: BTreeMap::from([
                    ("Front".into(), "What produces ATP?".into()),
                    ("Back".into(), "Mitochondria".into()),
                ]),
                allow_duplicate: false,
                tags: Vec::new(),
            },
        )
        .unwrap();
    let card: Id = fact.cards[0].id.parse().unwrap();
    (dir, core, ws, card)
}

/// Answer, then jump the clock to just after the card is due.
fn answer(core: &mut Core, card: Id, rating: Rating, now: &mut i64) -> Card {
    let out = core.answer_card(card, rating, 5_000, *now).unwrap();
    *now = out.card.due.unwrap() + HOUR;
    out.card
}

#[test]
fn first_answer_moves_a_new_card_out_of_new() {
    let (_d, mut core, _ws, card) = setup();
    let out = core.answer_card(card, Rating::Good, 6_000, T0).unwrap();
    assert_ne!(out.card.state, "new");
    assert_eq!(out.card.reps, 1);
    assert_eq!(out.card.last_review_at, Some(T0));
    assert!(out.card.due.unwrap() > T0);
    assert!(out.card.stability.unwrap() > 0.0);
    assert!(!out.review_log_id.is_empty());
}

#[test]
fn good_answers_grow_intervals_and_again_is_a_lapse() {
    let (_d, mut core, _ws, card) = setup();
    let mut now = T0;
    let mut gaps = Vec::new();
    for _ in 0..5 {
        let before = now;
        let c = answer(&mut core, card, Rating::Good, &mut now);
        gaps.push(c.due.unwrap() - before);
    }
    assert!(
        gaps.windows(2).all(|w| w[1] > w[0]),
        "intervals grow: {gaps:?}"
    );
    let c = core.card(card).unwrap();
    assert_eq!(c.state, "review");
    assert_eq!(c.lapses, 0);

    let before = now;
    let lapsed = answer(&mut core, card, Rating::Again, &mut now);
    assert_eq!(lapsed.lapses, 1);
    assert_eq!(lapsed.reps, 6);
    // FSRS-7 decides how soon a forgotten card returns: for a well-known card that can
    // still be a day or more (state stays `review`), but the interval must shrink sharply.
    let lapse_gap = lapsed.due.unwrap() - before;
    assert!(
        lapse_gap * 3 < *gaps.last().unwrap(),
        "{lapse_gap} vs {gaps:?}"
    );
}

#[test]
fn forgetting_a_young_card_sends_it_to_short_term_relearning() {
    let (_d, mut core, _ws, card) = setup();
    let mut now = T0;
    let mut c = answer(&mut core, card, Rating::Good, &mut now);
    while c.state != "review" {
        c = answer(&mut core, card, Rating::Good, &mut now);
    }
    let before_lapse = now;
    let lapsed = answer(&mut core, card, Rating::Again, &mut now);
    assert_eq!(lapsed.lapses, 1);
    assert_eq!(lapsed.state, "relearning");
    assert!(
        lapsed.due.unwrap() - before_lapse < HOUR,
        "comes back within the session"
    );
}

#[test]
fn previews_match_what_answering_does() {
    let (_d, mut core, _ws, card) = setup();
    let previews = core.preview_intervals(card, T0).unwrap();
    assert_eq!(previews.len(), 4);
    assert!(
        previews
            .windows(2)
            .all(|w| w[0].interval_days <= w[1].interval_days)
    );
    let good = previews.iter().find(|p| p.rating == Rating::Good).unwrap();
    let out = core.answer_card(card, Rating::Good, 1_000, T0).unwrap();
    assert_eq!(out.card.due, Some(good.due));
}

#[test]
fn replaying_the_review_log_reproduces_the_card_exactly() {
    let (_d, mut core, _ws, card) = setup();
    let mut now = T0;
    for r in [
        Rating::Good,
        Rating::Hard,
        Rating::Good,
        Rating::Again,
        Rating::Good,
        Rating::Easy,
    ] {
        answer(&mut core, card, r, &mut now);
    }
    let stored = core.card(card).unwrap();
    let rebuilt = core.rebuild_card_state(card).unwrap();
    assert_eq!(rebuilt, stored, "D-009: state is derivable from the log");
}

#[test]
fn due_counts_follow_the_clock() {
    let (_d, mut core, ws, card) = setup();
    let c = core.due_counts(ws, T0, T0 + DAY).unwrap();
    assert_eq!((c.new, c.learning, c.review), (1, 0, 0));

    let mut now = T0;
    let mut answered = answer(&mut core, card, Rating::Good, &mut now);
    while answered.state != "review" {
        answered = answer(&mut core, card, Rating::Good, &mut now);
    }
    let due = answered.due.unwrap();
    let before = core.due_counts(ws, due - DAY, due - 1).unwrap();
    assert_eq!((before.new, before.review), (0, 0));
    let after = core.due_counts(ws, due, due + DAY).unwrap();
    assert_eq!(after.review, 1);
}

#[test]
fn answers_are_visible_to_other_processes() {
    let (dir, mut core, _ws, card) = setup();
    let other = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
    let since = other.last_change_seq().unwrap();
    core.answer_card(card, Rating::Good, 1_000, T0).unwrap();
    let entities: Vec<_> = other
        .changes_since(since)
        .unwrap()
        .into_iter()
        .map(|c| (c.entity, c.op))
        .collect();
    assert_eq!(
        entities,
        [
            ("review_log".to_string(), "insert".to_string()),
            ("card".to_string(), "update".to_string())
        ]
    );
}
