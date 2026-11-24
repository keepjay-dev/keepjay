// SPDX-License-Identifier: AGPL-3.0-or-later
//! "Today" summary for `keepjay status` and the Today widget (DASH `today`, J3).

use jiff::tz::TimeZone;
use rusqlite::params;
use serde::Serialize;

use super::DueCounts;
use crate::Core;
use crate::domain::Id;
use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Today {
    pub workspace: String,
    /// The study day as a local date, `YYYY-MM-DD` (it starts at 04:00).
    pub study_day: String,
    /// Short local label, e.g. `Thu 19 Nov`.
    pub label: String,
    pub due: DueCounts,
    /// Answers given this study day.
    pub reviewed: u64,
    /// Time spent answering this study day (ms).
    pub answer_time_ms: i64,
    /// When the next learning card is due (UTC ms), if any.
    pub next_learning_due: Option<i64>,
    /// Cards whose anchored code changed since they were written (SRC-006).
    pub stale: u64,
}

impl Core {
    /// Today's summary for a workspace, in the system time zone.
    pub fn today(&self, workspace: Id, now_ms: i64) -> Result<Today> {
        self.today_in(&TimeZone::system(), workspace, now_ms)
    }

    pub(crate) fn today_in(&self, tz: &TimeZone, workspace: Id, now_ms: i64) -> Result<Today> {
        let err = |e: jiff::Error| Error::Unexpected(format!("time error: {e}"));
        let day = super::study_day_in(tz, now_ms, super::DAY_STARTS_AT_HOUR)?;
        let start = jiff::Timestamp::from_millisecond(day.start_ms)
            .map_err(err)?
            .to_zoned_datetime(tz.clone());
        let now = jiff::Timestamp::from_millisecond(now_ms)
            .map_err(err)?
            .to_zoned_datetime(tz.clone());
        let name: String = self.conn.query_row(
            "SELECT name FROM workspaces WHERE id = ?1",
            [workspace.as_bytes()],
            |r| r.get(0),
        )?;
        let (reviewed, answer_time_ms): (u64, i64) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(rl.answer_ms), 0)
             FROM live_review_logs rl
             JOIN cards c ON c.id = rl.card_id
             JOIN decks d ON d.id = c.deck_id
             WHERE d.workspace_id = ?1 AND rl.reviewed_at >= ?2 AND rl.reviewed_at < ?3",
            params![workspace.as_bytes(), day.start_ms, day.end_ms],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let stale: u64 = self.conn.query_row(
            "SELECT COUNT(*)
             FROM anchors a
             JOIN facts f ON f.id = a.fact_id
             JOIN sources s ON s.id = a.source_id
             WHERE f.workspace_id = ?1 AND a.stale_since IS NOT NULL
               AND a.deleted_at IS NULL AND f.deleted_at IS NULL AND s.deleted_at IS NULL",
            [workspace.as_bytes()],
            |r| r.get(0),
        )?;
        Ok(Today {
            workspace: name,
            study_day: start.strftime("%Y-%m-%d").to_string(),
            label: now.strftime("%a %-d %b").to_string(),
            due: self.queue_counts(workspace, None, now_ms)?,
            reviewed,
            answer_time_ms,
            next_learning_due: self.next_learning_due(workspace, None)?,
            stale,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use jiff::tz::TimeZone;

    use crate::{CardType, Core, NewFact, Paths, Rating};

    #[test]
    fn counts_answers_in_the_current_study_day_only() {
        let dir = tempfile::tempdir().unwrap();
        let mut core = Core::open(Paths::new(dir.path(), dir.path())).unwrap();
        core.init("Uni").unwrap();
        let ws = core.resolve_workspace(None).unwrap();
        core.create_deck(ws, "Bio").unwrap();
        let mut ids = Vec::new();
        for q in ["a", "b", "c"] {
            let fact = core
                .add_fact(
                    ws,
                    NewFact {
                        deck: "Bio".into(),
                        card_type: CardType::Basic,
                        fields: BTreeMap::from([("Front".to_string(), q.to_string())]),
                        allow_duplicate: false,
                        tags: Vec::new(),
                    },
                )
                .unwrap();
            ids.push(fact.cards[0].id.parse::<crate::Id>().unwrap());
        }
        let utc = TimeZone::UTC;
        let t = 1_763_550_000_000; // 2025-11-19T11:00:00Z
        let yesterday = t - 86_400_000;
        core.answer_card(ids[0], Rating::Good, 4_000, yesterday)
            .unwrap();
        core.answer_card(ids[1], Rating::Good, 3_000, t).unwrap();
        core.answer_card(ids[2], Rating::Again, 5_000, t).unwrap();

        let today = core.today_in(&utc, ws, t).unwrap();
        assert_eq!(today.workspace, "Uni");
        assert_eq!(today.study_day, "2025-11-19");
        assert_eq!(today.label, "Wed 19 Nov");
        assert_eq!(today.reviewed, 2, "yesterday's answer is excluded");
        assert_eq!(today.answer_time_ms, 8_000);
        assert_eq!(today.due.new, 0);
        assert_eq!(today.stale, 0);
        assert!(
            today.next_learning_due.is_some(),
            "the Again card is relearning"
        );
    }
}
