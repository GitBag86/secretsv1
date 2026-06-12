use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use crate::sync::enqueue_sync;
use crate::commands::recurring_todos::advance_recurring_todo;
use crate::commands::helpers;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone)]
pub struct Todo { pub id: String, pub user_id: String, pub title: String, pub description: Option<String>, pub is_completed: bool, pub priority: String, pub due_date: Option<i64>, pub created_at: i64, pub updated_at: i64 }

fn validate_todo_title(title: &str) -> Result<(), String> {
    if title.is_empty() { return Err("Title cannot be empty".into()); }
    if title.len() > 10000 { return Err("Title too long (max 10000 chars)".into()); }
    if title.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err("Title contains invalid control characters".into());
    }
    Ok(())
}

fn validate_todo_description(desc: &str) -> Result<(), String> {
    if desc.len() > 100_000 { return Err("Description too long (max 100KB)".into()); }
    Ok(())
}

#[tauri::command]
pub async fn list_todos(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<Vec<Todo>, String> {
    let mut todos = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, user_id, title, description, is_completed, priority, due_date, created_at, updated_at FROM todos WHERE is_archived = 0 ORDER BY is_completed ASC").map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        let rows = stmt.query_map([], |r| {
            Ok(Todo { id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?, is_completed: r.get::<_, i64>(4)? != 0, priority: r.get(5)?, due_date: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(t) = row { result.push(t); } }
        result
    };
    helpers::decrypt_todos(&enc, &mut todos).await;
    Ok(todos)
}

#[tauri::command]
pub async fn create_todo(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, title: String, description: Option<String>, priority: Option<String>, due_date: Option<i64>, id: Option<String>, is_completed: Option<bool>) -> Result<Todo, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    validate_todo_title(&title)?;
    if let Some(ref d) = description { validate_todo_description(d)?; }
    let valid_priorities = ["low", "medium", "high"];
    let p = priority.unwrap_or_else(|| "medium".into());
    if !valid_priorities.contains(&p.as_str()) {
        return Err(format!("Invalid priority '{}'. Must be one of: {:?}", p, valid_priorities));
    }
    let id = id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let user_id = "local-user".to_string();
    let et = enc.encrypt_or_pass(&title).await.map_err(|e| e.to_string())?;
    let ed = if let Some(ref d) = description { Some(enc.encrypt_or_pass(d).await.map_err(|e| e.to_string())?) } else { None };
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let ic = is_completed.unwrap_or(false);
    conn.execute("INSERT INTO todos (id, user_id, title, description, is_completed, priority, due_date, is_archived, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,0,?8,?9)", (&id, &user_id, &et, &ed, ic as i64, &p, &due_date, now, now)).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({"id": &id, "title": &et, "description": &ed, "is_completed": ic, "priority": &p, "due_date": &due_date, "created_at": now, "updated_at": now});
    enqueue_sync(&conn, "todo", &id, "create", Some(&payload.to_string())).ok();
    drop(conn);
    Ok(Todo { id, user_id, title, description, is_completed: ic, priority: p, due_date, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn update_todo(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String, title: Option<String>, description: Option<String>, is_completed: Option<bool>, priority: Option<String>, due_date: Option<i64>) -> Result<Todo, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    if let Some(ref t) = title { validate_todo_title(t)?; }
    if let Some(ref d) = description { validate_todo_description(d)?; }
    if let Some(ref p) = priority {
        let valid_priorities = ["low", "medium", "high"];
        if !valid_priorities.contains(&p.as_str()) {
            return Err(format!("Invalid priority '{}'. Must be one of: {:?}", p, valid_priorities));
        }
    }
    // Read existing todo, then drop conn BEFORE any async encryption work
    let existing = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        conn.query_row("SELECT id, user_id, title, description, is_completed, priority, due_date, created_at, updated_at FROM todos WHERE id = ?1", [&id], |r| {
            Ok(Todo { id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?, is_completed: r.get::<_, i64>(4)? != 0, priority: r.get(5)?, due_date: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)? })
        }).map_err(|e| e.to_string())?
    }; // conn dropped here — Mutex released before async encryption
    let stored_t = if let Some(ref nt) = title { enc.encrypt_or_pass(nt).await.map_err(|e| e.to_string())? } else { existing.title.clone() };
    let stored_d = if let Some(ref nd) = description { Some(enc.encrypt_or_pass(nd).await.map_err(|e| e.to_string())?) } else { existing.description.clone() };
    let resp_t = enc.try_decrypt(&stored_t).await;
    let resp_d = if let Some(ref d) = stored_d { Some(enc.try_decrypt(d).await) } else { None };
    let mut c = is_completed.unwrap_or(existing.is_completed);
    let mut dd = due_date.or(existing.due_date);
    let p = priority.unwrap_or(existing.priority);
    let now = chrono::Utc::now().timestamp();

    // Re-acquire Mutex for DB operations (recurring check + UPDATE)
    {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        // If completing a recurring todo, advance it instead
        if c {
            if let Ok(()) = advance_recurring_todo(&conn, &id) {
                c = false;
                if let Ok(advanced_due) = conn.query_row(
                    "SELECT due_date FROM todos WHERE id = ?1",
                    [&id],
                    |r| r.get::<_, Option<i64>>(0),
                ) {
                    dd = advanced_due;
                }
            }
        }
        conn.execute("UPDATE todos SET title=?1, description=?2, is_completed=?3, priority=?4, due_date=?5, updated_at=?6 WHERE id=?7", (&stored_t, &stored_d, c as i64, &p, &dd, now, &id)).map_err(|e| e.to_string())?;
        let payload = serde_json::json!({"id": &id, "title": &stored_t, "description": &stored_d, "is_completed": c, "priority": &p, "due_date": &dd, "updated_at": now});
        enqueue_sync(&conn, "todo", &id, "update", Some(&payload.to_string())).ok();
    } // conn dropped here
    Ok(Todo { id, user_id: existing.user_id, title: resp_t, description: resp_d, is_completed: c, priority: p, due_date: dd, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub async fn delete_todo(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String) -> Result<(), String> {
    helpers::require_valid_session(&pool, &enc).await?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM todo_tags WHERE todo_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM recurring_todos WHERE todo_id = ?1", [&id]).ok();
    conn.execute("DELETE FROM todos WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    enqueue_sync(&conn, "todo", &id, "delete", None).ok();
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn bulk_update_todos(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, ids: Vec<String>, is_completed: bool) -> Result<(), String> {
    helpers::require_valid_session(&pool, &enc).await?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().timestamp();
    for id in &ids {
        conn.execute("UPDATE todos SET is_completed=?1, updated_at=?2 WHERE id=?3", (is_completed as i64, now, id)).map_err(|e| e.to_string())?;
    }
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn bulk_delete_todos(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, ids: Vec<String>) -> Result<(), String> {
    helpers::require_valid_session(&pool, &enc).await?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    for id in &ids {
        conn.execute("DELETE FROM todo_tags WHERE todo_id = ?1", [id]).ok();
        conn.execute("DELETE FROM recurring_todos WHERE todo_id = ?1", [id]).ok();
        conn.execute("DELETE FROM todos WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    }
    drop(conn);
    Ok(())
}
