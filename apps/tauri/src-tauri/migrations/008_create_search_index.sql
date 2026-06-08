CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(title, content, content_rowid='rowid', content='notes');
