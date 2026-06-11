# Project State

## Product Summary
Local-first, encrypted personal knowledge base desktop app. Notes, todos, calendar with end-to-end encryption. Tauri v2 desktop shell + Next.js 14 SSG frontend + SQLite backend. Optional Supabase sync.

## Current Maturity: **Advanced MVP / Internal Alpha**
- All core CRUD features implemented (notes, todos, calendar)
- Encryption foundations solid (AES-256-GCM, Argon2id)
- Security audit completed, 5 critical bugs fixed
- Not yet production-safe — several open P0/P1 items

## What's Implemented
- Notes CRUD + rich text editor (TipTap) + word count + reading time
- Todos CRUD + priority levels + due dates + completion toggle + recurrence
- Calendar CRUD + FullCalendar (day/week/month) + drag-and-drop rescheduling
- Auth pages (login, register, unlock, master-password setup)
- Encryption (AES-256-GCM content, Argon2id hashing + key derivation)
- Attachments (encrypted at rest)
- Templates (stored unencrypted)
- Tags (many-to-many notes/todos)
- Session timeout + auto-lock on idle
- Command palette (Cmd+K unified search)
- Theme (dark/light)
- Export/import
- Archive/trash
- Unified search
- Undo/redo (partial)
- Sync scaffolding (Supabase client, queue, vector clocks)

## What's Blocked / Open
- **P0**: Sync payloads contain plaintext (roadmap #4) — Model B fix pending
- **P1**: Auth state bootstrap from backend on startup (#7)
- **P1**: Login/register sets isUnlocked without backend unlock (#8)
- **P1**: No route guards on protected pages (#9)
- **P1**: Search holds DB lock during decrypt (#10)
- **P1**: Key rotation scope incomplete (#11)
- **P2**: 10 items (error handling, page decomposition, quick wins, etc.)
- **P3**: 10 items (knowledge graph, CI/CD, accessibility, etc.)

## Security Posture
- Encryption at rest: AES-256-GCM ✓
- KDF: Argon2id (OWASP params) ✓
- Session expiry enforced server-side ✓
- Key rotation transactional ✓
- Attachments encrypted ✓
- Sync: NOT end-to-end encrypted (planned for Model B)
- Rate limiting: Not implemented (low priority, local app)
- Session timeout DB manipulation: Not fixed (low priority)

## Current Sprint Focus
Stabilization sprint — resolve sync model, auth lifecycle, search lock, key rotation scope, quick wins.

## Primary Risks
1. Sync payload confidentiality undermines trust model
2. Auth/unlock state inconsistency causes wrong UI on refresh
3. Single mutexed DB connection creates serialization bottleneck
4. Doc drift between README, AGENTS, and actual code
