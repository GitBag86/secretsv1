use tauri::State;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use base64::Engine;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Attachment {
    pub id: String,
    pub user_id: String,
    pub note_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_path: String,
    pub encrypted: bool,
    pub created_at: i64,
}

fn get_attachments_dir(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let att_dir = app_dir.join("attachments");
    std::fs::create_dir_all(&att_dir).map_err(|e| e.to_string())?;
    Ok(att_dir)
}

#[tauri::command]
pub async fn attach_file(
    app: tauri::AppHandle,
    pool: State<'_, DbPool>,
    enc: State<'_, EncryptionManager>,
    note_id: String,
    filename: String,
    mime_type: String,
    data_base64: String,
) -> Result<Attachment, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = "local-user".to_string();
    let now = chrono::Utc::now().timestamp();

    // Decode base64 data
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&data_base64)
        .map_err(|e| format!("Failed to decode file data: {}", e))?;

    let original_size = decoded.len() as i64;

    // Encrypt file data at rest if the database is unlocked
    let (stored_data, is_encrypted) = if !enc.is_locked().await {
        match enc.encrypt_raw(&decoded).await {
            Ok(enc_data) => (enc_data, true),
            Err(_) => (decoded.clone(), false), // Fall back to unencrypted on error
        }
    } else {
        (decoded.clone(), false)
    };

    // Save file to attachments directory
    let att_dir = get_attachments_dir(&app)?;
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let storage_name = format!("{}.{}", id, ext);
    let file_path = att_dir.join(&storage_name);
    std::fs::write(&file_path, &stored_data).map_err(|e| format!("Failed to save file: {}", e))?;

    let storage_path = file_path.to_string_lossy().to_string();

    // Insert DB record
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO attachments (id, user_id, note_id, filename, mime_type, size, storage_path, encrypted, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        (&id, &user_id, &note_id, &filename, &mime_type, original_size, &storage_path, is_encrypted as i64, now),
    ).map_err(|e| e.to_string())?;
    drop(conn);

    Ok(Attachment {
        id,
        user_id,
        note_id,
        filename,
        mime_type,
        size: original_size,
        storage_path,
        encrypted: is_encrypted,
        created_at: now,
    })
}

#[tauri::command]
pub async fn list_note_attachments(pool: State<'_, DbPool>, note_id: String) -> Result<Vec<Attachment>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, note_id, filename, mime_type, size, storage_path, encrypted, created_at
         FROM attachments WHERE note_id = ?1 ORDER BY created_at ASC"
    ).map_err(|e| e.to_string())?;
    let mut attachments = Vec::new();
    let rows = stmt.query_map([&note_id], |r| {
        Ok(Attachment {
            id: r.get(0)?,
            user_id: r.get(1)?,
            note_id: r.get(2)?,
            filename: r.get(3)?,
            mime_type: r.get(4)?,
            size: r.get(5)?,
            storage_path: r.get(6)?,
            encrypted: r.get::<_, i64>(7)? != 0,
            created_at: r.get(8)?,
        })
    }).map_err(|e| e.to_string())?;
    for row in rows {
        if let Ok(a) = row {
            attachments.push(a);
        }
    }
    Ok(attachments)
}

#[tauri::command]
pub async fn delete_attachment(
    app: tauri::AppHandle,
    pool: State<'_, DbPool>,
    id: String,
) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;

    // Get the storage path so we can delete the file
    let storage_path: String = conn.query_row(
        "SELECT storage_path FROM attachments WHERE id = ?1",
        [&id],
        |r| r.get(0),
    ).map_err(|_| "Attachment not found".to_string())?;

    conn.execute("DELETE FROM attachments WHERE id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    drop(conn);

    // Delete the file from disk (ignore errors if file is already gone)
    let path = std::path::Path::new(&storage_path);
    if path.exists() {
        std::fs::remove_file(path).ok();
    }

    Ok(())
}

