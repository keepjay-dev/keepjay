// SPDX-License-Identifier: AGPL-3.0-or-later
//! Spaced-repetition scheduling (SRS-003, SRS-006; D-005).
//!
//! The [`Scheduler`] trait keeps the algorithm pluggable. FSRS (via the `fsrs` crate,
//! FSRS-7 default parameters) is the default. Every answer is written to the append-only
//! review log, and a card's scheduling columns can always be rebuilt by replaying it.

mod queue;
mod review;

pub use queue::{
    DAY_STARTS_AT_HOUR, LEARNING_WAIT_MS, QueueKind, QueuedCard, StudyDay, study_day_in,
};
pub use review::{AnswerOutcome, DueCounts, IntervalPreview};

use serde::Serialize;

use crate::error::{Error, Result};

const DAY_MS: f64 = 86_400_000.0;

/// The four answer buttons (SRS-005).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Rating {
    pub const ALL: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];

    pub fn from_number(n: u8) -> Result<Self> {
        Ok(match n {
            1 => Rating::Again,
            2 => Rating::Hard,
            3 => Rating::Good,
            4 => Rating::Easy,
            _ => return Err(Error::validation(format!("rating must be 1–4, got {n}"))),
        })
    }

    /// Accepts `1`–`4` or `again` / `hard` / `good` / `easy`.
    pub fn parse(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "again" => Ok(Rating::Again),
            "hard" => Ok(Rating::Hard),
            "good" => Ok(Rating::Good),
            "easy" => Ok(Rating::Easy),
            other => other
                .parse::<u8>()
                .map_err(|_| Error::validation(format!("unknown rating '{s}'")))
                .and_then(Rating::from_number),
        }
    }
}

/// Memory state tracked per card. FSRS-7 adds a fast-stability trace.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Memory {
    pub stability: f64,
    pub difficulty: f64,
    pub stability_fast: f64,
}

/// Where a card goes after one answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    pub memory: Memory,
    /// Unrounded interval suggested by the algorithm, in days.
    pub interval_days: f64,
}

/// A spaced-repetition algorithm (SRS-003). Implementations must be deterministic:
/// replaying the same answers must give the same state (SRS-006).
pub trait Scheduler {
    /// Stored in every review log entry, e.g. `fsrs@7.0.0`.
    fn id(&self) -> &str;

    /// Next memory state and interval for each rating, given the current memory (`None`
    /// for a new card) and the days elapsed since the previous review.
    fn next_steps(&self, memory: Option<Memory>, elapsed_days: f64) -> Result<[Step; 4]>;
}

/// FSRS via the reference `fsrs` crate.
pub struct FsrsScheduler {
    fsrs: fsrs::FSRS,
    desired_retention: f32,
}

impl FsrsScheduler {
    pub fn new(desired_retention: f64) -> Result<Self> {
        if !(0.7..=0.99).contains(&desired_retention) {
            return Err(Error::validation(format!(
                "desired retention must be between 0.70 and 0.99, got {desired_retention}"
            )));
        }
        Ok(FsrsScheduler {
            fsrs: fsrs::FSRS::default(),
            desired_retention: desired_retention as f32,
        })
    }
}

impl Scheduler for FsrsScheduler {
    fn id(&self) -> &str {
        "fsrs@7.0.0"
    }

    fn next_steps(&self, memory: Option<Memory>, elapsed_days: f64) -> Result<[Step; 4]> {
        let current = memory.map(|m| fsrs::MemoryState {
            stability: m.stability as f32,
            difficulty: m.difficulty as f32,
            stability_fast: m.stability_fast as f32,
        });
        let next = self
            .fsrs
            .next_states_with_elapsed_days(
                current,
                self.desired_retention,
                elapsed_days.max(0.0) as f32,
            )
            .map_err(|e| Error::Unexpected(format!("FSRS failed: {e}")))?;
        let step = |s: &fsrs::ItemState| Step {
            memory: Memory {
                stability: s.memory.stability as f64,
                difficulty: s.memory.difficulty as f64,
                stability_fast: s.memory.stability_fast as f64,
            },
            interval_days: s.interval as f64,
        };
        Ok([
            step(&next.again),
            step(&next.hard),
            step(&next.good),
            step(&next.easy),
        ])
    }
}

