# Knowledge Base — Local-First Encrypted Personal Knowledge System

A desktop application for managing notes, todos, and calendar events with end-to-end encryption, built with Tauri v2, Next.js 14, and SQLite.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Tauri Window                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │           Next.js (Static Site Generator)         │  │
│  │  ┌─────────┐ ┌────────┐ ┌──────────┐ ┌────────┐  │  │
│  │  │Dashboard│ │ Notes  │ │   Todos  │ │Calendar│  │  │
│  │  └────┬────┘ └───┬────┘ └────┬─────┘ └───┬────┘  │  │
│  │       │          │           │            │        │  │
│  │  ┌────┴──────────┴───────────┴────────────┴────┐  │  │
│  │  │         React Query + Zustand               │  │  │
│  │  └────────────────┬────────────────────────────┘  │  │
│  └───────────────────┼───────────────────────────────┘  │
│                      │  Tauri IPC (invoke)               │
│  ┌───────────────────┴───────────────────────────────┐  │
│  │              Rust Backend                          │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │  │
│  │  │ Commands │ │  Crypto  │ │   Sync Engine    │  │  │
│  │  │ (auth,   │ │ (argon2, │ │ (supabase,       │  │  │
│  │  │  notes,  │ │  aes-gcm,│ │  queue,          │  │  │
│  │  │  todos,  │ │  sha256) │ │  vector clocks)  │  │  │
│  │  │  calendar│ │          │ │                  │  │  │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────────────┘  │  │
│  │       │            │            │                  │  │
│  │  ┌────┴────────────┴────────────┴──────────────┐  │  │
│  │  │          SQLite (rusqlite, WAL mode)        │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                         │
                         │ Optional sync
                         ▼
                ┌─────────────────┐
                │   Supabase      │
                │   (REST API)    │
                └─────────────────┘
```

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Desktop shell | Tauri | 2.x |
| Frontend | Next.js | 14 (SSG) |
| UI | Tailwind CSS | 3.x |
| State | Zustand + React Query | 5.x |
| Rich text | TipTap | 2.x |
| Calendar | FullCalendar | 6.x |
| Database | SQLite (rusqlite) | 0.32 (bundled) |
| Encryption | AES-256-GCM | — |
| KDF | Argon2id | 0.5 |
| Backend | Rust | 1.96+ |

## Project Structure

```
knowledge-base/
├── apps/
│   ├── frontend/              # Next.js SSG application
│   │   └── src/
│   │       ├── app/           # Pages: dashboard, notes, todos, calendar
│   │       ├── components/    # TipTap editor, FullCalendar wrapper
│   │       ├── hooks/         # useAuth, useNotes, useTodos, useCalendar
│   │       ├── lib/           # Tauri API bridge (api.ts)
│   │       └── types/         # TypeScript interfaces
│   └── tauri/
│       └── src-tauri/         # Rust backend
│           ├── src/
│           │   ├── commands/   # Tauri IPC handlers
│           │   ├── crypto/     # Encryption, hashing, key derivation
│           │   ├── database/   # SQLite pool, models, migrations
│           │   └── sync/       # Supabase sync, queue, conflict resolution
│           └── migrations/     # 14 SQL schema files
├── packages/
│   ├── shared/                # Shared types (placeholder)
│   └── ui/                    # Shared UI utils (placeholder)
├── Dockerfile                 # Web-only nginx build
├── docker-compose.yml         # Container orchestration
└── nginx.conf                 # SPA routing + gzip
```

## Getting Started

### Prerequisites

- **Node.js** 18+ and **pnpm** 9+
- **Rust** 1.70+ with MSVC toolchain (Windows) or Xcode CLI (macOS)
- **WebView2** (Windows) — comes with Windows 10/11
- **Tauri CLI** — installed automatically via `pnpm`

### Development

```bash
# Install dependencies
pnpm install

# Run in development mode (frontend only — browser)
pnpm dev

# Run as Tauri desktop app
pnpm tauri:dev

# Build frontend for production
pnpm build

# Build desktop app installer
pnpm tauri:build
```

### Docker (Web-Only Mode)

```bash
docker compose up --build
# Access at http://localhost:3000
```

> **Note:** Docker mode serves the Next.js frontend without Tauri backend features (local encryption, SQLite, native OS integration).

## Features

### Implemented

- **Notes** — Create, edit, delete with rich text (TipTap), word count, reading time
- **Todos** — CRUD with priority levels (high/medium/low), due dates, completion toggle
- **Calendar** — FullCalendar integration, day/week/month views, drag-and-drop rescheduling
- **Authentication** — User registration, password login, database unlock
- **Encryption** — AES-256-GCM content encryption, Argon2id password hashing, per-entity key derivation
- **Sync** — Supabase REST client with push/pull, CRDT vector clocks for conflict resolution
- **Search** — FTS5 full-text search index over notes
- **Tags** — Many-to-many tag system for notes
- **Attachments** — File attachment metadata storage
- **Recurring Todos** — Recurrence rule storage
- **Dark/Light Theme** — CSS variables with `next-themes`
- **Docker Deployment** — Web-only nginx container

### Database Schema

14 migrations covering:

| Migration | Purpose |
|-----------|---------|
| 001 | Users (id, email, password_hash) |
| 002 | Notebooks (color, sort_order) |
| 003 | Notes (encrypted_content, word_count, reading_time) |
| 004 | Todos (priority CHECK, due_date) |
| 005 | Calendar events (start/end, all_day, color) |
| 006 | Tags (unique user+name) |
| 007 | Note-tag junction table |
| 008 | FTS5 full-text search index |
| 009 | Encryption keys (salt, algorithm) |
| 010 | Sync queue (vector_clock, synced flag) |
| 011 | Attachments (file_path, mime_type, size) |
| 012 | App settings (key-value store) |
| 013 | Device registry |
| 014 | Recurring todos (freq, interval, until) |

## Security

- **Encryption at rest**: AES-256-GCM encrypts note content before SQLite storage
- **Key derivation**: SHA-256 derives per-entity keys from master password
- **Password hashing**: Argon2id with random salt (memory-hard, GPU-resistant)
- **Local-first**: Data never leaves device unless sync is explicitly configured
- **No telemetry**: Zero network calls in default configuration

## Scripts

| Script | Description |
|--------|-------------|
| `pnpm dev` | Start frontend dev server (localhost:3000) |
| `pnpm build` | Build frontend for production |
| `pnpm tauri:dev` | Start Tauri desktop app in dev mode |
| `pnpm tauri:build` | Build desktop app installer |
| `pnpm lint` | Run ESLint + Prettier |
| `pnpm test` | Run test suite |

## Configuration

### Supabase Sync (Optional)

KnowledgeBase can sync across devices using Supabase as the cloud backend. Data is end-to-end encrypted before leaving your device.

📖 **[Full setup guide →](apps/tauri/src-tauri/SYNC_SETUP.md)**

Covers:
- Creating a Supabase project
- Creating the required database tables
- Configuring Row-Level Security policies
- In-app sync configuration (Settings > Sync)
- Conflict resolution with vector clocks

### Tauri

Edit `apps/tauri/src-tauri/tauri.conf.json`:

- `window.title` — App window title
- `bundle.identifier` — App ID for OS integration
- `security.csp` — Content Security Policy

## License

MIT
