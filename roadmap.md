# KnowledgeBase — Roadmap & Audit Findings

> Generated from architectural review — June 9, 2026

This document captures all findings, organized by priority, from a comprehensive review of the KnowledgeBase application. Each item includes specific file references and should be actionable.

---

## Priority Key

| Priority | Meaning | Timeframe |
|----------|---------|-----------|
| 🔴 **P0** | Security vulnerability or build-breaking bug | Fix immediately |
| 🟠 **P1** | Architectural weakness or missing critical flow | Fix this week |
| 🟡 **P2** | Quality improvement or moderate feature gap | This sprint |
| 🟢 **P3** | Nice-to-have or long-term improvement | Backlog |

---

## 🔴 P0 — Critical

### 1. ✅ Fix password verification in `login()`

**Status:** FIXED

**File:** `apps/tauri/src-tauri/src/commands/auth.rs`

`login()` calls `verify_password()` but did not check the returned `bool`. Added explicit `if !valid { return Err(...) }` check.

Also fixed the same bug in `rotate_encryption_key()` in `apps/tauri/src-tauri/src/commands/encryption.rs`.

---

### 2. ✅ Database unlock now validates password

**Status:** FIXED

**File:** `apps/tauri/src-tauri/src/commands/auth.rs`, function `unlock_database()`

Previously `unlock_database()` derived a key from any password without verifying correctness. Now it first verifies the password against the stored Argon2id hash, and only derives the encryption key if the password is valid.

---

### 3. ✅ Session expiry now enforced server-side

**Status:** FIXED

**Files:**
- `apps/tauri/src-tauri/src/commands/helpers.rs` (new `require_valid_session()` function)
- `apps/tauri/src-tauri/src/commands/notes.rs`
- `apps/tauri/src-tauri/src/commands/todos.rs`
- `apps/tauri/src-tauri/src/commands/calendar.rs`
- `apps/tauri/src-tauri/src/commands/encryption.rs`

Added `require_valid_session()` helper that checks both `is_locked()` and session expiry (from `session_unlocked_at` + `session_timeout`). If the session has expired, it **clears the encryption key** to enforce the lock server-side. Applied to all mutating commands (create/update for notes, todos, calendar events + encryption operations).

---

### 4. Sync payloads contain plaintext content

**Files:**
- `apps/tauri/src-tauri/src/commands/notes.rs`
- `apps/tauri/src-tauri/src/commands/todos.rs`
- `apps/tauri/src-tauri/src/commands/calendar.rs`
- `apps/tauri/src-tauri/src/commands/sync.rs`

Sync queue payloads are constructed from **plaintext** values (title, content, description), not encrypted content. This means:
- Remote sync is **not end-to-end encrypted**
- Local `sync_queue` table may contain plaintext sensitive data
- This undermines the stated encrypted-at-rest model

**Fix:** Decide explicitly between two models:
- **Model A** — "Encrypted local, plaintext cloud" (document clearly)
- **Model B** — "Zero-knowledge sync" (upload encrypted envelopes)

---

### 5. ✅ Frontend master-password setup flow

**Status:** FIXED

**Files:**
- `apps/frontend/src/app/unlock/page.tsx`
- `apps/frontend/src/hooks/useAuth.ts`

Added first-run setup flow to the unlock page:
- On mount, checks if a master password salt exists via `api.encryption.getSalt()`
- **No salt** → shows "Set Master Password" form with password strength indicator, confirm password with mismatch validation, and a security warning
- **Salt exists** → shows the existing "Unlock Database" form (unchanged)
- Added `setupMasterPassword()` action to `useAuth` store that calls `setMasterPassword`, refreshes the session, and sets `isUnlocked: true`
- Shows a loading spinner while checking the database
- After setup/unlock, redirects to `/`

---

### 6. ✅ Build-breaking bugs (all fixed)

#### 6a. ✅ Malformed import in notes page
**File:** `apps/frontend/src/app/notes/page.tsx`
- Missing closing brace `}` in `import { TemplatePicker from...` — added. Was causing TS1005 parse error.

#### 6b. ✅ Temporal dead zone in todos page
**File:** `apps/frontend/src/app/todos/page.tsx`
- Moved `filteredRef` and keyboard handler `useEffect` after `const filtered = useMemo(...)` declaration. No hook order violation.

#### 6c. ✅ Calendar timestamps treated as ms
**File:** `apps/frontend/src/app/calendar/page.tsx`
- Changed `new Date(e.start_time)` to `new Date(e.start_time * 1000)` (and for end_time, rrule dtstart). Rust stores Unix seconds, JS Date() expects ms.

