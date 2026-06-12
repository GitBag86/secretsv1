use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use crate::sync::enqueue_sync;
use super::helpers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Note { pub id: String, pub user_id: String, pub notebook_id: Option<String>, pub title: String, pub content: String, pub word_count: i64, pub reading_time: i64, pub is_pinned: bool, pub is_archived: bool, pub created_at: i64, pub updated_at: i64 }

fn validate_title(title: &str) -> Result<(), String> {
    if title.is_empty() { return Err("Title cannot be empty".into()); }
    if title.len() > 10000 { return Err("Title too long (max 10000 chars)".into()); }
    if title.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err("Title contains invalid control characters".into());
    }
    Ok(())
}

fn validate_content(content: &str) -> Result<(), String> {
    if content.len() > 1_000_000 { return Err("Content too long (max 1MB)".into()); }
    Ok(())
}

/// Decrypt notes returned from DB (title + content).
pub async fn decrypt_notes(enc: &EncryptionManager, notes: &mut [Note]) {
    for n in notes {
        n.title = enc.try_decrypt(&n.title).await;
        n.content = enc.try_decrypt(&n.content).await;
    }
}

#[tauri::command]
pub async fn list_notes(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<Vec<Note>, String> {
    log::info!("[list_notes] acquiring pool lock...");
    let conn = pool.get().await.map_err(|e| { log::error!("[list_notes] pool.get failed: {}", e); e.to_string() })?;
    log::info!("[list_notes] pool lock acquired, querying...");
    let mut notes = {
        let mut stmt = conn.prepare("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes WHERE is_archived = 0 ORDER BY is_pinned DESC, updated_at DESC").map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        let rows = stmt.query_map([], |r| {
            Ok(Note { id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?, title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?, reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0, is_archived: r.get::<_, i64>(8)? != 0, created_at: r.get(9)?, updated_at: r.get(10)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { result.push(n); } }
        result
    };
    drop(conn);
    log::info!("[list_notes] lock released, decrypting {} notes...", notes.len());
    decrypt_notes(&enc, &mut notes).await;
    log::info!("[list_notes] done");
    Ok(notes)
}

#[tauri::command]
pub async fn get_note(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String) -> Result<Note, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut note = conn.query_row("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes WHERE id = ?1", [&id], |r| {
        Ok(Note { id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?, title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?, reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0, is_archived: r.get::<_, i64>(8)? != 0, created_at: r.get(9)?, updated_at: r.get(10)? })
    }).map_err(|e| e.to_string())?;
    drop(conn);
    helpers::decrypt_note(&enc, &mut note).await;
    Ok(note)
}

#[tauri::command]
pub async fn create_note(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, title: String, content: String, notebook_id: Option<String>, id: Option<String>) -> Result<Note, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    validate_title(&title)?;
    validate_content(&content)?;
    let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let user_id = "local-user".to_string();
    let et = enc.encrypt_or_pass(&title).await.map_err(|e| e.to_string())?;
    let ec = enc.encrypt_or_pass(&content).await.map_err(|e| e.to_string())?;
    let wc = content.split_whitespace().count() as i64;
    let rt = (wc / 200).max(1);
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO notes (id, user_id, notebook_id, title, content, word_count, reading_time, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)", (&id, &user_id, &notebook_id, &et, &ec, wc, rt, now, now)).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({"id": &id, "title": &et, "content": &ec, "notebook_id": &notebook_id, "word_count": wc, "reading_time": rt, "created_at": now, "updated_at": now});
    enqueue_sync(&conn, "note", &id, "create", Some(&payload.to_string())).ok();
    drop(conn);
    Ok(Note { id, user_id, notebook_id, title, content, word_count: wc, reading_time: rt, is_pinned: false, is_archived: false, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn update_note(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String, title: Option<String>, content: Option<String>, is_pinned: Option<bool>, is_archived: Option<bool>, notebook_id: Option<String>) -> Result<Note, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    if let Some(ref t) = title { validate_title(t)?; }
    if let Some(ref c) = content { validate_content(c)?; }
    // Read existing note, then drop conn BEFORE any async encryption work
    let existing = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.query_row("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes WHERE id = ?1", [&id], |r| {
            Ok(Note { id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?, title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?, reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0, is_archived: r.get::<_, i64>(8)? != 0, created_at: r.get(9)?, updated_at: r.get(10)? })
        }).map_err(|e| e.to_string())?
    }; // conn dropped here — Mutex released before async encryption
    let stored_title = if let Some(ref nt) = title { enc.encrypt_or_pass(nt).await.map_err(|e| e.to_string())? } else { existing.title.clone() };
    let stored_content = if let Some(ref nc) = content { enc.encrypt_or_pass(nc).await.map_err(|e| e.to_string())? } else { existing.content.clone() };
    let resp_title = enc.try_decrypt(&stored_title).await;
    let resp_content = enc.try_decrypt(&stored_content).await;
    let wc = content.as_ref().map(|c| c.split_whitespace().count() as i64).unwrap_or(existing.word_count);
    let rt = (wc / 200).max(1);
    let p = is_pinned.unwrap_or(existing.is_pinned);
    let a = is_archived.unwrap_or(existing.is_archived);
    let nb = notebook_id.or(existing.notebook_id);
    let now = chrono::Utc::now().timestamp();
    // Re-acquire Mutex for the UPDATE statement
    {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.execute("UPDATE notes SET title=?1, content=?2, word_count=?3, reading_time=?4, is_pinned=?5, is_archived=?6, notebook_id=?7, updated_at=?8 WHERE id=?9", (&stored_title, &stored_content, wc, rt, p as i64, a as i64, &nb, now, &id)).map_err(|e| e.to_string())?;
        let payload = serde_json::json!({"id": &id, "title": &stored_title, "content": &stored_content, "notebook_id": &nb, "word_count": wc, "reading_time": rt, "is_pinned": p, "is_archived": a, "updated_at": now});
        enqueue_sync(&conn, "note", &id, "update", Some(&payload.to_string())).ok();
    } // conn dropped here
    Ok(Note { id, user_id: existing.user_id, notebook_id: nb, title: resp_title, content: resp_content, word_count: wc, reading_time: rt, is_pinned: p, is_archived: a, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub async fn delete_note(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String) -> Result<(), String> {
    helpers::require_valid_session(&pool, &enc).await?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM note_tags WHERE note_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM notes WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    enqueue_sync(&conn, "note", &id, "delete", None).ok();
    drop(conn);
    Ok(())
}


#[tauri::command]
pub async fn search_notes(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, query: String) -> Result<Vec<Note>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut notes = {
        let mut stmt = conn.prepare("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes WHERE is_archived = 0 ORDER BY is_pinned DESC, updated_at DESC").map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        let rows = stmt.query_map([], |r| {
            Ok(Note { id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?, title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?, reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0, is_archived: r.get::<_, i64>(8)? != 0, created_at: r.get(9)?, updated_at: r.get(10)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { result.push(n); } }
        result
    };
    drop(conn);
    let q = query.to_lowercase();
    let mut out = Vec::new();
    for n in &mut notes {
        helpers::decrypt_note(&enc, n).await;
        let plain = helpers::strip_html(&n.content);
        if n.title.to_lowercase().contains(&q) || plain.to_lowercase().contains(&q) {
            out.push(n.clone());
        }
    }
    Ok(out)
}
