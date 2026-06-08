use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto;
use crate::crypto::manager::EncryptionManager;
use serde::Serialize;

#[derive(Serialize)]
pub struct AuthResponse { pub user: UserResponse, pub token: String }
#[derive(Serialize)]
pub struct UserResponse { pub id: String, pub email: String, pub name: Option<String>, pub created_at: i64, pub updated_at: i64 }

#[tauri::command]
pub async fn register(pool: State<'_, DbPool>, email: String, password: String, name: Option<String>) -> Result<AuthResponse, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let password_hash = crypto::argon2::hash_password(&password).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", (&id, &email, &name, &password_hash, now, now)).map_err(|e| e.to_string())?;
    Ok(AuthResponse { user: UserResponse { id, email, name, created_at: now, updated_at: now }, token: "local-session".into() })
}

#[tauri::command]
pub async fn login(pool: State<'_, DbPool>, email: String, password: String) -> Result<AuthResponse, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let row: (String, String, Option<String>, String, i64, i64) = conn.query_row("SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE email = ?1", [&email], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))).map_err(|e| e.to_string())?;
    crypto::argon2::verify_password(&password, &row.3).map_err(|_| "Invalid password".to_string())?;
    Ok(AuthResponse { user: UserResponse { id: row.0, email: row.1, name: row.2, created_at: row.4, updated_at: row.5 }, token: "local-session".into() })
}

#[tauri::command]
pub async fn unlock_database(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, password: String) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let salt_hex: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get(0)).map_err(|_| "No master password set".to_string())?;
    let salt = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    enc.set_key(&password, &salt).await;
    let now = chrono::Utc::now().timestamp();
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_unlocked_at', ?1, ?2)", (now.to_string(), now)).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
pub async fn lock_database(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<(), String> {
    enc.clear_key().await;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM app_settings WHERE key = 'session_unlocked_at'", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn check_session(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row("SELECT value FROM app_settings WHERE key = 'session_unlocked_at'", [], |r| r.get(0));
    match result {
        Ok(unlocked_at_str) => {
            let unlocked_at: i64 = unlocked_at_str.parse().map_err(|_| "Invalid timestamp".to_string())?;
            let timeout: i64 = conn.query_row(
                "SELECT COALESCE((SELECT value FROM app_settings WHERE key = 'session_timeout'), '15')", [], |r| r.get::<_, String>(0)
            ).map(|v| v.parse().unwrap_or(15)).unwrap_or(15);
            let now = chrono::Utc::now().timestamp();
            let elapsed = now - unlocked_at;
            Ok(serde_json::json!({ "valid": elapsed < timeout * 60, "unlocked_at": unlocked_at, "elapsed_seconds": elapsed, "timeout_minutes": timeout }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(serde_json::json!({ "valid": false })),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn get_session_timeout(pool: State<'_, DbPool>) -> Result<i64, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let result: Result<String, _> = conn.query_row("SELECT value FROM app_settings WHERE key = 'session_timeout'", [], |r| r.get(0));
    Ok(result.map(|v| v.parse().unwrap_or(15)).unwrap_or(15))
}

#[tauri::command]
pub async fn set_session_timeout(pool: State<'_, DbPool>, minutes: i64) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_timeout', ?1, ?2)", (minutes.to_string(), now)).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn refresh_session(pool: State<'_, DbPool>) -> Result<serde_json::Value, String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_unlocked_at', ?1, ?2)", (now.to_string(), now)).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "refreshed_at": now }))
}

#[tauri::command]
pub async fn logout() -> Result<(), String> { Ok(()) }
