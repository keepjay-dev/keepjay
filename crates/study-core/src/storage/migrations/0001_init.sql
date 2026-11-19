-- SPDX-License-Identifier: AGPL-3.0-or-later
-- 0001: foundations (docs/specs/09-data-model.md §1, §3). Released migrations are immutable.

-- Key/value metadata about this database.
CREATE TABLE schema_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;

-- Installations that have opened this database (this device's id lives in device.toml).
CREATE TABLE devices (
    id           BLOB PRIMARY KEY,
    name         TEXT NOT NULL,
    platform     TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL
) STRICT;

-- WS-001. Carries the sync columns required by D-009.
CREATE TABLE workspaces (
    id            BLOB PRIMARY KEY,
    name          TEXT NOT NULL,
    icon          TEXT,
    color         TEXT,
    sort          INTEGER NOT NULL DEFAULT 0,
    archived_at   INTEGER,
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    deleted_at    INTEGER,
    hlc           TEXT NOT NULL,
    origin_device BLOB NOT NULL
) STRICT;

CREATE UNIQUE INDEX workspaces_live_name ON workspaces (name) WHERE deleted_at IS NULL;

-- Append-only change log: live refresh across processes (08 §8) and future sync (08 §9).
CREATE TABLE oplog (
    seq       INTEGER PRIMARY KEY AUTOINCREMENT,
    hlc       TEXT NOT NULL,
    device_id BLOB NOT NULL,
    entity    TEXT NOT NULL,
    entity_id BLOB NOT NULL,
    op        TEXT NOT NULL CHECK (op IN ('insert', 'update', 'delete')),
    fields    TEXT NOT NULL,
    synced_at INTEGER
) STRICT;

INSERT INTO schema_meta (key, value) VALUES ('created_by', 'addisco');
