// SPDX-License-Identifier: AGPL-3.0-or-later
//! The review queue and the study day (SRS-004, SRS-005; docs/specs/09 §1 "Study day").
//!
//! Order: learning/relearning cards due now, then reviews due today, then new cards within
//! the daily new-card limit.
//!
//! Learning cards are never shown *before* they're due. FSRS-7 models short-term memory
//! from real elapsed time, so reviewing early adds almost nothing and the card would loop
//! (found in M0.4 testing). Instead, shells wait for the next learning card when only
//! near-future ones remain (see [`Core::next_learning_due`] and [`LEARNING_WAIT_MS`]).

use std::collections::BTreeMap;

use jiff::ToSpan;
use jiff::tz::TimeZone;
use rusqlite::{OptionalExtension, params};
use serde::Serialize;

use super::{DeckOptions, DueCounts};
use crate::Core;
use crate::cards::{CARD_COLUMNS, Card, CardType, card_from_row, render};
use crate::domain::Id;
use crate::error::{Error, Result};

/// If the next learning card is due within this window, a session waits for it rather than
/// ending.
pub const LEARNING_WAIT_MS: i64 = 20 * 60_000;

/// Default `srs.day_starts_at` (03 §5): late-night study counts for the previous day.
pub const DAY_STARTS_AT_HOUR: i8 = 4;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub struct StudyDay {
    pub start_ms: i64,
    pub end_ms: i64,
}

/// The study day containing `now_ms` in `tz`, starting at `starts_at_hour` local time.
pub fn study_day_in(tz: &TimeZone, now_ms: i64, starts_at_hour: i8) -> Result<StudyDay> {
    let err = |e: jiff::Error| Error::Unexpected(format!("time zone error: {e}"));
    let now = jiff::Timestamp::from_millisecond(now_ms)
        .map_err(err)?
        .to_zoned_datetime(tz.clone());
    let mut start = now
        .with()
        .hour(starts_at_hour)
        .minute(0)
        .second(0)
        .subsec_nanosecond(0)
        .build()
        .map_err(err)?;
    if start > now {
        start = start.checked_sub(1.day()).map_err(err)?;
    }
    let end = start.checked_add(1.day()).map_err(err)?;
    Ok(StudyDay {
        start_ms: start.timestamp().as_millisecond(),
        end_ms: end.timestamp().as_millisecond(),
    })
}

/// Why a card is in the queue.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueueKind {
    Learning,
    Review,
    New,
}

/// A card ready to show: the card plus its rendered question and answer.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QueuedCard {
    pub card: Card,
    pub kind: QueueKind,
    pub deck_path: String,
    pub question: String,
    pub answer: String,
}

impl Core {
    /// The current study day in the system time zone.
    pub fn study_day(&self, now_ms: i64) -> Result<StudyDay> {
        study_day_in(&TimeZone::system(), now_ms, DAY_STARTS_AT_HOUR)
    }

