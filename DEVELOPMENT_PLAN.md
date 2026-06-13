# Development Plan — Knowledge Base
 
 ## Phase 1: Core Polish (Weeks 1-2)
 
 ### 1.1 Auth Flow UI
 - [x] Login page (`/login`)
 - [x] Register page (`/register`)
 - [x] Database unlock screen (`/unlock`)
 - [x] Password strength indicator
 - [x] Session persistence (keep unlocked for N minutes)
 - [x] Auto-lock on idle
 
 ### 1.2 Notes Enhancement
 - [x] Notebook organization (create/edit notebooks)
 - [x] Note ↔ notebook assignment
 - [x] Note filtering by notebook
 - [x] Drag-and-drop note reordering
 - [x] Markdown export
 - [x] Rich text image embedding (inline images via Tauri file picker)
 - [ ] Note version history (snapshot on save)
 
 ### 1.3 Todos Enhancement
 - [x] Todo ↔ note linking
 - [x] Subtask support
 - [x] Todo lists / checklists
 - [x] Filter by priority / due date / completion
 - [x] Bulk operations (delete all completed, mark all)
 - [x] Undo/redo
 
 ### 1.4 Calendar Enhancement
 - [x] Recurring event rules (RRULE parser)
 - [x] Event categories / color coding
 - [x] Week number display
 - [x] Today button + keyboard navigation
 - [x] Event drag from notes/todos
 - [x] Event resize (drag-to-resize)
 
 ---
 
 ## Phase 2: Encryption & Security (Weeks 3-4)
 
 ### 2.1 Encryption Hardening
 - [x] Encrypt ALL note content (not just marked-as-encrypted)
 - [x] Encrypt todo descriptions
 - [x] Encrypt calendar event descriptions
 - [x] Key rotation mechanism (re-encrypt with new master password)
 - [x] Key backup export (encrypted keychain file)
 - [x] Secure memory wiping (zeroize on drop for all key material)
 
 ### 2.2 Authentication Hardening
 - [x] Account lockout after N failed attempts (RateLimiter: 5 attempts / 15 min)
 - [ ] Two-factor authentication (TOTP)
 - [ ] Recovery codes
 - [ ] Hardware key support (WebAuthn/FIDO2)
 - [ ] Biometric unlock (Windows Hello / macOS Touch ID)
 
 ### 2.3 Data Integrity
 - [x] SQLite WAL checkpoint on app exit
 - [ ] Automatic database backup on startup
 - [ ] Backup rotation (keep last N)
 - [x] Database integrity check (PRAGMA integrity_check)
 - [ ] Corruption recovery (export + re-create)
 
 ---
 
 ## Phase 3: Sync & Multi-Device (Weeks 5-6)
 
