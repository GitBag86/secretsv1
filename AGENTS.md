# KnowledgeBase — Agent Context

## Project
Local-first, encrypted personal knowledge base desktop app with notes, todos, and calendar. Tauri v2 + Next.js 14 SSG + SQLite + Tailwind CSS. Optional Supabase sync.

## Architecture
- **Monorepo** (pnpm + turborepo): `apps/frontend` (Next.js 14 SSG), `apps/tauri/src-tauri` (Rust backend)
- **DB**: SQLite via rusqlite (bundled), WAL mode, 20 migrations in `apps/tauri/src-tauri/migrations/`
- **Encryption**: AES-256-GCM client-side, Argon2id password hashing, Argon2id key derivation (with legacy SHA-256 backward compat). Content stored with `$enc$` prefix + hex(nonce||ciphertext). Backward-compatible with plaintext. Session HMAC-SHA256 detects DB tampering.
- **State**: React Query for server cache, Zustand for client state (auth, undo stack)
- **IPC**: Frontend → Tauri `invoke()` → Rust commands. All DB access through `DbPool` (Arc<Mutex<Connection>>).

## Commands
| Command | What |
|---|---|
| `pnpm dev` | turbo dev (frontend only — browser) |
| `pnpm build` | turbo build |
| `pnpm test` | vitest run (frontend tests only) |
| `pnpm test:watch` | vitest watch |
| `pnpm test:rust` | cargo test (in apps/tauri/src-tauri) |
| `pnpm lint` | turbo run lint |
| `pnpm format` | prettier --write |
| `pnpm tauri:dev` | tauri dev (full desktop app) |
| `pnpm tauri:build` | tauri build (production installer) |

## Key Architecture Decisions
- FTS5 unusable with encrypted content — server-side decrypt+filter in `search_notes` and `unified_search`
- Undo/redo uses zustand stack + inverse Tauri commands (not git-like snapshots)
- Key rotation re-encrypts all rows in a single transaction
- All DB commands use parameterized queries (no SQL injection risk)
- `user_id` is hardcoded to `"local-user"` everywhere — multi-user not implemented
- Template content is stored **unencrypted** (only note/todo/event content is encrypted)
- Attachments stored on disk **encrypted** using AES-256-GCM via `encrypt_raw`/`decrypt_raw`
- Auth endpoints (register/login/unlock) use in-memory rate limiting (5 attempts / 15 min window per email)

## Security Issues (Fixed)
1. **SHA-256 for key derivation** — FIXED: Now uses Argon2id (OWASP params: m=64MB, t=3, p=4) with backward-compatible SHA-256 fallback for legacy data. Salt versioned with "argon2id:" prefix.
2. **`encrypt_or_pass` silent fallback** — FIXED: Now returns `Result<String, String>` and propagates errors. Callers use `.map_err(|e| e.to_string())?`.
3. **Attachments unencrypted** — FIXED: Files encrypted at rest using AES-256-GCM via `encrypt_raw`/`decrypt_raw`. DB `encrypted` flag set correctly.
4. **Supabase key in plaintext** — FIXED: API key encrypted at rest via `encrypt_or_pass` when stored, decrypted via `try_decrypt` when used.
5. **Key rotation atomicity** — FIXED: Transaction ROLLBACK on any failure preserves old key + salt. No partial re-encryption.
6. **Session timeout DB manipulation** — FIXED: HMAC-SHA256 on `session_unlocked_at` timestamps. Session secret stored in memory only (never persisted). Tampered timestamps detected via `require_valid_session` and `check_session`.
7. **No rate limiting on auth** — FIXED: In-memory rate limiter tracks failed login/unlock attempts per email (5 attempts per 15-minute window). Successful auth resets counter.

