# Todos — Live Prioritized Execution List

Priority key: 🔴 P0 (critical) / 🟠 P1 (high) / 🟡 P2 (medium) / 🟢 P3 (low)

---

## 🔴 P0 — Critical

- [ ] **Sync confidentiality (roadmap #4)** — Implement Model B: encrypt sync queue payloads with AES-256-GCM before queueing. Update docs to match.

## 🟠 P1 — High Priority

- [ ] **Auth state bootstrap (#7)** — On app load, check backend session + encryption status, hydrate Zustand from reality
- [ ] **State decoupling (#8)** — Separate "logged in" from "database unlocked" in frontend state model
- [ ] **Route guards (#9)** — Protect /notes, /todos, /calendar, /, /trash, /settings from unauthenticated/unlocked access
- [ ] **Search lock fix (#10)** — Load rows first, drop DB lock, then decrypt and filter
- [ ] **Key rotation scope (#11)** — Extend to cover attachments, sync queue, encrypted config; add tests

## 🟡 P2 — Medium Priority

- [ ] **Error handling (#12)** — Structured error types, global error handler/toast system
- [ ] **Page decomposition (#13)** — Break up giant page components (notes ~300 lines, todos ~350, settings ~450)
- [ ] **Dashboard medium count (#14)** — Fix to count only medium-priority todos
- [ ] **Settings import key (#15)** — Fix invalidation key from "events" to "calendar"
- [ ] **Bulk todo ops (#16)** — Handle recurring advancement + sync enqueue in bulk_update/bulk_delete
- [ ] **Nav <Link> (#17)** — Replace `<a href>` with Next.js `<Link>`
- [ ] **RTE content sync (#18)** — Fix editor not updating when content prop changes
- [ ] **TipTap CSS (#19)** — Fix globals.css selectors targeting .tiptap class
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

## Active Sprint: Stabilization

- [ ] Step 1: Populate memory files (opencode + autonomous-app-builder)
- [ ] Step 2: Implement sync confidentiality (Model B)
- [ ] Step 3: Fix auth lifecycle (bootstrap + state + guards)
- [ ] Step 4: Fix search lock + key rotation scope
- [ ] Step 5: Quick wins (dashboard, settings key, nav Link, RTE, CSS)
