ALTER TABLE calendar_events ADD COLUMN rrule TEXT;
ALTER TABLE calendar_events ADD COLUMN parent_event_id TEXT REFERENCES calendar_events(id) ON DELETE CASCADE;
CREATE INDEX IF NOT EXISTS idx_events_parent ON calendar_events(parent_event_id);
