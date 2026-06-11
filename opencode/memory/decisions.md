# Architectural Decisions Log

Format:
- Date: YYYY-MM-DD
- Decision: What was decided
- Reason: Why this choice
- Alternatives: What else was considered
- Impact: Consequences of this decision

---

## Implemented Decisions (from codebase analysis)

### 1. Single-User Local Model
- **Decision**: Hardcoded `user_id = "local-user"` everywhere
- **Reason**: Simplicity for a personal knowledge base; multi-user not a requirement
- **Impact**: Simplifies all queries; multi-user would require significant refactor

### 2. Tauri + Next.js Architecture
- **Decision**: Desktop shell (Tauri v2) + web frontend (Next.js 14 SSG)
- **Reason**: Reuse web dev skills, static export for optional web deployment
- **Impact**: All pages are `"use client"` — no SSR/SSG benefit in desktop mode; consider Vite migration

### 3. SQLite with rusqlite
- **Decision**: Bundled SQLite via rusqlite crate, WAL mode
- **Reason**: Zero-config local database, single file, well-supported
- **Impact**: Single `DbPool` (Mutex<Connection>) creates potential bottleneck; no replication

### 4. AES-256-GCM Content Encryption
- **Decision**: Encrypt note/todo/event content with AES-256-GCM, stored as `$enc$` + hex(nonce||ciphertext)
- **Reason**: Industry standard, authenticated encryption, nonce-based
- **Impact**: FTS5 unusable — search must decrypt+filter server-side; plaintext backward compatible

### 5. Argon2id for Password Hashing + Key Derivation
- **Decision**: Argon2id (OWASP params: m=64MB, t=3, p=4) with SHA-256 legacy fallback
- **Reason**: Memory-hard, GPU-resistant; backward compatible with existing data
- **Impact**: Salt versioned with "argon2id:" prefix; slower but more secure

### 6. Decrypt+Filter Search
- **Decision**: Load all rows from DB, decrypt, then filter in Rust
- **Reason**: FTS5 can't index encrypted content
- **Impact**: Linear scan degrades with data volume; search holds DB lock during decrypt (bug)

### 7. Zustand Undo/Redo via Inverse Commands
- **Decision**: Undo stack stores inverse Tauri commands, not git-like snapshots
- **Reason**: Lightweight, synchronous, no full-state copies
- **Impact**: Tags/attachments not restored on undo; ID/timestamp drift possible

### 8. Key Rotation is Transactional
- **Decision**: Re-encrypt all rows in single SQLite transaction; ROLLBACK on failure
- **Reason**: Atomicity guarantees no partial re-encryption; old key preserved on failure
- **Impact**: Rotation scope currently excludes attachments, sync queue, and config

### 9. Templates Stored Unencrypted
- **Decision**: Template content is plaintext in SQLite
- **Reason**: Templates are not sensitive; need to be accessible before unlock
- **Impact**: Documented limitation

### 10. Attachments Encrypted at Rest
- **Decision**: File content encrypted with AES-256-GCM via encrypt_raw/decrypt_raw
- **Reason**: Attachments may contain sensitive data
- **Impact**: Old key may still be needed (not covered by rotation)

---

## Open / Unresolved Decisions

### Sync Security Model
- **Status**: UNRESOLVED — active decision needed
- **Options**:
  - Model A: "Encrypted local, plaintext cloud" (document honestly)
  - Model B: "Zero-knowledge sync" (encrypt envelopes before queueing)
- **ADR needed**: Yes

### Auth vs Unlock State Model
- **Status**: UNRESOLVED — needs design
- **Problem**: login/register sets `isUnlocked: true` in frontend without backend unlock
- **Options**: Unified state vs separate "logged in" / "database unlocked"

### Next.js vs Vite
- **Status**: Pending — raised in roadmap (#33)
- **Context**: All pages are `"use client"` — no SSR/SSG benefit in desktop mode
- **Impact**: Bundle size, build complexity

### Search Architecture for Scale
- **Status**: Pending — roadmap #24
- **Options**: In-memory index after unlock, blind hashed term index, FTS mirror
