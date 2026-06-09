use tauri::State;
use crate::database::pool::DbPool;
use crate::sync::supabase::SupabaseClient;
use crate::crypto::manager::EncryptionManager;
use serde_json::Value;
use crate::sync::conflict::VectorClock;

fn entity_to_table(entity_type: &str) -> &str {
    match entity_type {
        "note" => "notes",
        "todo" => "todos",
        "event" => "calendar_events",
        _ => entity_type,
    }
}

/// Get the Supabase client, decrypting the API key from storage if needed.
async fn get_client(conn: &rusqlite::Connection, enc: &EncryptionManager) -> Result<SupabaseClient, String> {
    let url: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'supabase_url'",
        [], |r| r.get(0)
    ).map_err(|_| "Supabase URL not configured. Go to Settings to configure sync.".to_string())?;
    let stored_key: String = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'supabase_key'",
        [], |r| r.get(0)
    ).map_err(|_| "Supabase key not configured. Go to Settings to configure sync.".to_string())?;
    // Decrypt the API key if it's encrypted
    let key = enc.try_decrypt(&stored_key).await;
    Ok(SupabaseClient::new(url, key))
}

#[tauri::command]
pub async fn sync_push(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<serde_json::Value, String> {
    // Read all unsynced items first, then drop the connection lock before network I/O
    let (items, client) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let client = get_client(&conn, &enc).await?;
        let items: Vec<(i64, String, String, String, Option<String>)> = {
            let mut stmt = conn.prepare(
                "SELECT id, entity_type, entity_id, operation, payload FROM sync_queue WHERE synced = 0 ORDER BY created_at ASC"
            ).map_err(|e| e.to_string())?;
            let mut out = Vec::new();
            let rows = stmt.query_map([], |r| Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))).map_err(|e| e.to_string())?;
            for row in rows { if let Ok(r) = row { out.push(r); } }
            out
        };
        (items, client)
    }; // conn lock dropped here

    let total = items.len();
    let mut pushed = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let mut synced_ids: Vec<i64> = Vec::new();

    // Network I/O happens without holding the DB lock
    for (id, entity_type, entity_id, operation, payload) in &items {
        let table = entity_to_table(entity_type);
        let result = match operation.as_str() {
            "delete" => client.delete_entity(table, entity_id).await,
            _ => {
                if let Some(p) = payload {
                    match serde_json::from_str::<Value>(p) {
                        Ok(data) => client.upsert_entity(table, &data).await,
                        Err(e) => Err(format!("Invalid payload JSON: {}", e)),
                    }
                } else {
                    Err("Missing payload for create/update operation".to_string())
                }
            }
        };

        match result {
            Ok(()) => {
                synced_ids.push(*id);
                pushed += 1;
            }
            Err(e) => errors.push(format!("{} ({}): {}", entity_id, operation, e)),
        }
    }

    // Re-acquire connection to mark items synced and update timestamp
    {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        for id in &synced_ids {
            conn.execute("UPDATE sync_queue SET synced = 1 WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
        }
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('last_sync_at', ?1, ?2)",
            (now.to_string(), now)
        ).ok();
    }

    // Read remaining count
    let remaining = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get::<i64>(0)).unwrap_or(0)
    };

    Ok(serde_json::json!({
        "pushed": pushed,
        "total": total,
        "remaining": remaining,
        "errors": errors,
    }))
}

