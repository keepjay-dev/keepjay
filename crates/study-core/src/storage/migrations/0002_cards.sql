-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0002: decks, facts and cards (SRS-001, SRS-002; docs/specs/09-data-model.md §3 Flashcards).
-- Card scheduling columns are a cache derived from review logs (SRS-006, added in 0003).

CREATE TABLE decks (
    id            BLOB PRIMARY KEY,
    workspace_id  BLOB NOT NULL REFERENCES workspaces (id),
    parent_id     BLOB REFERENCES decks (id),
    name          TEXT NOT NULL,
    options_id    TEXT NOT NULL DEFAULT 'builtin:deck-options-default',
    sort          INTEGER NOT NULL DEFAULT 0,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

-- Sibling deck names are unique among live decks (root decks use an empty-blob sentinel).
CREATE UNIQUE INDEX decks_live_name
    ON decks (workspace_id, COALESCE(parent_id, x''), name) WHERE deleted_at IS NULL;

-- A fact is the set of field values; cards are generated from it by its card type.
CREATE TABLE facts (
    id            BLOB PRIMARY KEY,
    workspace_id  BLOB NOT NULL REFERENCES workspaces (id),
    deck_id       BLOB NOT NULL REFERENCES decks (id),
    card_type     TEXT NOT NULL,
    fields        TEXT NOT NULL,
    sort_field    TEXT NOT NULL,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

-- Duplicate detection (SRS-007): same first field, card type and deck.
CREATE INDEX facts_duplicate_key ON facts (deck_id, card_type, sort_field) WHERE deleted_at IS NULL;

CREATE TABLE cards (
    id             BLOB PRIMARY KEY,
    fact_id        BLOB NOT NULL REFERENCES facts (id),
    ord            INTEGER NOT NULL,
    deck_id        BLOB NOT NULL REFERENCES decks (id),
    state          TEXT NOT NULL DEFAULT 'new'
                   CHECK (state IN ('new', 'learning', 'review', 'relearning')),
    due            INTEGER,
    stability      REAL,
    difficulty     REAL,
    reps           INTEGER NOT NULL DEFAULT 0,
    lapses         INTEGER NOT NULL DEFAULT 0,
    last_review_at INTEGER,
    suspended      INTEGER NOT NULL DEFAULT 0,
    buried_until   INTEGER,
    flag           INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    deleted_at     INTEGER,
    hlc            TEXT NOT NULL,
    origin_device  BLOB NOT NULL,
    UNIQUE (fact_id, ord)
) STRICT;

CREATE INDEX cards_deck_state_due ON cards (deck_id, state, due) WHERE deleted_at IS NULL;
