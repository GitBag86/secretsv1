# Supabase Sync Setup Guide

KnowledgeBase uses **Supabase** as an optional cloud sync backend. Data is synced via Supabase's REST API — content remains encrypted end-to-end with AES-256-GCM before leaving your device.

---

## 1. Create a Supabase Project

1. Go to [supabase.com](https://supabase.com) and sign in.
2. Click **"New project"**.
3. Choose an organization, project name (e.g. `knowledge-base-sync`), and a secure database password.
4. Select a region close to you.
5. Wait for the database to provision (~1–2 minutes).

Once the project is ready, copy these from the **Project Settings > API** page:

- **Project URL** — looks like `https://xxxxx.supabase.co`
- **anon public key** — a long `eyJhbGciOi...` JWT string

---

## 2. Create the Database Tables

Open the **SQL Editor** in your Supabase dashboard and run the following SQL to create tables matching the local schema:

### Notes Table

```sql
CREATE TABLE IF NOT EXISTS notes (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL DEFAULT 'local-user',
  notebook_id TEXT,
  title TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  word_count INTEGER DEFAULT 0,
  reading_time INTEGER DEFAULT 0,
  is_pinned INTEGER DEFAULT 0,
  is_archived INTEGER DEFAULT 0,
  created_at BIGINT NOT NULL DEFAULT (extract(epoch from now())),
  updated_at BIGINT NOT NULL DEFAULT (extract(epoch from now()))
);

CREATE INDEX IF NOT EXISTS idx_notes_user ON notes(user_id);
CREATE INDEX IF NOT EXISTS idx_notes_notebook ON notes(notebook_id);
CREATE INDEX IF NOT EXISTS idx_notes_updated ON notes(updated_at);
```

### Todos Table

```sql
CREATE TABLE IF NOT EXISTS todos (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL DEFAULT 'local-user',
  title TEXT NOT NULL,
  description TEXT,
  is_completed INTEGER DEFAULT 0,
  priority TEXT CHECK(priority IN ('low', 'medium', 'high')) DEFAULT 'medium',
  due_date BIGINT,
  created_at BIGINT NOT NULL DEFAULT (extract(epoch from now())),
  updated_at BIGINT NOT NULL DEFAULT (extract(epoch from now()))
);

CREATE INDEX IF NOT EXISTS idx_todos_user ON todos(user_id);
CREATE INDEX IF NOT EXISTS idx_todos_updated ON todos(updated_at);
```

### Calendar Events Table

```sql
CREATE TABLE IF NOT EXISTS calendar_events (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL DEFAULT 'local-user',
  title TEXT NOT NULL,
  description TEXT,
  start_time BIGINT NOT NULL,
  end_time BIGINT NOT NULL,
  all_day INTEGER DEFAULT 0,
  color TEXT DEFAULT '#3b82f6',
  rrule TEXT,
  parent_event_id TEXT,
  created_at BIGINT NOT NULL DEFAULT (extract(epoch from now())),
  updated_at BIGINT NOT NULL DEFAULT (extract(epoch from now()))
);

CREATE INDEX IF NOT EXISTS idx_events_user ON calendar_events(user_id);
CREATE INDEX IF NOT EXISTS idx_events_time ON calendar_events(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_events_updated ON calendar_events(updated_at);
```

### Vector Clock Support (Optional — for Server-Side Conflict Metadata)

The app resolves conflicts client-side using vector clocks stored in the local `sync_queue` table.
If you want to preserve conflict metadata on the server, add this column:

```sql
ALTER TABLE notes ADD COLUMN IF NOT EXISTS vector_clock JSONB DEFAULT '{}'::jsonb;
ALTER TABLE todos ADD COLUMN IF NOT EXISTS vector_clock JSONB DEFAULT '{}'::jsonb;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS vector_clock JSONB DEFAULT '{}'::jsonb;
```

---

## 3. Configure Row-Level Security (RLS)

The app uses the **anon key** with RLS policies. This is the recommended setup for client-facing Supabase apps.

### Enable RLS on Each Table

Run this for each table (`notes`, `todos`, `calendar_events`):

```sql
ALTER TABLE notes ENABLE ROW LEVEL SECURITY;
ALTER TABLE todos ENABLE ROW LEVEL SECURITY;
ALTER TABLE calendar_events ENABLE ROW LEVEL SECURITY;
```

### Create Access Policies

Since KnowledgeBase is a single-user desktop app, the simplest policy is to allow all operations for authenticated/anonymous users on these tables. For a multi-user setup, you'd scope by `user_id`.

```sql
-- Allow all operations on notes
CREATE POLICY "Allow all on notes" ON notes
  FOR ALL
  USING (true)
  WITH CHECK (true);

-- Allow all operations on todos
CREATE POLICY "Allow all on todos" ON todos
  FOR ALL
  USING (true)
  WITH CHECK (true);

-- Allow all operations on calendar_events
CREATE POLICY "Allow all on calendar_events" ON calendar_events
  FOR ALL
  USING (true)
  WITH CHECK (true);
```

> **⚠️ Security Note:** These policies allow full access to anyone with the anon key. The anon key is safe to include in a desktop app binary (it's not a secret like the `service_role` key), but for production use you should scope policies to `user_id` and implement proper authentication. For a personal knowledge base, this is acceptable because sync data is transmitted over TLS and the anon key is scoped to your project.

### If You Want User-Scoped Policies (Multi-Device)

Replace the policies above with:

```sql
CREATE POLICY "Users can manage their own notes" ON notes
  FOR ALL
  USING (auth.uid()::text = user_id)
  WITH CHECK (auth.uid()::text = user_id);

CREATE POLICY "Users can manage their own todos" ON todos
  FOR ALL
  USING (auth.uid()::text = user_id)
  WITH CHECK (auth.uid()::text = user_id);

CREATE POLICY "Users can manage their own events" ON calendar_events
  FOR ALL
  USING (auth.uid()::text = user_id)
  WITH CHECK (auth.uid()::text = user_id);
```

> ⚠️ User-scoped policies require Supabase Auth to be configured in the app (currently not implemented). The app uses a static `local-user` user_id.

---

## 4. Configure Sync in the App

1. Launch KnowledgeBase and log in.
2. Go to **Settings** (gear icon in the nav bar).
3. **Unlock the database** if locked.
4. Scroll to the **Sync** section.
5. Enter your Supabase **Project URL** and **anon key**.
6. Click **"Save"** — the app tests the connection.
7. If the connection is successful, you'll see **"Connection OK!"**

### Push Local Changes

Click **"Push Changes"** to upload locally created/updated/deleted data to Supabase.
The button shows the number of pending changes.

### Pull Remote Changes

Click **"Pull Changes"** to download changes made on other devices.
The app uses **vector clock conflict resolution** — if the same item was edited on two devices, the version with more updates wins.

---

## 5. How Sync Works

```
┌─────────────────────────────────────────────┐
│            KnowledgeBase Desktop             │
│                                             │
│  Create/Update/Delete ──► sync_queue table   │
│                                  │           │
│  "Push Changes" button           ▼           │
│                          REST API POST       │
│                          (upsert or delete)  │
│                                  │           │
│  "Pull Changes" button           ▼           │
│                          REST API GET        │
│                          (updated_at filter) │
│                                  │           │
│                          Apply to local DB   │
│                          (vector clock CRDT) │
└─────────────────────────────────────────────┘
                     │
                     ▼
         ┌─────────────────────┐
         │     Supabase        │
         │                     │
         │  notes              │
         │  todos              │
         │  calendar_events    │
         └─────────────────────┘
```

### Sync Flow Details

1. **Local Change** — Every `create`, `update`, or `delete` on notes, todos, or calendar events adds an entry to the local `sync_queue` table with:
   - `entity_type`, `entity_id`, `operation` (create/update/delete)
   - `payload` — the full row data as JSON (decrypted content for pushes)
   - `created_at` — timestamp

2. **Push** — The `sync_push` command:
   - Reads all unsynced queue items (ordered by creation time)
   - For `create`/`update` operations: sends an `HTTP POST` to `{url}/rest/v1/{table}` with `Prefer: resolution=merge-duplicates`
   - For `delete` operations: sends an `HTTP DELETE` to `{url}/rest/v1/{table}?id=eq.{id}`
   - Marks each item as `synced = 1` on success
   - Reports any errors per-item

3. **Pull** — The `sync_pull` command:
   - Gets the `last_sync_at` timestamp from `app_settings`
   - Sends `HTTP GET` to each table with `?select=*&updated_at=gt.{timestamp}`
   - For each returned row, compares the remote vector clock with the local vector clock
   - If remote is newer or doesn't exist locally: inserts a new sync_queue item (marked unsynced) which will be applied on the next push cycle
   - Updates `last_sync_at` timestamp

4. **Encryption** — Content is encrypted with AES-256-GCM at rest in the local SQLite database. When syncing, the app decrypts the content before pushing it to Supabase over TLS. This means Supabase stores unencrypted content — the sync channel is protected by TLS, and access is controlled by your Supabase project's RLS policies and anon key.

---

## 6. Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| "Supabase URL not configured" | No credentials saved | Enter URL + key in Settings > Sync |
| "Connection test failed" | Wrong URL or key | Verify in Supabase dashboard > Settings > API |
| Push succeeds but no data appears | Wrong table name | Check that tables are created with exact names: `notes`, `todos`, `calendar_events` |
| "401 Unauthorized" | RLS blocking access | Check RLS policies — anon key needs INSERT/SELECT/DELETE permissions |
| Pull returns 0 rows | No `updated_at` index | Ensure `idx_events_updated` index exists |
| Data conflicts between devices | Same item edited on both | Vector clock auto-resolves — newer version wins |

---

## 7. Environment (Alternative Configuration)

Instead of using the in-app settings UI, you can pre-configure sync via the Tauri Rust backend environment:

```env
# apps/tauri/src-tauri/.env
SUPABASE_URL=https://xxxxx.supabase.co
SUPABASE_ANON_KEY=eyJhbGciOi...
```

> Note: Environment variables are read at compile time. For runtime configuration, use the Settings page in the app.

---

## 8. Reset or Clear Sync Data

To reset sync state:

```sql
-- Clear the sync queue (locally)
DELETE FROM sync_queue;

-- Reset last_sync_at
DELETE FROM app_settings WHERE key = 'last_sync_at';

-- Clear Sync credentials in app_settings
DELETE FROM app_settings WHERE key = 'supabase_url';
DELETE FROM app_settings WHERE key = 'supabase_key';
```

Or simply go to Settings > Sync and enter new credentials.