    /// SQL fragment defining `scope(id)`: the deck and its subdecks, or every deck in the
    /// workspace. Parameter ?1 is the workspace, ?2 the root deck (or NULL).
    const SCOPE_CTE: &'static str = "WITH RECURSIVE scope(id) AS (
            SELECT id FROM decks
             WHERE workspace_id = ?1 AND deleted_at IS NULL AND (?2 IS NULL OR id = ?2)
            UNION
            SELECT d.id FROM decks d JOIN scope s ON d.parent_id = s.id
             WHERE d.deleted_at IS NULL
        )";

    fn scope_root(&self, workspace: Id, deck: Option<&str>) -> Result<Option<Vec<u8>>> {
        match deck {
            None => Ok(None),
            Some(path) => self
                .find_deck(workspace, path)?
                .map(|id| Some(id.as_bytes().to_vec()))
                .ok_or_else(|| Error::NotFound(format!("no deck '{path}'"))),
        }
    }

    /// New cards still allowed today in this scope (daily limit, SRS-004).
    fn new_allowance(&self, workspace: Id, root: &Option<Vec<u8>>, day: StudyDay) -> Result<u64> {
        let introduced: u64 = self.conn.query_row(
            &format!(
                "{} SELECT COUNT(*) FROM review_logs rl JOIN cards c ON c.id = rl.card_id
                 WHERE c.deck_id IN (SELECT id FROM scope)
                   AND rl.state_before = 'new' AND rl.reviewed_at >= ?3",
                Self::SCOPE_CTE
            ),
            params![workspace.as_bytes(), root, day.start_ms],
            |r| r.get(0),
        )?;
        Ok((DeckOptions::default().new_per_day as u64).saturating_sub(introduced))
    }

    /// Counts for the queue header, scoped to a deck (and subdecks) or the whole workspace.
    pub fn queue_counts(
        &self,
        workspace: Id,
        deck: Option<&str>,
        now_ms: i64,
    ) -> Result<DueCounts> {
        let root = self.scope_root(workspace, deck)?;
        let day = self.study_day(now_ms)?;
        let (learning, review, new): (u64, u64, u64) = self.conn.query_row(
            &format!(
                "{} SELECT
                     COALESCE(SUM(state IN ('learning', 'relearning') AND due < ?3), 0),
                     COALESCE(SUM(state = 'review' AND due < ?3), 0),
                     COALESCE(SUM(state = 'new'), 0)
                 FROM cards WHERE deck_id IN (SELECT id FROM scope)
                   AND deleted_at IS NULL AND suspended = 0",
                Self::SCOPE_CTE
            ),
            params![workspace.as_bytes(), root, day.end_ms],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let allowance = self.new_allowance(workspace, &root, day)?;
        Ok(DueCounts {
            learning,
            review,
            new: new.min(allowance),
        })
    }

    /// The next card to study, or `None` when the queue is empty for now.
    pub fn next_card(
        &self,
        workspace: Id,
        deck: Option<&str>,
        now_ms: i64,
    ) -> Result<Option<QueuedCard>> {
        let root = self.scope_root(workspace, deck)?;
        let day = self.study_day(now_ms)?;
        let base = format!(
            "{} SELECT {CARD_COLUMNS} FROM cards
             WHERE deck_id IN (SELECT id FROM scope) AND deleted_at IS NULL AND suspended = 0",
            Self::SCOPE_CTE
        );
        // ?3 is the time bound for learning and review cards; new cards don't use it.
        let pick = |extra: &str, bound: Option<i64>| -> Result<Option<Card>> {
            let sql = format!("{base} {extra} LIMIT 1");
            let found = match bound {
                Some(b) => {
                    self.conn
                        .query_row(&sql, params![workspace.as_bytes(), root, b], card_from_row)
                }
                None => {
                    self.conn
                        .query_row(&sql, params![workspace.as_bytes(), root], card_from_row)
                }
            };
            Ok(found.optional()?)
        };
        let (card, kind) = if let Some(c) = pick(
            "AND state IN ('learning', 'relearning') AND due <= ?3 ORDER BY due",
            Some(now_ms),
        )? {
            (c, QueueKind::Learning)
        } else if let Some(c) = pick(
            "AND state = 'review' AND due < ?3 ORDER BY due",
            Some(day.end_ms),
        )? {
            (c, QueueKind::Review)
        } else if self.new_allowance(workspace, &root, day)? > 0
            && let Some(c) = pick("AND state = 'new' ORDER BY id", None)?
        {
            (c, QueueKind::New)
        } else {
            return Ok(None);
        };
        self.queued(card, kind).map(Some)
    }

    /// When the next learning/relearning card in scope becomes due, if any.
    pub fn next_learning_due(&self, workspace: Id, deck: Option<&str>) -> Result<Option<i64>> {
        let root = self.scope_root(workspace, deck)?;
        Ok(self.conn.query_row(
            &format!(
                "{} SELECT MIN(due) FROM cards WHERE deck_id IN (SELECT id FROM scope)
                   AND state IN ('learning', 'relearning') AND deleted_at IS NULL
                   AND suspended = 0",
                Self::SCOPE_CTE
            ),
            params![workspace.as_bytes(), root],
            |r| r.get(0),
        )?)
    }

    fn queued(&self, card: Card, kind: QueueKind) -> Result<QueuedCard> {
        let fact_id: Id = card.fact_id.parse()?;
        let (card_type, fields): (String, String) = self.conn.query_row(
            "SELECT card_type, fields FROM facts WHERE id = ?1",
            [fact_id.as_bytes()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let fields: BTreeMap<String, String> = serde_json::from_str(&fields)
            .map_err(|e| Error::Unexpected(format!("corrupt fact fields: {e}")))?;
        let (question, answer) = render::render(CardType::parse(&card_type)?, &fields, card.ord);
        let deck_id: Id = card.deck_id.parse()?;
        let workspace: Vec<u8> = self.conn.query_row(
            "SELECT workspace_id FROM decks WHERE id = ?1",
            [deck_id.as_bytes()],
            |r| r.get(0),
        )?;
        let deck_path = self
            .list_decks(Id::from_slice(&workspace)?)?
            .into_iter()
            .find(|d| d.id == card.deck_id)
            .map(|d| d.path)
            .unwrap_or_default();
        Ok(QueuedCard {
            card,
            kind,
            deck_path,
            question,
            answer,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AT_0300_UTC: i64 = 1_763_521_200_000; // 2025-11-19T03:00:00Z

    #[test]
    fn study_day_starts_at_four_local() {
        let utc = TimeZone::UTC;
        // 03:00 belongs to the previous study day (04:00 the day before → 04:00 today).
        let d = study_day_in(&utc, AT_0300_UTC, 4).unwrap();
        assert_eq!(d.end_ms, AT_0300_UTC + 3_600_000);
        assert_eq!(d.end_ms - d.start_ms, 86_400_000);
        // 05:00 starts a new study day at 04:00.
        let later = AT_0300_UTC + 2 * 3_600_000;
        let d2 = study_day_in(&utc, later, 4).unwrap();
        assert_eq!(d2.start_ms, d.end_ms);
    }

    #[test]
    fn study_day_respects_the_time_zone() {
        let tokyo = TimeZone::get("Asia/Tokyo").unwrap(); // UTC+9, no DST
        let d = study_day_in(&tokyo, AT_0300_UTC, 4).unwrap();
        // 03:00Z is 12:00 in Tokyo, so the day started at 04:00 JST = 19:00Z the day before.
        assert_eq!(d.start_ms, AT_0300_UTC - 8 * 3_600_000);
    }
}
