export interface User {
  id: string;
  email: string;
  name?: string;
  created_at: number;
  updated_at: number;
}

export interface Note {
  id: string;
  user_id: string;
  notebook_id?: string;
  title: string;
  content: string;
  word_count: number;
  reading_time: number;
  is_pinned: boolean;
  is_archived: boolean;
  created_at: number;
  updated_at: number;
}

export interface Todo {
  id: string;
  user_id: string;
  title: string;
  description?: string;
  is_completed: boolean;
  priority: "low" | "medium" | "high";
  due_date?: number;
  created_at: number;
  updated_at: number;
}

export interface CalendarEvent {
  id: string;
  user_id: string;
  title: string;
  description?: string;
  start_time: number;
  end_time: number;
  all_day: boolean;
  color: string;
  rrule?: string;
  parent_event_id?: string;
  created_at: number;
  updated_at: number;
}

export interface Notebook {
  id: string;
  user_id: string;
  name: string;
  color: string;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface Tag {
  id: string;
  user_id: string;
  name: string;
  color: string;
  created_at: number;
}

export interface RecurringTodo {
  id: string;
  todo_id: string;
  recurrence_rule: string;
  next_due_date: number;
  created_at: number;
}

export interface Attachment {
  id: string;
  user_id: string;
  note_id: string;
  filename: string;
  mime_type: string;
  size: number;
  storage_path: string;
  encrypted: boolean;
  created_at: number;
}

export interface Template {
  id: string;
  user_id: string;
  name: string;
  content: string;
  created_at: number;
  updated_at: number;
}

export interface UnifiedSearchItem {
  id: string;
  title: string;
  snippet: string;
  entity_type: "note" | "todo" | "event";
  url: string;
  subtitle: string;
}
