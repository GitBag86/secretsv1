pub mod manager;
pub mod queue;
pub mod supabase;
pub mod conflict;

use rusqlite::Connection;

/// Enqueue a sync operation for a local data change.
/// Call this from create/update/delete commands to track changes for push.
pub fn enqueue_sync(conn: &Connection, entity_type: &str, entity_id: &str, operation: &str, payload: Option<&str>) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO sync_queue (user_id, entity_type, entity_id, operation, payload, created_at, synced)
         VALUES ('local-user', ?1, ?2, ?3, ?4, ?5, 0)",
        (entity_type, entity_id, operation, payload, now)
    ).map_err(|e| e.to_string())?;
    Ok(())
}
