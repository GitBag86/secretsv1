import { invoke } from "@tauri-apps/api/core";
import type { User, Note, Todo, CalendarEvent, Notebook } from "@/types";

export const api = {
  auth: {
    register: (email: string, password: string, name?: string) =>
      invoke<{ user: User; token: string }>("register", { email, password, name }),
    login: (email: string, password: string) =>
      invoke<{ user: User; token: string }>("login", { email, password }),
    unlock: (password: string) =>
      invoke<{ success: boolean }>("unlock_database", { password }),
    logout: () => invoke<void>("logout"),
    lock: () => invoke<void>("lock_database"),
    checkSession: () => invoke<{ valid: boolean; unlocked_at?: number; elapsed_seconds?: number; timeout_minutes?: number }>("check_session"),
    getSessionTimeout: () => invoke<number>("get_session_timeout"),
    setSessionTimeout: (minutes: number) => invoke<void>("set_session_timeout", { minutes }),
    refreshSession: () => invoke<{ refreshed_at: number }>("refresh_session"),
  },
  notes: {
    list: () => invoke<Note[]>("list_notes"),
    get: (id: string) => invoke<Note>("get_note", { id }),
    create: (data: { title: string; content: string; notebook_id?: string }) =>
      invoke<Note>("create_note", data),
    update: (id: string, data: Partial<Note>) =>
      invoke<Note>("update_note", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_note", { id }),
    search: (query: string) => invoke<Note[]>("search_notes", { query }),
  },
  todos: {
    list: () => invoke<Todo[]>("list_todos"),
    create: (data: { title: string; description?: string; priority?: string; due_date?: number }) =>
      invoke<Todo>("create_todo", data),
    update: (id: string, data: Partial<Todo>) =>
      invoke<Todo>("update_todo", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_todo", { id }),
    bulkUpdate: (ids: string[], is_completed: boolean) =>
      invoke<void>("bulk_update_todos", { ids, is_completed }),
    bulkDelete: (ids: string[]) =>
      invoke<void>("bulk_delete_todos", { ids }),
  },
  calendar: {
    list: () => invoke<CalendarEvent[]>("list_calendar_events"),
    create: (data: { title: string; start_time: number; end_time: number; description?: string; all_day?: boolean; color?: string; rrule?: string }) =>
      invoke<CalendarEvent>("create_calendar_event", data),
    update: (id: string, data: Partial<CalendarEvent>) =>
      invoke<CalendarEvent>("update_calendar_event", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_calendar_event", { id }),
  },
  encryption: {
    setMasterPassword: (password: string) =>
      invoke<{ salt: string }>("set_master_password", { password }),
    getSalt: () => invoke<string | null>("get_encryption_salt"),
    rotate: (currentPassword: string, newPassword: string) =>
      invoke<{ rotated: boolean; notes: number; todos: number; events: number }>("rotate_encryption_key", { currentPassword, newPassword }),
  },
  notebooks: {
    list: () => invoke<Notebook[]>("list_notebooks"),
    create: (data: { name: string; color?: string }) =>
      invoke<Notebook>("create_notebook", data),
    update: (id: string, data: Partial<Notebook>) =>
      invoke<Notebook>("update_notebook", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_notebook", { id }),
  },
  sync: {
    push: () => invoke<{ synced: number }>("sync_push"),
    pull: () => invoke<{ synced: number }>("sync_pull"),
    status: () => invoke<{ pending: number; last_sync: number | null }>("sync_status"),
  },
};