#[tauri::command]
pub async fn open_attachment(
    app: tauri::AppHandle,
    pool: State<'_, DbPool>,
    enc: State<'_, EncryptionManager>,
    id: String,
) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let (storage_path, is_encrypted): (String, bool) = conn.query_row(
        "SELECT storage_path, encrypted FROM attachments WHERE id = ?1",
        [&id],
        |r| Ok((r.get(0)?, r.get::<_, i64>(1)? != 0)),
    ).map_err(|_| "Attachment not found".to_string())?;
    drop(conn);

    // Read file from disk
    let raw_data = std::fs::read(&storage_path)
        .map_err(|e| format!("Failed to read attachment file: {}", e))?;

    // Decrypt if necessary
    let data = if is_encrypted && !enc.is_locked().await {
        enc.decrypt_raw(&raw_data).await
            .map_err(|e| format!("Failed to decrypt attachment: {}", e))?
    } else if is_encrypted && enc.is_locked().await {
        return Err("Cannot open encrypted attachment: database is locked".to_string());
    } else {
        raw_data
    };

    // Write decrypted data to a temp file and open with OS default app
    let temp_dir = std::env::temp_dir();
    let filename = std::path::Path::new(&storage_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment");
    let temp_path = temp_dir.join(filename);
    std::fs::write(&temp_path, &data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    app.shell()
        .open_path(&temp_path.to_string_lossy(), None)
        .map_err(|e| format!("Failed to open file: {}", e))?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AttachmentCount {
    pub note_id: String,
    pub count: i64,
}

#[tauri::command]
pub async fn get_all_attachment_counts(pool: State<'_, DbPool>) -> Result<Vec<AttachmentCount>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT note_id, COUNT(*) as cnt FROM attachments GROUP BY note_id"
    ).map_err(|e| e.to_string())?;
    let mut counts = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(AttachmentCount { note_id: r.get(0)?, count: r.get(1)? })
    }).map_err(|e| e.to_string())?;
    for row in rows {
        if let Ok(c) = row {
            counts.push(c);
        }
    }
    Ok(counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attachment() -> Attachment {
        Attachment {
            id: "att-1".into(),
            user_id: "user-1".into(),
            note_id: "note-1".into(),
            filename: "photo.jpg".into(),
            mime_type: "image/jpeg".into(),
            size: 1024,
            storage_path: "/tmp/attachments/uuid.jpg".into(),
            encrypted: true,
            created_at: 1700000000,
        }
    }

    fn make_count() -> AttachmentCount {
        AttachmentCount { note_id: "note-1".into(), count: 3 }
    }

    #[test]
    fn attachment_serialize_roundtrip() {
        let a = make_attachment();
        let json = serde_json::to_string(&a).unwrap();
        let restored: Attachment = serde_json::from_str(&json).unwrap();
        assert_eq!(a, restored);
    }

    #[test]
    fn attachment_fields() {
        let a = make_attachment();
        assert_eq!(a.filename, "photo.jpg");
        assert_eq!(a.mime_type, "image/jpeg");
        assert_eq!(a.size, 1024);
        assert!(a.encrypted);
    }

    #[test]
    fn attachment_encrypted_flag() {
        let mut a = make_attachment();
        a.encrypted = false;
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("false"));
    }

    #[test]
    fn attachment_count_serialize_roundtrip() {
        let c = make_count();
        let json = serde_json::to_string(&c).unwrap();
        let restored: AttachmentCount = serde_json::from_str(&json).unwrap();
        assert_eq!(c, restored);
    }

    #[test]
    fn attachment_count_fields() {
        let c = make_count();
        assert_eq!(c.note_id, "note-1");
        assert_eq!(c.count, 3);
    }

    #[test]
    fn clone_produces_equal() {
        let a = make_attachment();
        let c = make_count();
        assert_eq!(a, a.clone());
        assert_eq!(c, c.clone());
    }
}
