CREATE TABLE IF NOT EXISTS device_info (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  name TEXT NOT NULL,
  platform TEXT NOT NULL,
  last_sync_at INTEGER,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
