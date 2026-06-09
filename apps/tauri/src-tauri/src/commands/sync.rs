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
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    let client = get_client(&conn, &enc).await?;

    // Read all unsynced items
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

    let total = items.len();
    let mut pushed = 0usize;
    let mut errors: Vec<String> = Vec::new();

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
                conn.execute("UPDATE sync_queue SET synced = 1 WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
                pushed += 1;
            }
            Err(e) => errors.push(format!("{} ({}): {}", entity_id, operation, e)),
        }
    }

    // Update last_sync timestamp
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('last_sync_at', ?1, ?2)",
        (now.to_string(), now)
    ).ok();

    let remaining: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get(0)).unwrap_or(0);

    Ok(serde_json::json!({
        "pushed": pushed,
        "total": total,
        "remaining": remaining,
        "errors": errors,
    }))
}

#[tauri::command]
pub async fn sync_pull(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<serde_json::Value, String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    let client = get_client(&conn, &enc).await?;

    let last_sync: Option<String> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'last_sync_at'",
        [], |r| r.get(0)
    ).ok();
    let since = last_sync.and_then(|s| s.parse::<i64>().ok());

    let tables = vec!["notes", "todos", "calendar_events"];
    let mut pulled = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for table in &tables {
        let rows = match client.pull_entities(table, since).await {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{}: {}", table, e));
                continue;
            }
        };

        for row in &rows {
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

            let entity_type = match *table {
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
                    // If remote has updates not in local, apply
                    let remote_newer = remote_vc.clocks.iter().any(|(node, count)| {
                        *count > local.get(node)
                    });
                    let local_newer = local.clocks.iter().any(|(node, count)| {
                        *count > remote_vc.get(node)
                    });
                    remote_newer && !local_newer
                }
                None => true, // Doesn't exist locally, always apply
            };

            if should_apply {
                // Store the operation in the sync queue for playback
                let op = if row.get("deleted").and_then(|v| v.as_bool()).unwrap_or(false) {
                    "delete"
                } else {
                    "update"
                };

                let now = chrono::Utc::now().timestamp();
                let vc_json = serde_json::to_string(&remote_vc).unwrap_or_default();
                conn.execute(
                    "INSERT INTO sync_queue (user_id, entity_type, entity_id, operation, payload, vector_clock, created_at, synced)
                     VALUES ('local-user', ?1, ?2, ?3, ?4, ?5, ?6, 0)",
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

    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get(0)).unwrap_or(0);

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