/// Deck options that affect scheduling (SRS-004). Stored as a `deck_options` document
/// later (D-006); for now every deck uses the built-in defaults.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeckOptions {
    pub desired_retention: f64,
    pub max_interval_days: f64,
    /// New cards introduced per study day.
    pub new_per_day: u32,
}

impl Default for DeckOptions {
    fn default() -> Self {
        DeckOptions {
            desired_retention: 0.9,
            max_interval_days: 36_500.0,
            new_per_day: 20,
        }
    }
}

/// Smallest gap before a card comes back, so "Again" never shows a card instantly.
const MIN_INTERVAL_MS: f64 = 60_000.0;

/// Turn the algorithm's interval into a due time and the interval actually used.
/// Under a day: kept as is (minimum 1 minute). A day or more: whole days, capped.
pub fn schedule_interval(interval_days: f64, options: &DeckOptions) -> (f64, i64) {
    let days = if interval_days < 1.0 {
        interval_days.max(MIN_INTERVAL_MS / DAY_MS)
    } else {
        interval_days.round().clamp(1.0, options.max_interval_days)
    };
    (days, (days * DAY_MS).round() as i64)
}

/// Card state after an answer (docs/specs/12-glossary: lapse).
pub fn next_card_state(before: &str, rating: Rating, interval_days: f64) -> (&'static str, bool) {
    let short_term = interval_days < 1.0;
    let lapse = rating == Rating::Again && matches!(before, "review");
    let state = match (before, short_term) {
        (_, false) => "review",
        ("review" | "relearning", true) if lapse || before == "relearning" => "relearning",
        (_, true) => "learning",
    };
    (state, lapse)
}

pub(crate) fn elapsed_days(last_review_at: Option<i64>, now_ms: i64) -> f64 {
    last_review_at
        .map(|last| ((now_ms - last).max(0)) as f64 / DAY_MS)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratings_parse_from_numbers_and_words() {
        assert_eq!(Rating::parse("3").unwrap(), Rating::Good);
        assert_eq!(Rating::parse(" Easy ").unwrap(), Rating::Easy);
        assert!(Rating::parse("5").is_err());
        assert!(Rating::parse("meh").is_err());
    }

    #[test]
    fn fsrs_orders_intervals_by_rating_for_a_new_card() {
        let s = FsrsScheduler::new(0.9).unwrap();
        let steps = s.next_steps(None, 0.0).unwrap();
        let iv: Vec<f64> = steps.iter().map(|s| s.interval_days).collect();
        assert!(iv[0] < iv[1] && iv[1] < iv[2] && iv[2] < iv[3], "{iv:?}");
    }

    #[test]
    fn higher_retention_means_shorter_intervals() {
        let lenient = FsrsScheduler::new(0.8)
            .unwrap()
            .next_steps(None, 0.0)
            .unwrap();
        let strict = FsrsScheduler::new(0.95)
            .unwrap()
            .next_steps(None, 0.0)
            .unwrap();
        assert!(strict[2].interval_days < lenient[2].interval_days);
        assert!(FsrsScheduler::new(1.2).is_err());
    }

    #[test]
    fn interval_rounding_and_limits() {
        let o = DeckOptions::default();
        assert_eq!(
            schedule_interval(0.000_01, &o).1,
            60_000,
            "minimum 1 minute"
        );
        assert_eq!(schedule_interval(0.5, &o).1, 43_200_000, "sub-day kept");
        assert_eq!(schedule_interval(4.4, &o).0, 4.0, "days rounded");
        assert_eq!(schedule_interval(1e9, &o).0, 36_500.0, "capped");
    }

    #[test]
    fn state_transitions() {
        assert_eq!(
            next_card_state("new", Rating::Good, 0.2),
            ("learning", false)
        );
        assert_eq!(next_card_state("new", Rating::Easy, 5.0), ("review", false));
        assert_eq!(
            next_card_state("review", Rating::Again, 0.01),
            ("relearning", true)
        );
        assert_eq!(
            next_card_state("relearning", Rating::Good, 0.5),
            ("relearning", false)
        );
        assert_eq!(
            next_card_state("relearning", Rating::Good, 2.0),
            ("review", false)
        );
        assert_eq!(
            next_card_state("learning", Rating::Again, 0.01),
            ("learning", false)
        );
    }
}