### 3.1 Sync Implementation
- [x] Full Supabase sync implementation (replace stubs)
- [x] Push: upload local changes since last sync
- [x] Pull: download remote changes, merge locally
- [x] Conflict resolution UI (when CRDT merge is ambiguous)
- [x] Sync status indicator in UI
- [x] Manual sync trigger + auto-sync interval
 - [ ] Bandwidth-efficient delta sync (only changed fields)
 
 ### 3.2 Multi-Device
 - [ ] Device registration (name, type, last seen)
 - [ ] Device management UI (list devices, revoke access)
 - [ ] Cross-device encryption key sync
 - [ ] Selective sync (choose which notebooks to sync)
 - [ ] Sync logs (history of sync operations)
 
 ### 3.3 Offline-First
 - [ ] Queue sync operations when offline
 - [ ] Auto-retry on network reconnect
 - [ ] Sync conflict detection with user notification
 - [ ] Merge strategy: local-wins vs remote-wins vs user-prompt
 
 ---
 
 ## Phase 4: UI/UX Polish (Weeks 7-8)
 
 ### 4.1 Layout
 - [x] Sidebar navigation (collapsible)
 - [x] Command palette (Cmd+K / Ctrl+K)
 - [ ] Split view: note list + editor
 - [ ] Resizable panels
 - [ ] Breadcrumb navigation
 - [x] Keyboard shortcuts system
 
 ### 4.2 Search & Discovery
 - [x] Global search across all content
 - [ ] Search filters (date, notebook, tags)
 - [ ] Recent notes quick access
 - [ ] Favorite / pinned notes
 - [ ] Backlinks (notes that reference this note)
 
 ### 4.3 Themes & Appearance
 - [ ] Custom theme editor (create your own color scheme)
 - [ ] Font size / family settings
 - [ ] Compact / comfortable view modes
 - [ ] Accent color picker
 - [ ] Animated transitions
 
 ### 4.4 Accessibility
 - [ ] Screen reader support (ARIA labels)
 - [ ] Keyboard navigation for all features
 - [ ] High contrast mode
 - [ ] Reduced motion mode
 - [ ] Focus management for modals
 
 ---
 
 ## Phase 5: Advanced Features (Weeks 9-12)
 
 ### 5.1 Knowledge Graph
 - [ ] Note linking (wiki-style [[link]])
 - [ ] Backlinks panel
 - [ ] Graph visualization (force-directed graph)
 - [ ] Orphan notes detection
 - [ ] Graph filtering by notebook/tag
 
 ### 5.2 Import/Export
 - [ ] Import from Markdown files
 - [ ] Import from Notion (HTML export)
 - [ ] Import from Evernote (ENEX)
 - [ ] Import from Apple Notes (HTML)
 - [ ] Export to Markdown
 - [ ] Export to PDF
 - [ ] Full database export (encrypted archive)
 - [ ] Selective export (by notebook, date range)
 
 ### 5.3 Templates
 - [x] Note templates (daily journal, meeting notes, book notes)
 - [ ] Todo templates (project checklist, travel checklist)
 - [ ] Calendar templates (recurring meeting setup)
 - [ ] Custom template creation
 - [ ] Template marketplace (community shared)
 
 ### 5.4 Media & Attachments
 - [x] Inline image support (paste/drag-drop)
 - [x] File attachment management UI
 - [x] Image preview / lightbox
 - [ ] PDF viewer inline
 - [ ] Audio recording (voice notes)
 - [ ] Screen capture integration
 
 ---
 
 ## Phase 6: Performance & Quality (Weeks 13-14)
 
 ### 6.1 Performance
 - [ ] Lazy loading for note list
 - [ ] Virtualized scrolling (large note lists)
 - [ ] SQLite query optimization (EXPLAIN ANALYZE)
 - [ ] Frontend bundle analysis and code splitting
 - [ ] Rust compile times optimization (incremental builds)
 - [ ] Memory usage profiling
 
 ### 6.2 Testing
 - [ ] Rust unit tests for all crypto functions
 - [ ] Rust integration tests for all Tauri commands
 - [x] Frontend component tests (Vitest + Testing Library)
 - [ ] E2E tests (Playwright for web mode)
 - [ ] Tauri E2E tests (tauri-driver)
 - [ ] Property-based tests for encryption roundtrips
 - [ ] Fuzzing for parser functions
 - [ ] Load testing for sync operations
 
 ### 6.3 CI/CD
 - [ ] GitHub Actions workflow
 - [ ] Lint on PR (ESLint + Clippy + Prettier + rustfmt)
 - [x] Test on PR (unit + integration)
 - [ ] Build on main (Tauri builds for Windows/macOS/Linux)
 - [ ] Release automation (semantic-release + GitHub Releases)
 - [ ] Auto-update mechanism (Tauri updater plugin)
 
 ### 6.4 Documentation
 - [ ] API documentation for all Tauri commands
 - [ ] Developer setup guide
 - [ ] Contributing guidelines
 - [ ] Architecture Decision Records (ADRs)
 - [ ] User guide with screenshots
 - [x] Keyboard shortcuts reference (in notes page: Ctrl+E, Ctrl+S, Ctrl+Shift+F)
 
 ---
 
 ## Priority Matrix
 
 | Priority | Phase | Effort | Impact |
 |----------|-------|--------|--------|
 | 🔴 P0 | Auth Flow UI | 1 week | High — app is unusable without login |
 | 🔴 P0 | Encryption Hardening | 1 week | High — security is non-negotiable |
 | 🟠 P1 | Full Sync Implementation | 2 weeks | High — core value proposition |
 | 🟠 P1 | Notes Enhancement | 1 week | High — primary use case |
 | 🟡 P2 | UI/UX Polish | 2 weeks | Medium — improves daily experience |
 | 🟡 P2 | Import/Export | 1 week | Medium — lock-in prevention |
 | 🟢 P3 | Knowledge Graph | 2 weeks | Low — nice-to-have, complex |
 | 🟢 P3 | Templates | 1 week | Low — productivity multiplier |
 | 🔵 P4 | Testing & CI/CD | 2 weeks | High — quality foundation |
 
 ---
 
 ## Technical Debt to Address
 
 1. ~~Replace stub sync code~~ — Supabase client implementation complete
 2. ~~Error boundaries~~ — Frontend error handling improved with toast notifications
 3. **Type safety** — Some Tauri commands use `String` for IDs instead of typed IDs
 4. **Database migrations** — No migration rollback support (21 migrations exist)
 5. **Logging** — No structured logging in Rust backend
 6. **Config management** — No runtime configuration (hardcoded values)
 7. **Icon generation** — Placeholder icons need real design
 
 ---
 
 ## Success Metrics
 
 | Metric | Target |
 |--------|--------|
 | Cold start | < 2 seconds |
 | Note search | < 100ms |
 | Sync full | < 30 seconds (1000 notes) |
 | Binary size | < 50MB (Windows), < 30MB (macOS) |
 | Memory idle | < 100MB |
 | Test coverage | > 80% |