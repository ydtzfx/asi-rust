-- Migration tracking table — records which migrations have been applied.
CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    applied_at INTEGER NOT NULL DEFAULT (unixepoch())
);

-- Seed: record the initial schema migration.
INSERT OR IGNORE INTO _migrations (name) VALUES ('20260627000001_init');
