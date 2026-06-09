use tauri::State;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use base64::Engine;
use crate::database::pool::DbPool;
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

    let size = decoded.len() as i64;

    // Save file to attachments directory
    let att_dir = get_attachments_dir(&app)?;
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let storage_name = format!("{}.{}", id, ext);
    let file_path = att_dir.join(&storage_name);
    std::fs::write(&file_path, &decoded).map_err(|e| format!("Failed to save file: {}", e))?;

    let storage_path = file_path.to_string_lossy().to_string();

    // Insert DB record
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO attachments (id, user_id, note_id, filename, mime_type, size, storage_path, encrypted, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
        (&id, &user_id, &note_id, &filename, &mime_type, size, &storage_path, now),
    ).map_err(|e| e.to_string())?;
    drop(conn);

    Ok(Attachment {
        id,
        user_id,
        note_id,
        filename,
        mime_type,
        size,
        storage_path,
        encrypted: false,
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
    id: String,
) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let storage_path: String = conn.query_row(
        "SELECT storage_path FROM attachments WHERE id = ?1",
        [&id],
        |r| r.get(0),
    ).map_err(|_| "Attachment not found".to_string())?;
    drop(conn);

    // Open the file with the OS default application
    app.shell()
        .open_path(&storage_path, None)
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
            encrypted: false,
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
        assert!(!a.encrypted);
    }

    #[test]
    fn attachment_encrypted_flag() {
        let mut a = make_attachment();
        a.encrypted = true;
        let json = serde_json::to_string(&a).unwrap();
        assert!(json.contains("true"));
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
