-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0004: where a fact came from (IO-001), e.g. `anki:<note guid>`, so re-importing the same
-- package skips notes that are already here.

ALTER TABLE facts ADD COLUMN source TEXT;

CREATE INDEX facts_source ON facts (workspace_id, source) WHERE source IS NOT NULL;
