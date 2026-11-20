-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0005: tags and taggings (WS-003; docs/specs/09-data-model.md §3 Organisation).
-- Hierarchy is by path prefix (`bio::cells` is under `bio`); parents need no row of their own.

CREATE TABLE tags (
    id            BLOB PRIMARY KEY,
    workspace_id  BLOB NOT NULL REFERENCES workspaces (id),
    path          TEXT NOT NULL,
    -- Case-folded path: tags match regardless of case (like Anki); the first spelling stays.
    path_key      TEXT NOT NULL,
    color         TEXT,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX tags_live_path ON tags (workspace_id, path_key) WHERE deleted_at IS NULL;

-- Polymorphic join: item_kind is an entity name (`fact` for now; notes, tasks, exams and
-- resources later). Checked in the core, not by a CHECK, so new kinds need no table rebuild.
CREATE TABLE taggings (
    id            BLOB PRIMARY KEY,
    tag_id        BLOB NOT NULL REFERENCES tags (id),
    item_kind     TEXT NOT NULL,
    item_id       BLOB NOT NULL,
    created_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX taggings_live ON taggings (tag_id, item_kind, item_id) WHERE deleted_at IS NULL;
CREATE INDEX taggings_item ON taggings (item_kind, item_id) WHERE deleted_at IS NULL;
