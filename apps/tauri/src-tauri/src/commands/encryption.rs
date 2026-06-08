use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto;
use crate::crypto::manager::EncryptionManager;

const PREFIX: &str = "$enc$";

fn decrypt_val(key: &[u8; 32], val: &str) -> Result<String, String> {
    if val.starts_with(PREFIX) {
        let raw = hex::decode(val.strip_prefix(PREFIX).unwrap()).map_err(|e| e.to_string())?;
        Ok(String::from_utf8(crypto::aes_gcm::decrypt(key, &raw)?).map_err(|e| e.to_string())?)
    } else {
        Ok(val.to_string())
    }
}

fn encrypt_val(key: &[u8; 32], val: &str) -> Result<String, String> {
    Ok(format!("{}{}", PREFIX, hex::encode(crypto::aes_gcm::encrypt(key, val.as_bytes())?)))
}

#[tauri::command]
pub async fn set_master_password(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, password: String) -> Result<serde_json::Value, String> {
    let salt = crypto::key_derivation::generate_salt();
    let _hash = crypto::argon2::hash_password(&password).map_err(|e| e.to_string())?;
    enc.set_key(&password, &salt).await;
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (hex::encode(&salt), now)).map_err(|e| e.to_string())?;
    drop(conn);
    Ok(serde_json::json!({ "salt": hex::encode(&salt) }))
}

#[tauri::command]
pub async fn get_encryption_salt(pool: State<'_, DbPool>) -> Result<Option<String>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let result = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get::<_, String>(0));
    drop(conn);
    match result {
        Ok(salt) => Ok(Some(salt)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn rotate_encryption_key(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, current_password: String, new_password: String) -> Result<serde_json::Value, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let salt_hex: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get(0)).map_err(|_| "No master password set".to_string())?;
    let old_salt = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let old_key = crypto::key_derivation::derive_key(&current_password, &old_salt);
    crypto::argon2::hash_password(&current_password).map_err(|e| e.to_string())?;
    let new_salt = crypto::key_derivation::generate_salt();
    let new_key = crypto::key_derivation::derive_key(&new_password, &new_salt);
    let _new_hash = crypto::argon2::hash_password(&new_password).map_err(|e| e.to_string())?;
    let mut note_count = 0usize;
    let mut todo_count = 0usize;
    let mut event_count = 0usize;

    let note_rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare("SELECT id, title, content FROM notes").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_content) in &note_rows {
        let dt = decrypt_val(&old_key, old_title)?;
        let dc = decrypt_val(&old_key, old_content)?;
        conn.execute("UPDATE notes SET title=?1, content=?2 WHERE id=?3", (encrypt_val(&new_key, &dt)?, encrypt_val(&new_key, &dc)?, id)).map_err(|e| e.to_string())?;
        note_count += 1;
    }

    let todo_rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM todos").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &todo_rows {
        let dt = decrypt_val(&old_key, old_title)?;
        let dd = decrypt_val(&old_key, old_desc)?;
        conn.execute("UPDATE todos SET title=?1, description=?2 WHERE id=?3", (encrypt_val(&new_key, &dt)?, encrypt_val(&new_key, &dd)?, id)).map_err(|e| e.to_string())?;
        todo_count += 1;
    }

    let event_rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM calendar_events").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &event_rows {
        let dt = decrypt_val(&old_key, old_title)?;
        let dd = decrypt_val(&old_key, old_desc)?;
        conn.execute("UPDATE calendar_events SET title=?1, description=?2 WHERE id=?3", (encrypt_val(&new_key, &dt)?, encrypt_val(&new_key, &dd)?, id)).map_err(|e| e.to_string())?;
        event_count += 1;
    }

    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (hex::encode(&new_salt), chrono::Utc::now().timestamp())).map_err(|e| e.to_string())?;
    drop(conn);
    enc.set_key(&new_password, &new_salt).await;
    Ok(serde_json::json!({ "rotated": true, "notes": note_count, "todos": todo_count, "events": event_count }))
}
