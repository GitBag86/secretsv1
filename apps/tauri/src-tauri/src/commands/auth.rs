use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto;
use crate::crypto::manager::EncryptionManager;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const RATE_LIMIT_MAX_ATTEMPTS: u32 = 5;
const RATE_LIMIT_WINDOW_SECS: i64 = 15 * 60; // 15 minutes

struct RateLimitEntry {
    attempts: u32,
    window_start: i64,
}

pub struct RateLimiter {
    entries: Mutex<HashMap<String, RateLimitEntry>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self { entries: Mutex::new(HashMap::new()) }
    }

    fn now_secs() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
    }

    /// Returns true if the request should be blocked.
    pub fn check_rate_limit(&self, email: &str) -> bool {
        let mut entries = self.entries.lock().unwrap();
        let now = Self::now_secs();
        let entry = entries.entry(email.to_lowercase()).or_insert(RateLimitEntry { attempts: 0, window_start: now });
        if now - entry.window_start > RATE_LIMIT_WINDOW_SECS {
            entry.attempts = 0;
            entry.window_start = now;
        }
        entry.attempts >= RATE_LIMIT_MAX_ATTEMPTS
    }

    /// Record a failed attempt.
    pub fn record_failure(&self, email: &str) {
        let mut entries = self.entries.lock().unwrap();
        let now = Self::now_secs();
        let entry = entries.entry(email.to_lowercase()).or_insert(RateLimitEntry { attempts: 0, window_start: now });
        if now - entry.window_start > RATE_LIMIT_WINDOW_SECS {
            entry.attempts = 0;
            entry.window_start = now;
        }
        entry.attempts += 1;
    }

    /// Reset counter on successful auth.
    pub fn reset(&self, email: &str) {
        self.entries.lock().unwrap().remove(&email.to_lowercase());
    }
}

#[derive(Serialize)]
pub struct AuthResponse { pub user: UserResponse, pub token: String }
#[derive(Serialize)]
pub struct UserResponse { pub id: String, pub email: String, pub name: Option<String>, pub created_at: i64, pub updated_at: i64 }

fn validate_email(email: &str) -> Result<(), String> {
    if email.len() > 254 { return Err("Email too long (max 254 chars)".into()); }
    if !email.contains('@') || !email.contains('.') { return Err("Invalid email format".into()); }
    if email.chars().any(|c| c.is_control()) { return Err("Email contains invalid characters".into()); }
    Ok(())
}

fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 { return Err("Password must be at least 8 characters".into()); }
    if password.len() > 128 { return Err("Password too long (max 128 chars)".into()); }
    if password.chars().any(|c| c.is_control()) { return Err("Password contains invalid characters".into()); }
    Ok(())
}

#[tauri::command]
pub async fn register(pool: State<'_, DbPool>, email: String, password: String, name: Option<String>) -> Result<AuthResponse, String> {
    validate_email(&email)?;
    validate_password(&password)?;
    if let Some(ref n) = name {
        if n.len() > 100 { return Err("Name too long (max 100 chars)".into()); }
        if n.chars().any(|c| c.is_control()) { return Err("Name contains invalid characters".into()); }
    }
    let id = "local-user".to_string();
    let password_hash = crypto::argon2::hash_password(&password).map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO users (id, email, name, password_hash, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", (&id, &email, &name, &password_hash, now, now)).map_err(|e| e.to_string())?;
    Ok(AuthResponse { user: UserResponse { id, email, name, created_at: now, updated_at: now }, token: "local-session".into() })
}

#[tauri::command]
pub async fn login(pool: State<'_, DbPool>, limiter: State<'_, RateLimiter>, email: String, password: String) -> Result<AuthResponse, String> {
    validate_email(&email)?;
    if password.is_empty() { return Err("Password is required".into()); }
    if password.len() > 128 { return Err("Password too long".into()); }
    if limiter.check_rate_limit(&email) {
        return Err("Too many failed attempts. Please try again later.".into());
    }
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let row: (String, String, Option<String>, String, i64, i64) = conn.query_row("SELECT id, email, name, password_hash, created_at, updated_at FROM users WHERE email = ?1", [&email], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?))).map_err(|e| e.to_string())?;
    let valid = crypto::argon2::verify_password(&password, &row.3).map_err(|_| "Invalid password".to_string())?;
    if !valid {
        limiter.record_failure(&email);
        return Err("Invalid password".to_string());
    }
    limiter.reset(&email);
    Ok(AuthResponse { user: UserResponse { id: row.0, email: row.1, name: row.2, created_at: row.4, updated_at: row.5 }, token: "local-session".into() })
}

