CREATE TABLE IF NOT EXISTS calendar_events (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL REFERENCES users(id),
  title TEXT NOT NULL,
  description TEXT,
  start_time INTEGER NOT NULL,
  end_time INTEGER NOT NULL,
  all_day INTEGER DEFAULT 0,
  color TEXT DEFAULT '#3b82f6',
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE INDEX IF NOT EXISTS idx_events_user ON calendar_events(user_id);
CREATE INDEX IF NOT EXISTS idx_events_time ON calendar_events(start_time, end_time);
