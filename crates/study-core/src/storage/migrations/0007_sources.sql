-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0007: sources and anchors (SRC-001, SRC-004; docs/specs/13-code-sources.md, 09 §3).
-- A source is a folder or Git repository that is scanned for cards. An anchor links a
-- fact to the code it's about. Card content stays in the user's files (D-029).

CREATE TABLE sources (
    id            BLOB PRIMARY KEY,
    workspace_id  BLOB NOT NULL REFERENCES workspaces (id),
    -- Unique per workspace; how the CLI addresses a source (`addisco scan <name>`).
    name          TEXT NOT NULL,
    -- `repo` or `folder` for now (`problems`, `deck_repo` later). Checked in the core, not by
    -- a CHECK, so new kinds need no table rebuild.
    kind          TEXT NOT NULL,
    -- Absolute path on this device.
    path          TEXT NOT NULL,
    -- Root commit hash for Git repositories: stable across clones and remotes.
    repo          TEXT,
    deck_id       BLOB NOT NULL REFERENCES decks (id),
    tags          TEXT NOT NULL DEFAULT '[]',
    write_ids     INTEGER NOT NULL DEFAULT 1,
    settings      TEXT NOT NULL DEFAULT '{}',
    last_scan_at  INTEGER,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX sources_live_name ON sources (workspace_id, name) WHERE deleted_at IS NULL;
CREATE UNIQUE INDEX sources_live_path ON sources (workspace_id, path) WHERE deleted_at IS NULL;

CREATE TABLE anchors (
    id            BLOB PRIMARY KEY,
    fact_id       BLOB NOT NULL REFERENCES facts (id),
    source_id     BLOB NOT NULL REFERENCES sources (id),
    -- The source's `repo`, or its id for plain folders.
    repo          TEXT NOT NULL,
    -- Relative to the source root, with `/` separators on every OS.
    path          TEXT NOT NULL,
    line_start    INTEGER NOT NULL,
    line_end      INTEGER NOT NULL,
    symbol        TEXT,
    code_hash     TEXT NOT NULL,
    stale_since   INTEGER,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX anchors_live_fact ON anchors (fact_id) WHERE deleted_at IS NULL;
CREATE INDEX anchors_source ON anchors (source_id) WHERE deleted_at IS NULL;
CREATE INDEX anchors_stale ON anchors (source_id) WHERE stale_since IS NOT NULL AND deleted_at IS NULL;
