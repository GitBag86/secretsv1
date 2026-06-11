# Architectural Decisions

Format:
- Date:
- Decision:
- Reason:
- Impact:

---

## Finalized Decisions

1. **Single-User Local Model**
   - Hardcoded `user_id = "local-user"` everywhere
   - Simplifies queries; multi-user would require significant refactor

2. **Tauri v2 + Next.js 14 SSG Architecture**
   - Desktop shell + web frontend
   - All pages `"use client"` — no SSR/SSG benefit in desktop mode

3. **SQLite via rusqlite (bundled, WAL mode)**
   - Zero-config local database
   - Single `DbPool` (Mutex<Connection>) creates potential bottleneck

4. **AES-256-GCM for Content Encryption**
   - Stored as `$enc$` + hex(nonce||ciphertext)
   - FTS5 unusable — search uses decrypt+filter

5. **Argon2id for Password Hashing + Key Derivation**
   - OWASP params: m=64MB, t=3, p=4
   - SHA-256 legacy fallback with versioned salt

6. **Decrypt+Filter Search over Encrypted Data**
   - Linear scan; scales poorly for 1000+ items

7. **Transactional Key Rotation**
   - ROLLBACK on failure preserves old key
   - Currently excludes attachments, sync queue, config

8. **Attachments Encrypted at Rest**
   - AES-256-GCM via encrypt_raw/decrypt_raw

9. **Templates Stored Unencrypted**
   - Accessible before unlock; not sensitive

## Open / Unresolved Decisions

- Sync security model (Model A: plaintext cloud vs Model B: zero-knowledge)
- Auth vs unlock state unification
- Next.js vs Vite migration feasibility
- Search architecture for scale (in-memory index?)
