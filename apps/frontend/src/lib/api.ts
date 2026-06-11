import { invoke } from "@tauri-apps/api/core";
import type { User, Note, Todo, CalendarEvent, Notebook, Tag, Attachment, RecurringTodo, Template, UnifiedSearchItem } from "@/types";

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
      invoke<{ rotated: boolean; notes: number; todos: number; events: number; attachments: number }>("rotate_encryption_key", { currentPassword, newPassword }),
  },
  notebooks: {
    list: () => invoke<Notebook[]>("list_notebooks"),
    create: (data: { name: string; color?: string }) =>
      invoke<Notebook>("create_notebook", data),
    update: (id: string, data: Partial<Notebook>) =>
      invoke<Notebook>("update_notebook", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_notebook", { id }),
  },
  tags: {
    list: () => invoke<Tag[]>("list_tags"),
    create: (data: { name: string; color?: string }) =>
      invoke<Tag>("create_tag", data),
    update: (id: string, data: { name?: string; color?: string }) =>
      invoke<Tag>("update_tag", { id, ...data }),
    delete: (id: string) => invoke<void>("delete_tag", { id }),
    getNoteTags: (noteId: string) => invoke<Tag[]>("get_note_tags", { noteId }),
    setNoteTags: (noteId: string, tagIds: string[]) =>
      invoke<void>("set_note_tags", { noteId, tagIds }),
    getNotesWithTag: (tagId: string) => invoke<string[]>("get_notes_with_tag", { tagId }),
    listAllNoteTags: () => invoke<{ note_id: string; tag_id: string }[]>("list_all_note_tags"),
    getTodoTags: (todoId: string) => invoke<Tag[]>("get_todo_tags", { todoId }),
    setTodoTags: (todoId: string, tagIds: string[]) =>
      invoke<void>("set_todo_tags", { todoId, tagIds }),
    getTodosWithTag: (tagId: string) => invoke<string[]>("get_todos_with_tag", { tagId }),
    listAllTodoTags: () => invoke<{ todo_id: string; tag_id: string }[]>("list_all_todo_tags"),
  },
  attachments: {
    list: (noteId: string) => invoke<Attachment[]>("list_note_attachments", { noteId }),
    attach: (noteId: string, filename: string, mimeType: string, dataBase64: string) =>
      invoke<Attachment>("attach_file", { noteId, filename, mimeType, dataBase64 }),
    delete: (id: string) => invoke<void>("delete_attachment", { id }),
    open: (id: string) => invoke<void>("open_attachment", { id }),
    getAllCounts: () => invoke<{ note_id: string; count: number }[]>("get_all_attachment_counts"),
  },
  recurringTodos: {
    set: (todoId: string, recurrenceRule: string) =>
      invoke<RecurringTodo>("set_recurrence", { todoId, recurrenceRule }),
    remove: (todoId: string) => invoke<void>("remove_recurrence", { todoId }),
    list: () => invoke<RecurringTodo[]>("list_recurring_todos"),
  },
  templates: {
    list: () => invoke<Template[]>("list_templates"),
    create: (data: { name: string; content: string }) =>
      invoke<Template>("create_template", data),
    delete: (id: string) => invoke<void>("delete_template", { id }),
    createNoteFrom: (templateId: string, title?: string, notebookId?: string) =>
      invoke<Note>("create_note_from_template", { templateId, title, notebookId }),
  },
  trash: {
    listNotes: () => invoke<Note[]>("list_archived_notes"),
    listTodos: () => invoke<Todo[]>("list_archived_todos"),
    restoreNote: (id: string) => invoke<void>("restore_note", { id }),
    restoreTodo: (id: string) => invoke<void>("restore_todo", { id }),
    permanentlyDeleteNote: (id: string) => invoke<void>("permanently_delete_note", { id }),
    permanentlyDeleteTodo: (id: string) => invoke<void>("permanently_delete_todo", { id }),
    archiveNote: (id: string) => invoke<void>("archive_note", { id }),
    archiveTodo: (id: string) => invoke<void>("archive_todo", { id }),
  },
  search: {
    unified: (query: string) => invoke<UnifiedSearchItem[]>("unified_search", { query }),
  },
  data: {
    exportData: () => invoke<string>("export_data"),
    importData: (data: string) =>
      invoke<{ notes: number; todos: number; events: number; notebooks: number; tags: number; note_tags: number; todo_tags: number; errors: string[] }>("import_data", { data }),
  },
  sync: {
    push: () => invoke<{ pushed: number; total: number; remaining: number; errors: string[] }>("sync_push"),
    pull: () => invoke<{ pulled: number; pending: number; errors: string[] }>("sync_pull"),
    status: () => invoke<{ pending: number; last_sync: number | null; configured: boolean }>("sync_status"),
    configure: (url: string, key: string) =>
      invoke<{ configured: boolean; connection_ok: boolean }>("configure_sync", { url, key }),
    getConfig: () =>
      invoke<{ url: string | null; has_key: boolean }>("get_sync_config"),
  },
};
