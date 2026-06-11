# Context Snapshot

## Backend Status
- Rust backend (Tauri v2) — 15 command modules in `apps/tauri/src-tauri/src/commands/`
- Crypto: AES-256-GCM, Argon2id hashing + key derivation (with SHA-256 legacy compat)
- DB: SQLite via rusqlite (bundled), WAL mode, 19 migrations
- Single `DbPool` (Arc<Mutex<Connection>>) — potential bottleneck
- Security: All 5 P0 fixes done; sync confidentiality unresolved

## Frontend Status
- Next.js 14 SSG, all pages `"use client"`
- State: React Query (server cache) + Zustand (auth, undo stack)
- IPC: Tauri `invoke()` wrapper in `src/lib/api.ts`
- Known issues:
  - Auth state not bootstrapped from backend on load
  - Login/register sets `isUnlocked` without actual backend unlock
  - No route guards
  - SearchPalette silently catches errors
  - components.test.tsx has 7 pre-existing failures (missing useTags mock, duplicate text query)

## Sync Status
- Supabase REST client exists (mostly stubs)
- Vector clock CRDT for conflict resolution
- Sync queue with plaintext payloads (needs Model B fix)
- Device registry schema exists
- **Not production-ready** — sync model decision pending

## Testing Status
- 71 tests total (vitest, jsdom)
- 7 pre-existing failures in `components.test.tsx`
  - Missing `useTags` export in `@/hooks` mock
  - `getByText("Notes")` finds multiple elements
- Rust tests: untested in current env (no cargo available)
- No integration tests or E2E tests

## Known Issue Clusters
1. **Auth lifecycle**: state mismatch, no bootstrap, no route guards
2. **Sync confidentiality**: plaintext payloads vs E2E promise
3. **Search locking**: DB lock held during async decrypt
4. **Key rotation scope**: attachments + sync queue + config excluded
5. **Doc drift**: README claims 14 migrations, AGENTS says 19; E2E claim vs reality

## Important Commands
| Command | What |
|---------|------|
| `pnpm install` | Install dependencies |
| `pnpm dev` | turbo dev (frontend only) |
| `pnpm build` | turbo build |
| `pnpm test` | vitest run (frontend) |
| `pnpm test:rust` | cargo test (in apps/tauri/src-tauri) |
| `pnpm lint` | turbo run lint |
| `pnpm format` | prettier --write |

## DB Schema
19 migrations in `apps/tauri/src-tauri/migrations/`:
- 001 users, 002 notebooks, 003 notes, 004 todos, 005 calendar events
- 006 tags, 007 note_tags, 008 search_index, 009 encryption_keys
- 010 sync_queue, 011 attachments, 012 app_settings, 013 device_info
- 014 recurring_todos, 015 recurring_events, 016 recurring_todos_index
- 017 todo_tags, 018 templates, 019 add_todos_archived