#[tauri::command]
pub async fn unlock_database(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, limiter: State<'_, RateLimiter>, password: String) -> Result<serde_json::Value, String> {
    let unlock_key = "_unlock_".to_string();
    if limiter.check_rate_limit(&unlock_key) {
        return Err("Too many failed attempts. Please try again later.".into());
    }

    // Read stored hash and salt, then drop conn BEFORE async key derivation
    let (stored_hash, salt_hex) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let stored_hash: String = conn.query_row(
            "SELECT password_hash FROM users LIMIT 1",
            [],
            |r| r.get(0)
        ).map_err(|_| "No master password set. Please register first.".to_string())?;
        let salt_hex: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get(0)).map_err(|_| "No encryption salt found".to_string())?;
        (stored_hash, salt_hex)
    }; // conn dropped — Mutex released before async key derivation

    let valid = crypto::argon2::verify_password(&password, &stored_hash)
        .map_err(|e| format!("Password verification failed: {}", e))?;
    if !valid {
        limiter.record_failure(&unlock_key);
        return Err("Invalid master password".to_string());
    }

    limiter.reset(&unlock_key);

    // Key derivation and HMAC — no Mutex held
    enc.set_key_from_stored_salt(&password, &salt_hex).await.map_err(|e| format!("Key derivation failed: {}", e))?;
    let now = chrono::Utc::now().timestamp();
    let _session_secret = enc.set_session_secret().await;
    let session_hmac = enc.compute_session_hmac(&now.to_string()).await.map_err(|e| e.to_string())?;

    // Re-acquire Mutex for DB writes
    {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_unlocked_at', ?1, ?2)", (now.to_string(), now)).map_err(|e| e.to_string())?;
        conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_hmac', ?1, ?2)", (&session_hmac, now)).map_err(|e| e.to_string())?;
    }
    Ok(serde_json::json!({ "success": true }))
}

#[tauri::command]
pub async fn lock_database(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<(), String> {
    enc.clear_key().await;
    enc.clear_session_secret().await;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM app_settings WHERE key = 'session_unlocked_at'", []).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM app_settings WHERE key = 'session_hmac'", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn check_session(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<serde_json::Value, String> {
    // Read all DB values in one Mutex-guarded section, then release the lock
    // before doing any async HMAC work (avoids holding Mutex across await)
    let (unlocked_at_str, stored_hmac_result, timeout) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let unlocked_at_str: Result<String, _> = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'session_unlocked_at'", [], |r| r.get(0)
        );
        let stored_hmac: Result<String, _> = conn.query_row(
            "SELECT value FROM app_settings WHERE key = 'session_hmac'", [], |r| r.get(0)
        );
        let timeout: i64 = conn.query_row(
            "SELECT COALESCE((SELECT value FROM app_settings WHERE key = 'session_timeout'), '15')",
            [], |r| r.get::<_, String>(0)
        ).map(|v| v.parse().unwrap_or(15)).unwrap_or(15);
        (unlocked_at_str, stored_hmac, timeout)
    }; // conn dropped here — Mutex released

    match unlocked_at_str {
        Ok(unlocked_at_str) => {
            // Verify HMAC to detect DB tampering — no Mutex held
            if let Ok(hmac) = stored_hmac_result {
                match enc.verify_session_hmac(&unlocked_at_str, &hmac).await {
                    Ok(true) => {} // HMAC valid
                    Ok(false) => {
                        // DB tampered — clear session and clean up DB
                        enc.clear_session_secret().await;
                        {
                            let conn = pool.get().await.map_err(|e| e.to_string())?;
                            let _ = conn.execute("DELETE FROM app_settings WHERE key = 'session_unlocked_at'", []);
                            let _ = conn.execute("DELETE FROM app_settings WHERE key = 'session_hmac'", []);
                        }
                        return Ok(serde_json::json!({ "valid": false }));
                    }
                    Err(_) => {
                        // No session secret in memory (locked) — can't verify, but check raw timestamp
                        // This happens on app restart when DB has session but memory is empty
                    }
                }
            }
            let unlocked_at: i64 = unlocked_at_str.parse().map_err(|_| "Invalid timestamp".to_string())?;
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
pub async fn refresh_session(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<serde_json::Value, String> {
    let now = chrono::Utc::now().timestamp();
    // Compute HMAC BEFORE acquiring the DB Mutex to avoid Mutex-across-await
    let session_hmac = enc.compute_session_hmac(&now.to_string()).await.map_err(|e| e.to_string())?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_unlocked_at', ?1, ?2)", (now.to_string(), now)).map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_hmac', ?1, ?2)", (&session_hmac, now)).map_err(|e| e.to_string())?;
    Ok(serde_json::json!({ "refreshed_at": now }))
}

#[tauri::command]
pub async fn logout() -> Result<(), String> { Ok(()) }
