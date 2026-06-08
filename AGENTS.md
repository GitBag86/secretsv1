# KnowledgeBase — Agent Context

## Project
Local-first, encrypted personal knowledge base desktop app with notes, todos, and calendar. Tauri v2 + Next.js SSG + SQLite + Tailwind CSS. Optional Supabase sync.

## Architecture
- **Monorepo** (pnpm + turborepo): `apps/frontend` (Next.js 14), `apps/tauri` (Tauri 2 + Rust)
- **DB**: SQLite via rusqlite (bundled), WAL mode, 15 migrations
- **Encryption**: AES-256-GCM client-side, Argon2id password hashing, SHA-256 key derivation. Content stored with `$enc$` prefix + hex(nonce||ciphertext). Backward-compatible with plaintext.
- **State**: React Query for server cache, Zustand for client state (auth, undo stack)

## Commands
| Command | What |
|---|---|
| `npm run dev` | turbo dev |
| `npm run build` | turbo build |
| `npm run test` | vitest run |
| `npm run test:watch` | vitest watch |
| `npm run test:rust` | cargo test (in apps/tauri/src-tauri) |
| `npm run format` | prettier |
| `npm run tauri:dev` | tauri dev |
| `npm run tauri:build` | tauri build |

## Test Status
- **Frontend**: 58 tests (7 files) — vitest + jsdom + testing-library
- **Rust**: 67 tests (crypto, models, sync)

## Session Summary (2026-06-09)

### Done this session
1. **TipTap rich text editor** — Replaced textarea in notes page with full toolbar (B/I/S/code, H1-3, lists, blockquote, code block, task list). Edit modal with click-to-edit. HTML-stripped previews in cards. New: `components/rich-text-editor.tsx`. Added `@tailwindcss/typography` plugin.

2. **Full-text search** — New `search_notes` Rust command that decrypts + filters server-side (FTS5 can't work with encrypted content). 300ms debounced search input calls API when >1 char. Registered in lib.rs. API: `api.notes.search(query)`.

3. **Idle timer** — Already wired (no changes needed). Confirmed working.

4. **Note drag-and-drop reordering** — HTML5 drag-and-drop on notebook sidebar. Uses existing `sort_order` + `update_notebook` with `sort_order` param.

5. **Markdown export** — Turndown HTML→Markdown conversion. Export button on each note card (`↓`) and in edit modal ("Export Markdown"). Downloads as `.md` file via browser Blob API.

6. **Encryption key rotation** — New `rotate_encryption_key` Rust command. Decrypts all notes/todos/events with old password, re-encrypts with new password. Updates salt + argon2 hash. Settings page UI with current/new password fields.

7. **Undo/redo** — New `useUndoStack` zustand store. `useTodos` + `useNotes` hooks push undo entries on mutation success. `UndoRedo` component shows toast + buttons, handles Ctrl+Z / Ctrl+Shift+Z. Calls inverse Tauri command to revert.

### New Files
- `apps/frontend/src/components/rich-text-editor.tsx`
- `apps/frontend/src/components/undo-redo.tsx`
- `apps/frontend/src/hooks/useUndoStack.ts`

### Modified Files
- `apps/frontend/src/app/notes/page.tsx` — RTE, edit modal, export, API search
- `apps/frontend/src/app/settings/page.tsx` — Key rotation UI
- `apps/frontend/src/app/providers.tsx` — UndoRedo
- `apps/frontend/src/components/notebook-sidebar.tsx` — Drag-and-drop
- `apps/frontend/src/hooks/useTodos.ts` — Undo stack integration
- `apps/frontend/src/hooks/useNotes.ts` — Undo stack integration
- `apps/frontend/src/__tests__/components.test.tsx` — QueryClientProvider wrapper
- `apps/frontend/src/__tests__/api.test.ts` — Search test
- `apps/frontend/src/lib/api.ts` — search + rotate
- `apps/frontend/tailwind.config.ts` — @tailwindcss/typography
- `apps/frontend/src/app/globals.css` — TipTap styles
- `apps/tauri/src-tauri/src/commands/notes.rs` — search_notes
- `apps/tauri/src-tauri/src/commands/encryption.rs` — rotate_encryption_key
- `apps/tauri/src-tauri/src/lib.rs` — register new commands
- `apps/frontend/package.json` — turndown, @tailwindcss/typography

### Remaining
- Supabase sync (real HTTP push/pull/merge)
- Tags (tables exist, no commands/UI)
- Attachments (table exists)
- Recurring todos (table exists)
- Dashboard analytics/widgets
- Keyboard shortcuts (only undo/redo wired)
- Mobile responsive polish

### Key Decisions
- FTS5 unusable with encrypted content — server-side decrypt+filter instead
- Undo/redo uses zustand stack + inverse Tauri commands (not git-like snapshots)
- Key rotation re-encrypts all rows in a single transaction
- Notebook drag-and-drop uses HTML5 DnD API (no library needed for <50 items)