#### 6d. ✅ Rust privacy/field access
**File:** `apps/tauri/src-tauri/src/commands/encryption.rs`, `apps/tauri/src-tauri/src/crypto/manager.rs`
- Removed direct `enc.key.lock().await` access to private field. Added `get_key_copy()` public method to `EncryptionManager`.

---

## 🟠 P1 — High Priority

### 7. App does not bootstrap auth state

**Files:**
- `apps/frontend/src/hooks/useAuth.ts`
- `apps/frontend/src/app/layout.tsx`
- `apps/frontend/src/app/providers.tsx`

`useAuth` is purely in-memory. `isHydrated` exists but is unused. On refresh/restart, the frontend state resets even if the backend DB session may still be active. There is no startup bootstrap that checks the current backend session or unlock state.

**Fix:** Add app bootstrap logic:
1. On load, check current user, encryption status, session validity
2. Hydrate auth state from backend reality, not local optimism
3. Route unauthenticated users appropriately

---

### 8. Auth state and encryption state are decoupled

**Files:**
- `apps/frontend/src/app/login/page.tsx`
- `apps/frontend/src/app/register/page.tsx`

`login` and `register` set `isUnlocked: true` in Zustand but do **not** unlock the encryption manager key in Rust. The UI believes the app is unlocked when Rust may still reject note/todo/event creation.

**Fix:** Ensure login/register also perform the backend unlock flow, or separate "logged in" from "database unlocked" in the state model.

---

### 9. No route guards

**Files:** All page components

Pages like `/notes`, `/todos`, `/calendar`, `/dashboard` are accessible without being logged in or having the database unlocked. This can expose decrypted placeholders or error states.

**Fix:** Add route-level guards that check auth/unlock state and redirect to login/unlock pages.

---

### 10. Search holds DB lock during decrypt

**File:** `apps/tauri/src-tauri/src/commands/archive.rs`

`unified_search()` keeps the DB connection guard alive while awaiting async decryption calls. Since `DbPool` is a single mutexed connection, this blocks the entire app.

**Fix:** Load all rows first, drop the DB lock, then decrypt and filter.

---

### 11. Key rotation scope is incomplete

**File:** `apps/tauri/src-tauri/src/commands/encryption.rs`

Key rotation re-encrypts notes, todos, and calendar events but does **not** cover:
- Attachments (stored encrypted but old key may still be needed)
- Sync queue payloads (if they contain encrypted data)
- Any other encrypted config rows in `app_settings`

**Fix:** Either extend rotation scope or explicitly document what is/isn't rotated. Add integration tests.

---

## 🟡 P2 — Medium Priority

### 12. Error handling is often silent

**Files:** Multiple frontend components

- `SearchPalette` catches and ignores search errors (`.catch(() => {})`)
- Attachment errors only log to `console.error`
- React Query has no global error handler
- Backend returns `Result<_, String>` with no structured error model

**Fix:** Introduce structured error types, add a global error handler/toast system, surface meaningful errors to users.

---

### 13. Giant page components

**Files:**
- `apps/frontend/src/app/notes/page.tsx` (~300 lines)
- `apps/frontend/src/app/todos/page.tsx` (~350 lines)
- `apps/frontend/src/app/settings/page.tsx` (~450 lines)

These pages own too much: data fetching, input state, business logic, modal management, attachment handling, tag management, etc.

**Fix:** Decompose into focused components:
- `NotesList`, `NoteEditorModal`, `NoteAttachmentsPanel`, `NoteTagsPanel`
- `SettingsSession`, `SettingsEncryption`, `SettingsSync`, `SettingsTags`, `SettingsTemplates`

---

### 14. Dashboard medium-priority count is wrong

**File:** `apps/frontend/src/app/page.tsx`

```ts
count: stats.activeTodos - stats.highPriority
```

This includes low-priority todos. It should subtract both high and low, or count medium directly.

---

### 15. Settings import invalidates wrong query key

**File:** `apps/frontend/src/app/settings/page.tsx`

After import, invalidates `["events"]` but calendar data uses `["calendar"]`.

---

### 16. Bulk todo operations bypass recurring/sync logic

**File:** `apps/tauri/src-tauri/src/commands/todos.rs`

`bulk_update_todos()` and `bulk_delete_todos()` do not handle recurring todo advancement or sync queue enqueuing.

---

### 17. Nav uses `<a>` instead of Next.js `<Link>`

**File:** `apps/frontend/src/components/nav-header.tsx`

Using `<a href>` causes full page reloads instead of client-side navigation.

---

### 18. RichTextEditor content sync

**File:** `apps/frontend/src/components/rich-text-editor.tsx`

`useEditor({ content })` only initializes content on mount. If the `content` prop changes (e.g., selecting a different note), the editor does not update.

**Fix:** Use `editor.commands.setContent(content)` in a `useEffect` when content prop changes.

