# Todos — Live Prioritized Execution List

Priority key: 🔴 P0 (critical) / 🟠 P1 (high) / 🟡 P2 (medium) / 🟢 P3 (low)

---

## 🔴 P0 — Critical

- [x] **Sync confidentiality (roadmap #4)** — Model B implemented: sync payloads now use already-encrypted content (title/content/description are `$enc$hex...`). Supabase sees only encrypted blobs.

## 🟠 P1 — High Priority

- [x] **Auth state bootstrap (#7)** — `bootstrap()` checks backend session + encryption salt on load, hydrates Zustand from reality
- [x] **State decoupling (#8)** — `isLoggedIn` separated from `isUnlocked`; login/register no longer set `isUnlocked: true`
- [x] **Route guards (#9)** — `AuthGuard` component protects all 6 protected pages (/, /notes, /todos, /calendar, /trash, /settings)
- [x] **Search lock fix (#10)** — Load all DB rows first, drop connection lock, then decrypt and filter
- [x] **Key rotation scope (#11)** — Extended to re-encrypt attachment files on disk; TypeScript return type updated

## 🟡 P2 — Medium Priority

- [ ] **Error handling (#12)** — Structured error types, global error handler/toast system
- [ ] **Page decomposition (#13)** — Break up giant page components (notes ~300 lines, todos ~350, settings ~450)
- [x] **Dashboard medium count (#14)** — FIXED: now subtracts low priority from active count
- [x] **Settings import key (#15)** — FIXED: `"events"` → `"calendar"`
- [ ] **Bulk todo ops (#16)** — Handle recurring advancement + sync enqueue in bulk_update/bulk_delete
- [x] **Nav <Link> (#17)** — FIXED: All `<a href>` replaced with Next.js `<Link>` for client-side navigation
- [x] **RTE content sync (#18)** — FIXED: Added `useEffect` that calls `setContent()` when content prop changes
- [x] **TipTap CSS (#19)** — FIXED: Changed `.tiptap` to `.ProseMirror` (actual class used by EditorContent)
- [ ] **Undo/redo completeness (#20)** — Restore tags + attachments; fix ID/timestamp drift
- [ ] **Search deep-links (#21)** — Return item-specific routes not just /notes, /todos, /calendar

## 🟢 P3 — Nice-to-Have

- [ ] **Shared UI component library (#22)** — Modal, FormField, Card, Button, EmptyState, Toast
- [ ] **Tag management dedup (#23)** — Extract shared tag editing component
- [ ] **Search architecture for scale (#24)** — In-memory index, blind hashed terms, or FTS mirror
- [ ] **Specialized summary endpoints (#25)** — list_note_summaries, dashboard_summary, tag_usage_stats
- [ ] **Attachment performance (#26)** — Use Tauri file APIs instead of base64 IPC
- [ ] **Accessibility audit (#27)** — ARIA labels, focus management, visible focus states
- [ ] **Keyboard shortcut discoverability (#28)** — Surface shortcuts in UI
- [ ] **Reusable CRUD patterns in Rust (#29)** — Extract generic encrypted CRUD helpers
- [ ] **Integration test coverage (#30)** — Login/unlock with wrong password, session expiry, etc.
- [ ] **Calendar event type (#31)** — Fix `handleEventDrop` `any` type to `EventDropArg`
- [ ] **Optimistic updates (#32)** — Instant feedback for toggle actions
- [ ] **Reassess Next.js (#33)** — Consider Vite + React for desktop-only use

---

## ✅ Completed Stabilization Sprint

- [x] Step 1: Populate memory files (opencode + autonomous-app-builder)
- [x] Step 2: Implement sync confidentiality (Model B)
- [x] Step 3: Fix auth lifecycle (bootstrap + state + guards)
- [x] Step 4: Fix search lock + key rotation scope
- [x] Step 5: Quick wins (dashboard, settings key, nav Link, RTE, CSS)
