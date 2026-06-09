CREATE TABLE IF NOT EXISTS encryption_keys (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  key_type TEXT NOT NULL,
  encrypted_key BLOB NOT NULL,
  salt BLOB NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
