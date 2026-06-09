CREATE TABLE IF NOT EXISTS recurring_todos (
  id TEXT PRIMARY KEY,
  todo_id TEXT NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
  recurrence_rule TEXT NOT NULL,
  next_due_date INTEGER NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