## Rust Backend Structure
```
apps/tauri/src-tauri/src/
├── lib.rs              # Entry point, plugin setup, command registration
├── main.rs             # Binary entry
├── commands/           # 15 Tauri IPC handler modules
│   ├── auth.rs         # register, login, unlock, lock, session management
│   ├── notes.rs        # CRUD + search_notes (decrypt+filter)
│   ├── todos.rs        # CRUD + bulk ops, recurring todo advancement
│   ├── calendar.rs     # CRUD
│   ├── encryption.rs   # set_master_password, rotate_encryption_key
│   ├── notebooks.rs    # CRUD
│   ├── tags.rs         # CRUD + junction table ops (note_tags, todo_tags)
│   ├── attachments.rs  # File attach/delete/open (ENCRYPTED at rest)
│   ├── sync.rs         # Supabase push/pull, configure
│   ├── recurring_todos.rs # Set/remove/list recurrence rules
│   ├── templates.rs    # CRUD + create note from template
│   ├── archive.rs      # Soft-delete + trash + unified_search
│   ├── data.rs         # Export/import (full DB dump)
│   ├── helpers.rs      # Shared: strip_html, make_snippet, decrypt_note/todo/event
│   └── mod.rs          # Module declarations
├── crypto/
│   ├── aes_gcm.rs      # AES-256-GCM encrypt/decrypt
│   ├── argon2.rs       # Password hashing (Argon2id)
│   ├── key_derivation.rs # Argon2id key derivation (with legacy SHA-256 compat)
│   └── manager.rs      # EncryptionManager (holds key in memory via Mutex)
├── database/
│   ├── pool.rs         # DbPool init, migration runner
│   ├── models.rs       # Shared Rust model structs
│   └── mod.rs
└── sync/
    ├── supabase.rs     # HTTP REST client for Supabase
    ├── conflict.rs     # VectorClock CRDT
    ├── queue.rs        # Sync queue ops
    └── manager.rs
```

## Frontend Structure
```
apps/frontend/src/
├── app/                # Next.js App Router pages
│   ├── layout.tsx      # Root layout (Inter font, Providers wrapper)
│   ├── providers.tsx   # QueryClientProvider, ThemeProvider, IdleTimer, UndoRedo
│   ├── page.tsx        # Dashboard
│   ├── notes/page.tsx  # Notes with RTE, tags, attachments, search
│   ├── todos/page.tsx  # Todo list with priorities
│   ├── calendar/page.tsx # FullCalendar integration
│   ├── settings/page.tsx # Session timeout, key rotation, sync config
│   ├── login/page.tsx  # Login form
│   ├── register/page.tsx # Registration form
│   ├── unlock/page.tsx # Database unlock screen
│   └── trash/page.tsx  # Archived items
├── components/
│   ├── rich-text-editor.tsx  # TipTap editor with toolbar
│   ├── undo-redo.tsx         # Undo/redo toast + buttons
│   ├── idle-timer.tsx        # Auto-lock on idle
│   ├── notebook-sidebar.tsx  # Sidebar with drag-and-drop reorder
│   ├── nav-header.tsx        # Top navigation bar
│   ├── search-palette.tsx    # Cmd+K unified search
│   └── template-picker.tsx   # Template selection modal
├── hooks/
│   ├── useAuth.ts       # Zustand auth store (user, unlock, lock)
│   ├── useNotes.ts      # React Query + undo stack
│   ├── useTodos.ts      # React Query + undo stack
│   ├── useCalendar.ts   # React Query
│   ├── useNotebooks.ts  # React Query
│   ├── useTags.ts       # React Query
│   └── useUndoStack.ts  # Zustand undo stack store
├── lib/
│   └── api.ts           # Tauri invoke wrapper (all IPC calls)
└── types/
    └── index.ts         # TypeScript interfaces
```

## Testing
- **Frontend**: vitest + jsdom + testing-library, 9 test files in `src/__tests__/`
- **Rust**: Unit tests in each module (`#[cfg(test)]` blocks), run with `cargo test`
- Tests mock Tauri `invoke` calls — they don't test actual IPC
- No integration tests or E2E tests exist yet
- **Known issue**: `components.test.tsx` fails to load due to OXC parser bug with deep JSX nesting (pre-existing, not caused by code changes)

## Migration System
20 SQL files in `apps/tauri/src-tauri/migrations/`, applied sequentially on startup via `pool.rs`. Idempotent (ignores "duplicate column" errors). New migrations should be added with sequential numbering (021_*, 022_*, etc.).

## Environment
- Node.js 18+ required
- Rust 1.70+ with MSVC (Windows) or Xcode CLI (macOS)
- Tauri CLI installed automatically via pnpm
- Docker available for web-only mode (nginx + static export)
