use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto;
use crate::crypto::manager::EncryptionManager;
use super::helpers;

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
    // On first setup there is no encryption salt yet — skip session check.
    // Only enforce session validation when a salt already exists (re-setting the password).
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let has_salt: bool = conn.query_row(
        "SELECT COUNT(*) FROM app_settings WHERE key = 'encryption_salt'",
        [],
        |r| r.get::<_, i64>(0),
    ).map(|c| c > 0).unwrap_or(false);
    drop(conn);

    if has_salt {
        helpers::require_valid_session(&pool, &enc).await?;
    }
    if password.len() < 8 { return Err("Password must be at least 8 characters".into()); }
    if password.len() > 128 { return Err("Password too long (max 128 chars)".into()); }
    if password.chars().any(|c| c.is_control()) { return Err("Password contains invalid characters".into()); }
    let salt = crypto::key_derivation::generate_salt();
    let hash = crypto::argon2::hash_password(&password).map_err(|e| e.to_string())?;
    // Use Argon2id for new key derivation (memory-hard, GPU-resistant)
    let versioned_salt = enc.set_key_argon2id(&password, &salt).await?;
    let now = chrono::Utc::now().timestamp();
    // Initialize session so refreshSession / HMAC works after first-time setup
    enc.set_session_secret().await;
    let session_hmac = enc.compute_session_hmac(&now.to_string()).await.map_err(|e| e.to_string())?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (&versioned_salt, now)).map_err(|e| e.to_string())?;
    conn.execute("UPDATE users SET password_hash = ?1, updated_at = ?2", (&hash, now)).map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_unlocked_at', ?1, ?2)", (now.to_string(), now)).map_err(|e| e.to_string())?;
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('session_hmac', ?1, ?2)", (&session_hmac, now)).map_err(|e| e.to_string())?;
    drop(conn);
    Ok(serde_json::json!({ "salt": versioned_salt }))
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

/// Export encryption key backup (encrypted with password-derived key).
#[tauri::command]
pub async fn export_key_backup(enc: State<'_, EncryptionManager>, backup_password: String) -> Result<String, String> {
    let key = enc.get_key_copy().await.ok_or_else(|| "Database locked, cannot export key backup".to_string())?;
    // Derive a key from the backup password for encrypting the backup
    let backup_key = crypto::key_derivation::derive_key_sha256(&backup_password, &[0u8; 16]);
    let encrypted = crypto::aes_gcm::encrypt(&backup_key.0, &key)?;
    Ok(hex::encode(encrypted))
}

/// Import encryption key backup.
#[tauri::command]
pub async fn import_key_backup(enc: State<'_, EncryptionManager>, backup_data: String, backup_password: String) -> Result<(), String> {
    let raw = hex::decode(&backup_data).map_err(|e| e.to_string())?;
    let backup_key = crypto::key_derivation::derive_key_sha256(&backup_password, &[0u8; 16]);
    let key = crypto::aes_gcm::decrypt(&backup_key.0, &raw)?;
    if key.len() != 32 {
        return Err("Invalid backup data: incorrect length".to_string());
    }
    let mut key_arr = [0u8; 32];
    key_arr.copy_from_slice(&key);
    *enc.key.lock().await = Some(key_arr);
    Ok(())
}

