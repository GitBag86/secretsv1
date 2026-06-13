-- Add note_id foreign key to todos for linking todos to notes
ALTER TABLE todos ADD COLUMN note_id TEXT REFERENCES notes(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_todos_note ON todos(note_id);