#[tauri::command]
pub async fn sync_pull(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<serde_json::Value, String> {
    // Phase 1: Read config and pull remote data (no DB lock held during network I/O)
    let (since, client) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let client = get_client(&conn, &enc).await?;
        let last_sync: Option<String> = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'last_sync_at'",
            [], |r| r.get(0)
        ).ok();
        let since = last_sync.and_then(|s| s.parse::<i64>().ok());
        (since, client)
    }; // conn lock dropped here

    let tables = vec!["notes", "todos", "calendar_events"];
    let mut all_remote_rows: Vec<(String, Vec<Value>)> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Network I/O happens without holding the DB lock
    for table in &tables {
        match client.pull_entities(table, since).await {
            Ok(rows) => all_remote_rows.push((table.to_string(), rows)),
            Err(e) => errors.push(format!("{}: {}", table, e)),
        }
    }

    // Phase 2: Apply pulled changes (re-acquire DB lock for writes)
    let mut pulled = 0usize;
    {
        let conn = pool.get().await.map_err(|e| e.to_string())?;

        for (table, rows) in &all_remote_rows {
            for row in rows {
                let entity_id = row.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                if entity_id.is_empty() {
                    continue;
                }

                let device_id = "supabase";
                let remote_vc: VectorClock = row.get("vector_clock")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_else(|| {
                        let mut vc = VectorClock::new();
                        vc.increment(device_id);
                        vc
                    });

                let entity_type = match table.as_str() {
                    "notes" => "note",
                    "todos" => "todo",
                    "calendar_events" => "event",
                    _ => continue,
                };

                // Check local version via sync_queue vector clocks
                let local_vc: Option<VectorClock> = conn.query_row(
                    "SELECT vector_clock FROM sync_queue WHERE entity_id = ?1 AND entity_type = ?2 AND synced = 1 ORDER BY id DESC LIMIT 1",
                    [&entity_id, entity_type],
                    |r| r.get::<_, String>(0)
                ).ok().and_then(|s| serde_json::from_str(&s).ok());

                let should_apply = match &local_vc {
                    Some(local) => {
                        let remote_newer = remote_vc.clocks.iter().any(|(node, count)| {
                            *count > local.get(node)
                        });
                        let local_newer = local.clocks.iter().any(|(node, count)| {
                            *count > remote_vc.get(node)
                        });
                        remote_newer && !local_newer
                    }
                    None => true,
                };

                if should_apply {
                    let op = if row.get("deleted").and_then(|v| v.as_bool()).unwrap_or(false) {
                        "delete"
                    } else {
                        "update"
                    };

                    match op {
                        "delete" => {
                            conn.execute(
                                &format!("DELETE FROM {} WHERE id = ?1", table),
                                [&entity_id],
                            ).map_err(|e| e.to_string())?;
                        }
                        _ => {
                            let exists: bool = conn.query_row(
                                &format!("SELECT COUNT(*) FROM {} WHERE id = ?1", table),
                                [&entity_id],
                                |r| r.get::<_, i64>(0),
                            ).map(|c| c > 0).unwrap_or(false);

                            if exists {
                                match table.as_str() {
                                    "notes" => {
                                        conn.execute(
                                            "UPDATE notes SET title=?1, content=?2, user_id=?3, notebook_id=?4, word_count=?5, reading_time=?6, is_pinned=?7, is_archived=?8, updated_at=?9 WHERE id=?10",
                                            rusqlite::params![
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, String>("content").unwrap_or_default(),
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, Option<String>>("notebook_id").ok().flatten(),
                                                row.get::<_, i64>("word_count").unwrap_or(0),
                                                row.get::<_, i64>("reading_time").unwrap_or(1),
                                                row.get::<_, bool>("is_pinned").unwrap_or(false) as i64,
                                                row.get::<_, bool>("is_archived").unwrap_or(false) as i64,
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                &entity_id,
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    "todos" => {
                                        conn.execute(
                                            "UPDATE todos SET title=?1, description=?2, user_id=?3, is_completed=?4, priority=?5, due_date=?6, updated_at=?7 WHERE id=?8",
                                            rusqlite::params![
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, Option<String>>("description").ok().flatten(),
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, bool>("is_completed").unwrap_or(false) as i64,
                                                row.get::<_, String>("priority").unwrap_or_else(|_| "medium".into()),
                                                row.get::<_, Option<i64>>("due_date").ok().flatten(),
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                &entity_id,
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    "calendar_events" => {
                                        conn.execute(
                                            "UPDATE calendar_events SET title=?1, description=?2, user_id=?3, start_time=?4, end_time=?5, all_day=?6, color=?7, rrule=?8, updated_at=?9 WHERE id=?10",
                                            rusqlite::params![
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, Option<String>>("description").ok().flatten(),
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, i64>("start_time").unwrap_or(0),
                                                row.get::<_, i64>("end_time").unwrap_or(0),
                                                row.get::<_, bool>("all_day").unwrap_or(false) as i64,
                                                row.get::<_, String>("color").unwrap_or_else(|_| "#3b82f6".into()),
                                                row.get::<_, Option<String>>("rrule").ok().flatten(),
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                &entity_id,
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    _ => {}
                                }
                            } else {
                                match table.as_str() {
                                    "notes" => {
                                        conn.execute(
                                            "INSERT OR IGNORE INTO notes (id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                                            rusqlite::params![
                                                &entity_id,
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, Option<String>>("notebook_id").ok().flatten(),
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, String>("content").unwrap_or_default(),
                                                row.get::<_, i64>("word_count").unwrap_or(0),
                                                row.get::<_, i64>("reading_time").unwrap_or(1),
                                                row.get::<_, bool>("is_pinned").unwrap_or(false) as i64,
                                                row.get::<_, bool>("is_archived").unwrap_or(false) as i64,
                                                row.get::<_, i64>("created_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    "todos" => {
                                        conn.execute(
                                            "INSERT OR IGNORE INTO todos (id, user_id, title, description, is_completed, priority, due_date, is_archived, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8,?9)",
                                            rusqlite::params![
                                                &entity_id,
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, Option<String>>("description").ok().flatten(),
                                                row.get::<_, bool>("is_completed").unwrap_or(false) as i64,
                                                row.get::<_, String>("priority").unwrap_or_else(|_| "medium".into()),
                                                row.get::<_, Option<i64>>("due_date").ok().flatten(),
                                                row.get::<_, i64>("created_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    "calendar_events" => {
                                        conn.execute(
                                            "INSERT OR IGNORE INTO calendar_events (id, user_id, title, description, start_time, end_time, all_day, color, rrule, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                                            rusqlite::params![
                                                &entity_id,
                                                row.get::<_, String>("user_id").unwrap_or_else(|_| "local-user".into()),
                                                row.get::<_, String>("title").unwrap_or_default(),
                                                row.get::<_, Option<String>>("description").ok().flatten(),
                                                row.get::<_, i64>("start_time").unwrap_or(0),
                                                row.get::<_, i64>("end_time").unwrap_or(0),
                                                row.get::<_, bool>("all_day").unwrap_or(false) as i64,
                                                row.get::<_, String>("color").unwrap_or_else(|_| "#3b82f6".into()),
                                                row.get::<_, Option<String>>("rrule").ok().flatten(),
                                                row.get::<_, i64>("created_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                                row.get::<_, i64>("updated_at").unwrap_or_else(|_| chrono::Utc::now().timestamp()),
                                            ],
                                        ).map_err(|e| e.to_string())?;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    let now = chrono::Utc::now().timestamp();
                    let vc_json = serde_json::to_string(&remote_vc).unwrap_or_default();
                    conn.execute(
                        "INSERT INTO sync_queue (user_id, entity_type, entity_id, operation, payload, vector_clock, created_at, synced)
                         VALUES ('local-user', ?1, ?2, ?3, ?4, ?5, ?6, 1)",
                        (entity_type, &entity_id, op, &row.to_string(), &vc_json, now)
                    ).map_err(|e| e.to_string())?;
                    pulled += 1;
                }
            }
        }

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('last_sync_at', ?1, ?2)",
            (now.to_string(), now)
        ).ok();
    } // conn lock dropped here

    let pending = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get::<i64>(0)).unwrap_or(0)
    };

    Ok(serde_json::json!({
        "pulled": pulled,
        "pending": pending,
        "errors": errors,
    }))
}

#[tauri::command]
pub async fn sync_status(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get(0)).unwrap_or(0);
    let last_sync: Option<i64> = conn.query_row("SELECT value FROM app_settings WHERE key = 'last_sync_at'", [], |r| r.get::<_, String>(0)).ok()
        .and_then(|s| s.parse().ok());
    let has_config: bool = conn.query_row("SELECT COUNT(*) FROM app_settings WHERE key = 'supabase_url'", [], |r| r.get::<_, i64>(0)).unwrap_or(0) > 0;
    Ok(serde_json::json!({
        "pending": pending,
        "last_sync": last_sync,
        "configured": has_config,
    }))
}

#[tauri::command]
pub async fn configure_sync(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, url: String, key: String) -> Result<serde_json::Value, String> {
    if enc.is_locked().await {
        return Err("Database is locked. Unlock to configure sync.".into());
    }
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('supabase_url', ?1, ?2)",
        (&url, now)
    ).map_err(|e| e.to_string())?;
    // Encrypt the API key at rest
    let encrypted_key = enc.encrypt_or_pass(&key).await.map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('supabase_key', ?1, ?2)",
        (&encrypted_key, now)
    ).map_err(|e| e.to_string())?;
    // Test the connection with the plaintext key
    let client = SupabaseClient::new(url, key);
    let ok = client.test_connection().await.unwrap_or(false);
    Ok(serde_json::json!({
        "configured": true,
        "connection_ok": ok,
    }))
}

#[tauri::command]
pub async fn get_sync_config(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let url: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'supabase_url'", [], |r| r.get(0)
    ).ok();
    let key: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'supabase_key'", [], |r| r.get(0)
    ).ok();
    Ok(serde_json::json!({
        "url": url,
        "has_key": key.is_some(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_to_table_notes() {
        assert_eq!(entity_to_table("note"), "notes");
    }

    #[test]
    fn entity_to_table_todos() {
        assert_eq!(entity_to_table("todo"), "todos");
    }

    #[test]
    fn entity_to_table_events() {
        assert_eq!(entity_to_table("event"), "calendar_events");
    }

    #[test]
    fn entity_to_table_unknown_returns_self() {
        assert_eq!(entity_to_table("custom"), "custom");
    }

    #[test]
    fn entity_to_table_empty() {
        assert_eq!(entity_to_table(""), "");
    }
}
