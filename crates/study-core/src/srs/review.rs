// SPDX-License-Identifier: AGPL-3.0-or-later
//! Answering cards, previews, due counts and review-log replay (SRS-005, SRS-006).

use rusqlite::{OptionalExtension, params};
use serde::Serialize;

use super::{
    DeckOptions, FsrsScheduler, Memory, Rating, Scheduler, elapsed_days, next_card_state,
    schedule_interval,
};
use crate::Core;
use crate::cards::{CARD_COLUMNS, Card, card_from_row};
use crate::domain::{self, Id};
use crate::error::{Error, Result};
use crate::write::{ReviewRow, Schedule, Writer};

/// What one button would do, for the review screen (SRS-005).
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IntervalPreview {
    pub rating: Rating,
    pub interval_days: f64,
    pub due: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnswerOutcome {
    pub card: Card,
    pub review_log_id: String,
    pub interval_days: f64,
}

/// Cards waiting in a workspace.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, Default)]
pub struct DueCounts {
    /// Learning/relearning cards due now.
    pub learning: u64,
    /// Review cards due by the cutoff (normally the end of the study day).
    pub review: u64,
    /// Cards never studied.
    pub new: u64,
}

/// Result of applying one answer to a card state. Pure, so answering and replay agree.
pub(crate) struct Applied {
    pub state: &'static str,
    pub lapse: bool,
    pub memory: Memory,
    pub interval_days: f64,
    pub due: i64,
}

pub(crate) fn apply(
    scheduler: &dyn Scheduler,
    options: &DeckOptions,
    state_before: &str,
    memory: Option<Memory>,
    last_review_at: Option<i64>,
    rating: Rating,
    now_ms: i64,
) -> Result<Applied> {
    let steps = scheduler.next_steps(memory, elapsed_days(last_review_at, now_ms))?;
    let step = steps[rating as usize - 1];
    let (interval_days, interval_ms) = schedule_interval(step.interval_days, options);
    let (state, lapse) = next_card_state(state_before, rating, interval_days);
    Ok(Applied {
        state,
        lapse,
        memory: step.memory,
        interval_days,
        due: now_ms + interval_ms,
    })
}

impl Core {
    /// One card by id.
    pub fn card(&self, id: Id) -> Result<Card> {
        self.conn
            .query_row(
                &format!("SELECT {CARD_COLUMNS} FROM cards WHERE id = ?1 AND deleted_at IS NULL"),
                [id.as_bytes()],
                card_from_row,
            )
            .optional()?
            .ok_or_else(|| Error::NotFound(format!("no card {id}")))
    }

    fn memory_of(&self, id: Id) -> Result<Option<Memory>> {
        let (s, d, f): (Option<f64>, Option<f64>, Option<f64>) = self.conn.query_row(
            "SELECT stability, difficulty, stability_fast FROM cards WHERE id = ?1",
            [id.as_bytes()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        Ok(match (s, d, f) {
            (Some(stability), Some(difficulty), Some(stability_fast)) => Some(Memory {
                stability,
                difficulty,
                stability_fast,
            }),
            _ => None,
        })
    }

    fn scheduler_for(&self, _card: &Card) -> Result<(FsrsScheduler, DeckOptions)> {
        // Per-deck options documents arrive with SRS-004 editing; built-in defaults for now.
        let options = DeckOptions::default();
        Ok((FsrsScheduler::new(options.desired_retention)?, options))
    }

    /// What each button would schedule if the card were answered at `now_ms`.
    pub fn preview_intervals(&self, card_id: Id, now_ms: i64) -> Result<Vec<IntervalPreview>> {
        let card = self.card(card_id)?;
        let (scheduler, options) = self.scheduler_for(&card)?;
        let memory = self.memory_of(card_id)?;
        Rating::ALL
            .into_iter()
            .map(|rating| {
                let a = apply(
                    &scheduler,
                    &options,
                    &card.state,
                    memory,
                    card.last_review_at,
                    rating,
                    now_ms,
                )?;
                Ok(IntervalPreview {
                    rating,
                    interval_days: a.interval_days,
                    due: a.due,
                })
            })
            .collect()
    }

    /// Record an answer (SRS-005, SRS-006): append to the review log and update the card's
    /// cached scheduling state in one transaction.
    pub fn answer_card(
        &mut self,
        card_id: Id,
        rating: Rating,
        answer_ms: i64,
        now_ms: i64,
    ) -> Result<AnswerOutcome> {
        let card = self.card(card_id)?;
        let suspended: bool = self.conn.query_row(
            "SELECT suspended FROM cards WHERE id = ?1",
            [card_id.as_bytes()],
            |r| r.get(0),
        )?;
        if suspended {
            return Err(Error::validation(format!(
                "card {} is suspended; unsuspend it to review",
                card.short_id
            )));
        }
        let memory_before = self.memory_of(card_id)?;
        let (scheduler, options) = self.scheduler_for(&card)?;
        let a = apply(
            &scheduler,
            &options,
            &card.state,
            memory_before,
            card.last_review_at,
            rating,
            now_ms,
        )?;

        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        let log_id = w.insert_review(&ReviewRow {
            card: card_id,
            reviewed_at: now_ms,
            rating,
            answer_ms,
            state_before: &card.state,
            memory_before,
            memory_after: a.memory,
            interval_days: a.interval_days,
            due_after: a.due,
            scheduler: scheduler.id(),
        })?;
        w.update_schedule(
            card_id,
            &Schedule {
                state: a.state,
                due: Some(a.due),
                memory: Some(a.memory),
                reps: card.reps + 1,
                lapses: card.lapses + a.lapse as u32,
                last_review_at: Some(now_ms),
            },
        )?;
        tx.commit()?;

        Ok(AnswerOutcome {
            card: self.card(card_id)?,
            review_log_id: log_id.to_string(),
            interval_days: a.interval_days,
        })
    }

    /// Recompute a card's scheduling state from its review log (SRS-006). The result must
    /// match the stored state; `keepjay doctor --fix` will use this to repair a cache.
    pub fn rebuild_card_state(&mut self, card_id: Id) -> Result<Card> {
        let card = self.card(card_id)?;
        let schedule = self.replayed_schedule(&card, None)?;
        Writer {
            conn: &self.conn,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        }
        .update_schedule(card_id, &schedule)?;
        self.card(card_id)
    }

    /// Undo an answer (SRS-005, D-041): append an undo record and put the card back as it
    /// was before the answer, by replaying its remaining answers. Only a card's latest
    /// answer can be undone. Returns the card as it is now.
    pub fn undo_answer(&mut self, review_log_id: Id) -> Result<Card> {
        let card_id: Option<Vec<u8>> = self
            .conn
            .query_row(
                "SELECT card_id FROM live_review_logs WHERE id = ?1",
                [review_log_id.as_bytes()],
                |r| r.get(0),
            )
            .optional()?;
        let card_id =
            Id::from_slice(&card_id.ok_or_else(|| Error::NotFound("no answer to undo".into()))?)?;
        let latest: Vec<u8> = self.conn.query_row(
            "SELECT id FROM live_review_logs WHERE card_id = ?1
             ORDER BY reviewed_at DESC, id DESC LIMIT 1",
            [card_id.as_bytes()],
            |r| r.get(0),
        )?;
        if latest != review_log_id.as_bytes() {
            return Err(Error::validation(
                "only a card's latest answer can be undone",
            ));
        }
        let card = self.card(card_id)?;
        let schedule = self.replayed_schedule(&card, Some(review_log_id))?;
        let tx = self
            .conn
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let mut w = Writer {
            conn: &tx,
            clock: &mut self.clock,
            device: self.device_id,
            now: domain::now_ms(),
        };
        w.insert_review_undo(review_log_id, card_id)?;
        w.update_schedule(card_id, &schedule)?;
        tx.commit()?;
        self.card(card_id)
    }

    /// The scheduling state from replaying a card's live answers, leaving out `skip`.
    fn replayed_schedule(&self, card: &Card, skip: Option<Id>) -> Result<Schedule<'static>> {
        let card_id: Id = card.id.parse()?;
        let (scheduler, options) = self.scheduler_for(card)?;
        let skip = skip.map(|id| id.as_bytes().to_vec());
        let answers: Vec<(i64, u8)> = {
            let mut stmt = self.conn.prepare(
                "SELECT reviewed_at, rating FROM live_review_logs
                 WHERE card_id = ?1 AND (?2 IS NULL OR id != ?2)
                 ORDER BY reviewed_at, id",
            )?;
            stmt.query_map(params![card_id.as_bytes(), skip], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?
            .collect::<rusqlite::Result<_>>()?
        };

        let mut state: &'static str = "new";
        let mut memory: Option<Memory> = None;
        let mut last: Option<i64> = None;
        let mut due: Option<i64> = None;
        let mut lapses = 0u32;
        for (reviewed_at, rating) in &answers {
            let rating = Rating::from_number(*rating)?;
            let a = apply(
                &scheduler,
                &options,
                state,
                memory,
                last,
                rating,
                *reviewed_at,
            )?;
            state = a.state;
            memory = Some(a.memory);
            last = Some(*reviewed_at);
            due = Some(a.due);
            lapses += a.lapse as u32;
        }
        Ok(Schedule {
            state,
            due,
            memory,
            reps: answers.len() as u32,
            lapses,
            last_review_at: last,
        })
    }

    /// Counts for `keepjay status` and the Today widget. `cutoff_ms` is normally the end of
    /// the current study day (`srs.day_starts_at`).
    pub fn due_counts(&self, workspace: Id, now_ms: i64, cutoff_ms: i64) -> Result<DueCounts> {
        let (learning, review, new) = self.conn.query_row(
            "SELECT
                 COALESCE(SUM(c.state IN ('learning', 'relearning') AND c.due <= ?2), 0),
                 COALESCE(SUM(c.state = 'review' AND c.due <= ?3), 0),
                 COALESCE(SUM(c.state = 'new'), 0)
             FROM cards c JOIN decks d ON d.id = c.deck_id
             WHERE d.workspace_id = ?1 AND c.deleted_at IS NULL AND d.deleted_at IS NULL
               AND c.suspended = 0",
            params![workspace.as_bytes(), now_ms, cutoff_ms],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        Ok(DueCounts {
            learning,
            review,
            new,
        })
    }
}

/// Write an imported answer history for a new card and set its schedule from it (IO-001).
/// Answers are applied in order with the default scheduler, exactly like live answers.
pub(crate) fn replay_into(
    w: &mut Writer<'_>,
    card: Id,
    answers: &[(i64, Rating, i64)],
) -> Result<(u32, u32)> {
    let options = DeckOptions::default();
    let scheduler = FsrsScheduler::new(options.desired_retention)?;
    let mut state: &str = "new";
    let mut memory: Option<Memory> = None;
    let mut last: Option<i64> = None;
    let mut due: Option<i64> = None;
    let mut lapses = 0u32;
    for &(reviewed_at, rating, answer_ms) in answers {
        let a = apply(
            &scheduler,
            &options,
            state,
            memory,
            last,
            rating,
            reviewed_at,
        )?;
        w.insert_review(&ReviewRow {
            card,
            reviewed_at,
            rating,
            answer_ms,
            state_before: state,
            memory_before: memory,
            memory_after: a.memory,
            interval_days: a.interval_days,
            due_after: a.due,
            scheduler: scheduler.id(),
        })?;
        state = a.state;
        memory = Some(a.memory);
        last = Some(reviewed_at);
        due = Some(a.due);
        lapses += a.lapse as u32;
    }
    if !answers.is_empty() {
        w.update_schedule(
            card,
            &Schedule {
                state,
                due,
                memory,
                reps: answers.len() as u32,
                lapses,
                last_review_at: last,
            },
        )?;
    }
    Ok((answers.len() as u32, lapses))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{CardType, Core, NewFact, Paths, Rating};

    fn one_card() -> (tempfile::TempDir, Core, crate::Id) {
        let dir = tempfile::tempdir().unwrap();
        let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
        core.init("Uni").unwrap();
        let ws = core.resolve_workspace(None).unwrap();
        core.create_deck(ws, "Bio").unwrap();
        let fields = BTreeMap::from([("Front".to_string(), "Q".to_string())]);
        let fact = core
            .add_fact(
                ws,
                NewFact {
                    deck: "Bio".into(),
                    card_type: CardType::Basic,
                    fields,
                    allow_duplicate: false,
                    tags: Vec::new(),
                },
            )
            .unwrap();
        let id = fact.cards[0].id.parse().unwrap();
        (dir, core, id)
    }

    #[test]
    fn rebuild_repairs_a_corrupted_cache() {
        let (_d, mut core, id) = one_card();
        let mut now = 1_700_000_000_000;
        for r in [Rating::Good, Rating::Good, Rating::Again, Rating::Good] {
            let out = core.answer_card(id, r, 4_000, now).unwrap();
            now = out.card.due.unwrap() + 3_600_000;
        }
        let good = core.card(id).unwrap();
        core.conn
            .execute(
                "UPDATE cards SET state = 'new', due = NULL, stability = 99, reps = 0 WHERE id = ?1",
                [id.as_bytes()],
            )
            .unwrap();
        assert_ne!(core.card(id).unwrap(), good);
        assert_eq!(core.rebuild_card_state(id).unwrap(), good);
    }

    #[test]
    fn undo_puts_the_card_back_and_keeps_the_log() {
        let (_d, mut core, id) = one_card();
        let ws = core.resolve_workspace(None).unwrap();
        let mut now = 1_700_000_000_000;
        let mut states = vec![core.card(id).unwrap()];
        let mut logs = Vec::new();
        for r in [Rating::Good, Rating::Again, Rating::Good] {
            let out = core.answer_card(id, r, 4_000, now).unwrap();
            logs.push(out.review_log_id.parse::<crate::Id>().unwrap());
            states.push(out.card.clone());
            now = out.card.due.unwrap() + 3_600_000;
        }
        // An earlier answer can't be undone while a later one stands.
        let err = core.undo_answer(logs[1]).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Validation);

        // Undo the last two, one at a time: each restores the state before it exactly.
        assert_eq!(core.undo_answer(logs[2]).unwrap(), states[2]);
        assert_eq!(core.undo_answer(logs[1]).unwrap(), states[1]);
        // An answer is undone once.
        let err = core.undo_answer(logs[1]).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::NotFound);

        // The log keeps all three answers (append-only, D-041); replay agrees with the cache.
        let rows: i64 = core
            .conn
            .query_row("SELECT COUNT(*) FROM review_logs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows, 3);
        assert_eq!(core.rebuild_card_state(id).unwrap(), states[1]);
        let changes = core.changes_since(0).unwrap();
        assert_eq!(
            changes.iter().filter(|c| c.entity == "review_undo").count(),
            2
        );

        // Undo the first answer too: the card is new again and counts as new today.
        assert_eq!(core.undo_answer(logs[0]).unwrap(), states[0]);
        let today = core.today(ws, now).unwrap();
        assert_eq!(today.reviewed, 0);
        assert_eq!(today.due.new, 1);
    }

    #[test]
    fn suspended_cards_cannot_be_answered() {
        let (_d, mut core, id) = one_card();
        core.conn
            .execute(
                "UPDATE cards SET suspended = 1 WHERE id = ?1",
                [id.as_bytes()],
            )
            .unwrap();
        let err = core.answer_card(id, Rating::Good, 1_000, 0).unwrap_err();
        assert_eq!(err.kind(), crate::ErrorKind::Validation);
    }
}
