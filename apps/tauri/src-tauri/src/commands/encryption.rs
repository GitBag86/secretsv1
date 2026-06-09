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
    let hash = crypto::argon2::hash_password(&password).map_err(|e| e.to_string())?;
    enc.set_key(&password, &salt).await;
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (hex::encode(&salt), now)).map_err(|e| e.to_string())?;
    // Store the password hash in the users table so key rotation can verify the password
    conn.execute("UPDATE users SET password_hash = ?1, updated_at = ?2", (&hash, now)).map_err(|e| e.to_string())?;
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

    // Verify current password against stored hash
    let stored_hash: String = conn.query_row(
        "SELECT password_hash FROM users LIMIT 1",
        [],
        |r| r.get(0)
    ).map_err(|_| "No user found".to_string())?;
    crypto::argon2::verify_password(&current_password, &stored_hash)
        .map_err(|_| "Invalid current password".to_string())?;

    let salt_hex: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get(0)).map_err(|_| "No master password set".to_string())?;
    let old_salt = hex::decode(&salt_hex).map_err(|e| e.to_string())?;
    let old_key = crypto::key_derivation::derive_key(&current_password, &old_salt);
    let new_salt = crypto::key_derivation::generate_salt();
    let new_key = crypto::key_derivation::derive_key(&new_password, &new_salt);
    // Hash the new password for storage
    let new_hash = crypto::argon2::hash_password(&new_password).map_err(|e| e.to_string())?;
    let mut note_count = 0usize;
    let mut todo_count = 0usize;
    let mut event_count = 0usize;

    // Use a transaction for atomicity
    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

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

    // Fix: read description as Option<String> to handle NULL values
    let todo_rows: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM todos").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &todo_rows {
        let dt = decrypt_val(&old_key, old_title)?;
        let dd = old_desc.as_ref().map(|d| decrypt_val(&old_key, d)).transpose()?;
        conn.execute(
            "UPDATE todos SET title=?1, description=?2 WHERE id=?3",
            (encrypt_val(&new_key, &dt)?, dd.as_deref().map(|d| encrypt_val(&new_key, d)).transpose()?, id)
        ).map_err(|e| e.to_string())?;
        todo_count += 1;
    }

    // Fix: read description as Option<String> to handle NULL values
    let event_rows: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM calendar_events").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &event_rows {
        let dt = decrypt_val(&old_key, old_title)?;
        let dd = old_desc.as_ref().map(|d| decrypt_val(&old_key, d)).transpose()?;
        conn.execute(
            "UPDATE calendar_events SET title=?1, description=?2 WHERE id=?3",
            (encrypt_val(&new_key, &dt)?, dd.as_deref().map(|d| encrypt_val(&new_key, d)).transpose()?, id)
        ).map_err(|e| e.to_string())?;
        event_count += 1;
    }

    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (hex::encode(&new_salt), chrono::Utc::now().timestamp())).map_err(|e| e.to_string())?;

    // Update the password hash so the new password can be used for future verifications
    conn.execute(
        "UPDATE users SET password_hash = ?1, updated_at = ?2",
        (&new_hash, chrono::Utc::now().timestamp())
    ).map_err(|e| e.to_string())?;

    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    drop(conn);
    enc.set_key(&new_password, &new_salt).await;
    Ok(serde_json::json!({ "rotated": true, "notes": note_count, "todos": todo_count, "events": event_count }))
}
