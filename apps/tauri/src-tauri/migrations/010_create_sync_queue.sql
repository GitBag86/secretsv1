CREATE TABLE IF NOT EXISTS sync_queue (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  user_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  operation TEXT NOT NULL CHECK(operation IN ('create','update','delete')),
  payload TEXT,
  vector_clock TEXT,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  synced INTEGER DEFAULT 0
);
CREATE INDEX IF NOT EXISTS idx_sync_queue_user ON sync_queue(user_id, synced);
