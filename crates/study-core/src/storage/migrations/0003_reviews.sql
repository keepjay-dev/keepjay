-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0003: append-only review log (SRS-006; docs/specs/09-data-model.md §3) and the
-- FSRS-7 fast-stability trace on cards. Card scheduling columns are a cache that
-- can always be rebuilt by replaying review_logs.

ALTER TABLE cards ADD COLUMN stability_fast REAL;

CREATE TABLE review_logs (
    id                    BLOB PRIMARY KEY,
    card_id               BLOB NOT NULL REFERENCES cards (id),
    session_id            BLOB,
    reviewed_at           INTEGER NOT NULL,
    rating                INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 4),
    answer_ms             INTEGER NOT NULL,
    state_before          TEXT NOT NULL,
    stability_before      REAL,
    difficulty_before     REAL,
    stability_fast_before REAL,
    stability_after       REAL NOT NULL,
    difficulty_after      REAL NOT NULL,
    stability_fast_after  REAL NOT NULL,
    interval_days         REAL NOT NULL,
    due_after             INTEGER NOT NULL,
    scheduler             TEXT NOT NULL,
    hlc                   TEXT NOT NULL,
    device_id             BLOB NOT NULL
) STRICT;

CREATE INDEX review_logs_card_time ON review_logs (card_id, reviewed_at);
CREATE INDEX review_logs_time ON review_logs (reviewed_at);
