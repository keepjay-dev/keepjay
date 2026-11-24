-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0008: undone answers (SRS-005, D-041). The review log stays append-only (08 §9): undoing
-- an answer appends a row here, and readers use `live_review_logs`, which leaves undone
-- answers out. Card scheduling state is then replayed from the live answers.

CREATE TABLE review_undos (
    review_log_id BLOB PRIMARY KEY REFERENCES review_logs (id),
    undone_at     INTEGER NOT NULL,
    hlc           TEXT NOT NULL,
    device_id     BLOB NOT NULL
) STRICT;

CREATE VIEW live_review_logs AS
    SELECT * FROM review_logs rl
     WHERE NOT EXISTS (SELECT 1 FROM review_undos u WHERE u.review_log_id = rl.id);
