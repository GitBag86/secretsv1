use tauri::State;
use crate::database::pool::DbPool;

#[tauri::command]
pub async fn sync_push(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get(0)).unwrap_or(0);
    conn.execute("UPDATE sync_queue SET synced = 1 WHERE synced = 0", []).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "synced": count }))
}

#[tauri::command]
pub async fn sync_pull(_pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({ "synced": 0 }))
}

#[tauri::command]
pub async fn sync_status(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let pending: i64 = conn.query_row("SELECT COUNT(*) FROM sync_queue WHERE synced = 0", [], |r| r.get(0)).unwrap_or(0);
    let last_sync: Option<i64> = conn.query_row("SELECT MAX(created_at) FROM sync_queue WHERE synced = 1", [], |r| r.get(0)).ok();
    Ok(serde_json::json!({ "pending": pending, "last_sync": last_sync }))
}
