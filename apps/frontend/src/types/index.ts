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