---

### 19. TipTap CSS selectors

**File:** `apps/frontend/src/app/globals.css`

CSS rules target `.tiptap` class but `EditorContent` does not appear to use that class by default.

---

### 20. Undo/redo is not complete

**File:** `apps/frontend/src/hooks/useUndoStack.ts`

- Tags and attachments are not restored on undo
- IDs and timestamps may not be faithfully restored
- Redo/create-after-delete semantics can drift

---

### 21. Search results don't deep-link

**File:** `apps/tauri/src-tauri/src/commands/archive.rs`

Unified search returns `/notes`, `/todos`, `/calendar` — not item-specific routes. Users cannot navigate directly to the matching entity.

---

## 🟢 P3 — Long-term / Nice-to-have

### 22. Shared UI component library

No reusable UI primitives exist:
- Modal
- Form field
- Card
- Button variants
- Empty state
- Toast/error banners

### 23. Tag management duplication

Tag editing UI is replicated across notes page, todos page, and settings page. Extract into a shared component.

### 24. Search architecture for scale

Current decrypt+filter search works for small/medium datasets but will degrade with 1000+ notes. Options:
- Ephemeral in-memory search index after unlock
- Blind hashed term index
- In-memory FTS mirror while unlocked

### 25. Specialized summary endpoints

Replace full-list queries with lighter endpoints:
- `list_note_summaries` (id, title, snippet, word_count)
- `dashboard_summary`
- `tag_usage_stats`

### 26. Attachment performance

Attachments are base64-encoded in the browser before IPC. For large files:
- Large memory spike
- ~33% size inflation
- Uses full `invoke()` payload instead of streaming

**Fix:** Use Tauri native file APIs or a streaming/path-based approach.

### 27. Accessibility audit

- Search palette needs `role="dialog"`, focus trap, listbox semantics
- Icon buttons need ARIA labels
- TipTap toolbar needs richer a11y metadata
- Color-only tag controls may be inaccessible
- Visible focus states need audit

### 28. Keyboard shortcut discoverability

Shortcuts exist (Cmd+K, Ctrl+N, Ctrl+E, Ctrl+Z) but are not surfaced in the UI. No command palette documentation.

### 29. Reusable CRUD patterns in Rust

CRUD code is repeated across notes/todos/calendar commands. Extract generic helpers for common patterns:
- List with decryption
- Get with decryption
- Create with encryption + sync enqueue
- Update with encryption + sync enqueue
- Delete with sync enqueue

### 30. Integration test coverage

Tests are biased toward small utility correctness. Missing high-value tests:
- Login with wrong password
- Unlock with wrong password
- Session expiry enforcement
- Create/update while locked
- Sync queue confidentiality
- Key rotation end-to-end
- Import/export with encrypted data
- Recurring todo completion
- Migration integration tests

### 31. `handleEventDrop` uses `any` type

**File:** `apps/frontend/src/app/calendar/page.tsx`

The `handleEventDrop` callback uses `(info: any)`. Type it properly with `EventDropArg` from `@fullcalendar/core`.

### 32. No optimistic updates in React Query

Most mutations refresh by invalidating + refetching. For toggle-like actions (e.g., marking a todo complete), optimistic updates would provide instant feedback.

### 33. Reassess whether Next.js is worth the overhead

Everything is `"use client"` — there is no SSR/SSG benefit in the current Tauri-only context. If desktop is the only target, consider whether Vite + React would reduce complexity and bundle size.

---

## Summary of Effort Estimates

| Category | Items | Estimated Effort | Progress |
|----------|-------|------------------|----------|
| 🔴 Security fixes (1-5) | 5 items | 2-3 days | 4/5 done ✅ |
| 🔴 Build fixes (6a-6d) | 4 items | 0.5-1 day | 4/4 done ✅ |
| 🟠 Auth/state model (7-9) | 3 items | 2-3 days |
| 🟠 Search/locking (10) | 1 item | 0.5 day |
| 🟠 Key rotation scope (11) | 1 item | 0.5 day |
| 🟡 UX/code quality (12-21) | 10 items | 3-5 days |
| 🟢 Polish & future (22-31) | 10 items | 5-10 days |

---

## Quick Wins (can be done in < 1 hour each)

Strikethrough = already completed.

1. ~~Fix `verify_password` boolean check in auth.rs (+ encryption.rs)~~ ✅
2. ~~Fix build bugs in notes/todos/calendar/encryption~~ ✅
3. ~~Add master-password setup flow on unlock page~~ ✅
4. Fix dashboard medium-priority count
5. Fix settings import query key (`events` → `calendar`)
6. Fix nav `<a>` → `<Link>`
7. Fix RichTextEditor content prop reactivity
8. Fix TipTap CSS selectors
