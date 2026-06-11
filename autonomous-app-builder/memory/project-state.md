# Current Project State

- **Features implemented:**
  - Notes CRUD + rich text editor (TipTap) + word count + reading time
  - Todos CRUD + priority levels + due dates + completion toggle + recurrence
  - Calendar CRUD + FullCalendar (day/week/month) + drag-and-drop
  - Auth pages (login, register, unlock, master-password setup)
  - Encryption (AES-256-GCM content, Argon2id hashing + key derivation)
  - Attachments (encrypted at rest using AES-256-GCM)
  - Templates (stored unencrypted)
  - Tags (many-to-many for notes/todos)
  - Session timeout + auto-lock on idle
  - Command palette (Cmd+K unified search)
  - Theme (dark/light)
  - Export/import
  - Archive/trash
  - Unified search
  - Undo/redo (partial)
  - Sync scaffolding (Supabase client, queue, vector clocks)

- **Features pending:**
  - Sync confidentiality (Model B: zero-knowledge encrypted sync)
  - Auth state bootstrap from backend
  - Auth/unlock state separation
  - Route guards on protected pages
  - Search DB lock decoupling
  - Complete key rotation scope
  - Structured error handling
  - Integration/E2E testing
  - CI/CD pipeline

- **Known issues:**
  - Sync payloads contain plaintext (contradicts E2E encrypted sync promise)
  - Auth/encryption state mismatch on refresh
  - DB lock held during async search decryption
  - Key rotation excludes attachments, sync queue, config
  - Doc drift (README claims 14 migrations, AGENTS says 19)
  - 7 pre-existing test failures in components.test.tsx

- **API contract version:** Internal / unversioned

- **DB schema version:** 19 migrations (migrations/001 through 019)

- **Current security posture:**
  - Encryption at rest: ✓ (AES-256-GCM)
  - Argon2id hashing + key derivation: ✓ (with SHA-256 legacy compat)
  - Session expiry enforced server-side: ✓
  - Key rotation atomic: ✓
  - Attachments encrypted: ✓
  - Sync E2E encryption: ✗ (planned)
  - Rate limiting: ✗ (low priority — local app)