#[tauri::command]
pub async fn rotate_encryption_key(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, current_password: String, new_password: String) -> Result<serde_json::Value, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    if new_password.len() < 8 { return Err("New password must be at least 8 characters".into()); }
    if new_password.len() > 128 { return Err("New password too long (max 128 chars)".into()); }
    if new_password.chars().any(|c| c.is_control()) { return Err("New password contains invalid characters".into()); }
    if current_password.is_empty() { return Err("Current password is required".into()); }

    // Read DB values BEFORE starting the transaction to avoid Mutex-across-await
    let (stored_hash, salt_hex) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let stored_hash: String = conn.query_row(
            "SELECT password_hash FROM users LIMIT 1",
            [],
            |r| r.get(0)
        ).map_err(|_| "No user found".to_string())?;
        let salt_hex: String = conn.query_row("SELECT value FROM app_settings WHERE key = 'encryption_salt'", [], |r| r.get(0)).map_err(|_| "No master password set".to_string())?;
        (stored_hash, salt_hex)
    }; // conn dropped before async key derivation

    let valid = crypto::argon2::verify_password(&current_password, &stored_hash)
        .map_err(|_| "Invalid current password".to_string())?;
    if !valid {
        return Err("Invalid current password".to_string());
    }

    // Derive old key using version-aware KDF (supports both legacy SHA-256 and Argon2id)
    let (old_key, _) = crypto::key_derivation::derive_key_from_stored_salt(&current_password, &salt_hex)
        .map_err(|e| format!("Failed to derive old key: {}", e))?;
    // Generate new key using Argon2id — no Mutex held
    let new_salt = crypto::key_derivation::generate_salt();
    let new_versioned_salt = enc.set_key_argon2id(&new_password, &new_salt).await?;
    let new_key = enc.get_key_copy().await
        .ok_or_else(|| "EncryptionManager key not set after Argon2id derivation".to_string())?;
    // Hash the new password for storage
    let new_hash = crypto::argon2::hash_password(&new_password).map_err(|e| e.to_string())?;

    // Now acquire the Mutex for the transaction (all key derivation is done)
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut note_count = 0usize;
    let mut todo_count = 0usize;
    let mut event_count = 0usize;
    let mut attachment_count = 0usize;

    // Use a transaction for atomicity — if re-encryption fails, ROLLBACK preserves old key + salt
    conn.execute("BEGIN TRANSACTION", []).map_err(|e| e.to_string())?;

    let note_rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare("SELECT id, title, content FROM notes").map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_content) in &note_rows {
        let dt = decrypt_val(&old_key, old_title).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        let dc = decrypt_val(&old_key, old_content).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        conn.execute("UPDATE notes SET title=?1, content=?2 WHERE id=?3", (encrypt_val(&new_key, &dt).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?, encrypt_val(&new_key, &dc).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?, id)).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        note_count += 1;
    }

    // Fix: read description as Option<String> to handle NULL values
    let todo_rows: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM todos").map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &todo_rows {
        let dt = decrypt_val(&old_key, old_title).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        let dd = old_desc.as_ref().map(|d| decrypt_val(&old_key, d)).transpose().map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        conn.execute(
            "UPDATE todos SET title=?1, description=?2 WHERE id=?3",
            (encrypt_val(&new_key, &dt).map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e
            })?, dd.as_deref().map(|d| encrypt_val(&new_key, d)).transpose().map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e
            })?, id)
        ).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        todo_count += 1;
    }

    // Fix: read description as Option<String> to handle NULL values
    let event_rows: Vec<(String, String, Option<String>)> = {
        let mut stmt = conn.prepare("SELECT id, title, description FROM calendar_events").map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    for (id, old_title, old_desc) in &event_rows {
        let dt = decrypt_val(&old_key, old_title).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        let dd = old_desc.as_ref().map(|d| decrypt_val(&old_key, d)).transpose().map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e
        })?;
        conn.execute(
            "UPDATE calendar_events SET title=?1, description=?2 WHERE id=?3",
            (encrypt_val(&new_key, &dt).map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e
            })?, dd.as_deref().map(|d| encrypt_val(&new_key, d)).transpose().map_err(|e| {
                let _ = conn.execute("ROLLBACK", []);
                e
            })?, id)
        ).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        event_count += 1;
    }

    // Re-encrypt attachment files on disk
    let attachment_rows: Vec<(String, String, String)> = {
        let mut stmt = conn.prepare("SELECT id, storage_path, filename FROM attachments WHERE encrypted = 1").map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        let mut out = Vec::new();
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            e.to_string()
        })?;
        for row in rows { if let Ok(r) = row { out.push(r); } }
        out
    };
    // Read, decrypt, re-encrypt, and write each attachment file
    for (id, storage_path, _filename) in &attachment_rows {
        let raw_data = std::fs::read(storage_path).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Failed to read attachment {}: {}", id, e)
        })?;
        let decrypted = crypto::aes_gcm::decrypt(&old_key, &raw_data).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Failed to decrypt attachment {}: {}", id, e)
        })?;
        let re_encrypted = crypto::aes_gcm::encrypt(&new_key, &decrypted).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Failed to re-encrypt attachment {}: {}", id, e)
        })?;
        std::fs::write(storage_path, &re_encrypted).map_err(|e| {
            let _ = conn.execute("ROLLBACK", []);
            format!("Failed to write attachment {}: {}", id, e)
        })?;
        attachment_count += 1;
    }

    // Update salt and password hash — if this fails, ROLLBACK
    conn.execute("INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES ('encryption_salt', ?1, ?2)", (&new_versioned_salt, chrono::Utc::now().timestamp())).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;
    conn.execute(
        "UPDATE users SET password_hash = ?1, updated_at = ?2",
        (&new_hash, chrono::Utc::now().timestamp())
    ).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        e.to_string()
    })?;

    // Commit transaction — if any step above failed, we already ROLLBACKed and returned Err
    conn.execute("COMMIT", []).map_err(|e| e.to_string())?;
    drop(conn);
    Ok(serde_json::json!({ "rotated": true, "notes": note_count, "todos": todo_count, "events": event_count, "attachments": attachment_count }))
}
