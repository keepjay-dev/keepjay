-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0006: media (D-026, IO-001; docs/specs/09-data-model.md §3 Resources).
-- `media` indexes the immutable, content-addressed files in the data dir's media/ folder.
-- Fields refer to media by name (`<img src="cell.png">`, `[sound:x.mp3]`), as in Anki, so
-- `media_names` maps each name to its content, per workspace.

CREATE TABLE media (
    hash       TEXT PRIMARY KEY, -- lowercase hex SHA-256 of the content
    ext        TEXT NOT NULL,    -- sanitised extension used on disk
    mime       TEXT,
    size       INTEGER NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE TABLE media_names (
    id            BLOB PRIMARY KEY,
    workspace_id  BLOB NOT NULL REFERENCES workspaces (id),
    name          TEXT NOT NULL,
    hash          TEXT NOT NULL REFERENCES media (hash),
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX media_names_live ON media_names (workspace_id, name) WHERE deleted_at IS NULL;
CREATE INDEX media_names_hash ON media_names (hash);
