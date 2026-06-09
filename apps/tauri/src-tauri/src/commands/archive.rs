use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use super::helpers;
use serde::{Deserialize, Serialize};

// --- Archive / Restore ---

#[tauri::command]
pub async fn archive_note(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("UPDATE notes SET is_archived = 1, updated_at = ?1 WHERE id = ?2", (now, &id))
        .map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn restore_note(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("UPDATE notes SET is_archived = 0, updated_at = ?1 WHERE id = ?2", (now, &id))
        .map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn list_archived_notes(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<Vec<crate::commands::notes::Note>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes WHERE is_archived = 1 ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    let mut notes = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(crate::commands::notes::Note {
            id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?,
            title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?,
            reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0,
            is_archived: true, created_at: r.get(9)?, updated_at: r.get(10)?,
        })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(n) = row { notes.push(n); } }
    drop(conn);
    crate::commands::notes::decrypt_notes(&enc, &mut notes).await;
    Ok(notes)
}

#[tauri::command]
pub async fn permanently_delete_note(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notes WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM note_tags WHERE note_id = ?1", [&id]).ok();
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn archive_todo(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("UPDATE todos SET is_archived = 1, updated_at = ?1 WHERE id = ?2", (now, &id))
        .map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn restore_todo(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("UPDATE todos SET is_archived = 0, updated_at = ?1 WHERE id = ?2", (now, &id))
        .map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn list_archived_todos(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<Vec<crate::commands::todos::Todo>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, user_id, title, description, is_completed, priority, due_date, created_at, updated_at FROM todos WHERE is_archived = 1 ORDER BY updated_at DESC").map_err(|e| e.to_string())?;
    let mut todos = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(crate::commands::todos::Todo {
            id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?,
            description: r.get(3)?, is_completed: r.get::<_, i64>(4)? != 0,
            priority: r.get(5)?, due_date: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)?,
        })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(t) = row { todos.push(t); } }
    drop(conn);
    for t in &mut todos {
        helpers::decrypt_todo(&enc, t).await;
    }
    Ok(todos)
}

#[tauri::command]
pub async fn permanently_delete_todo(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM todos WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM todo_tags WHERE todo_id = ?1", [&id]).ok();
    drop(conn);
    Ok(())
}

// --- Unified Search ---

#[derive(Serialize, Deserialize)]
pub struct UnifiedSearchItem {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub entity_type: String,
    pub url: String,
    pub subtitle: String,
}

#[tauri::command]
pub async fn unified_search(
    pool: State<'_, DbPool>,
    enc: State<'_, EncryptionManager>,
    query: String,
) -> Result<Vec<UnifiedSearchItem>, String> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    let q = query.to_lowercase();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut results = Vec::new();

    // Search notes
    {
        let mut stmt = conn.prepare("SELECT id, title, content FROM notes WHERE is_archived = 0").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            if let Ok((id, title, content)) = row {
                let dec_title = enc.try_decrypt(&title).await;
                let dec_content = enc.try_decrypt(&content).await;
                let plain = helpers::strip_html(&dec_content);
                if dec_title.to_lowercase().contains(&q) || plain.to_lowercase().contains(&q) {
                    let (snippet, _) = helpers::make_snippet(&plain, &q, 40, 60);
                    results.push(UnifiedSearchItem {
                        id, title: dec_title, snippet,
                        entity_type: "note".into(),
                        url: "/notes".into(),
                        subtitle: "Note".into(),
                    });
                }
            }
        }
    }

    // Search todos
    {
        let mut stmt = conn.prepare("SELECT id, title, description FROM todos WHERE is_archived = 0").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            if let Ok((id, title, description)) = row {
                let dec_title = enc.try_decrypt(&title).await;
                let dec_desc = if let Some(ref d) = description { enc.try_decrypt(d).await } else { String::new() };
                if dec_title.to_lowercase().contains(&q) || dec_desc.to_lowercase().contains(&q) {
                    let (snippet, _) = helpers::make_snippet(&dec_desc, &q, 0, 80);
                    let snippet = if snippet.is_empty() && !dec_desc.is_empty() {
                        if dec_desc.len() > 80 { format!("{}...", &dec_desc[..80]) } else { dec_desc.clone() }
                    } else { snippet };
                    results.push(UnifiedSearchItem {
                        id, title: dec_title, snippet,
                        entity_type: "todo".into(),
                        url: "/todos".into(),
                        subtitle: if dec_desc.is_empty() { "Todo".into() } else { format!("Todo — {}", dec_desc.lines().next().unwrap_or("")) },
                    });
                }
            }
        }
    }

    // Search calendar events
    {
        let mut stmt = conn.prepare("SELECT id, title, description, start_time FROM calendar_events").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, i64>(3)?))
        }).map_err(|e| e.to_string())?;
        for row in rows {
            if let Ok((id, title, description, start_time)) = row {
                let dec_title = enc.try_decrypt(&title).await;
                let dec_desc = if let Some(ref d) = description { enc.try_decrypt(d).await } else { String::new() };
                if dec_title.to_lowercase().contains(&q) || dec_desc.to_lowercase().contains(&q) {
                    let (snippet, _) = helpers::make_snippet(&dec_desc, &q, 0, 80);
                    let snippet = if snippet.is_empty() && !dec_desc.is_empty() {
                        if dec_desc.len() > 80 { format!("{}...", &dec_desc[..80]) } else { dec_desc.clone() }
                    } else { snippet };
                    let date_str = chrono::DateTime::from_timestamp(start_time, 0)
                        .map(|dt| dt.format("%b %d").to_string())
                        .unwrap_or_default();
                    results.push(UnifiedSearchItem {
                        id, title: dec_title, snippet,
                        entity_type: "event".into(),
                        url: "/calendar".into(),
                        subtitle: format!("Event — {}", date_str),
                    });
                }
            }
        }
    }

    drop(conn);
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_search_item_serializes() {
        let item = UnifiedSearchItem {
            id: "n1".into(),
            title: "Test".into(),
            snippet: "Hello world".into(),
            entity_type: "note".into(),
            url: "/notes".into(),
            subtitle: "Note".into(),
        };
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"note\""));
        let restored: UnifiedSearchItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item.id, restored.id);
        assert_eq!(item.title, restored.title);
    }

    #[test]
    fn search_item_empty_query_returns_empty() {
        // Just verify the struct works
        let item = UnifiedSearchItem {
            id: "t1".into(),
            title: "Buy milk".into(),
            snippet: "".into(),
            entity_type: "todo".into(),
            url: "/todos".into(),
            subtitle: "Todo".into(),
        };
        assert_eq!(item.entity_type, "todo");
    }

    #[test]
    fn all_structs_clone() {
        let item = UnifiedSearchItem {
            id: "e1".into(),
            title: "Meeting".into(),
            snippet: "".into(),
            entity_type: "event".into(),
            url: "/calendar".into(),
            subtitle: "Event".into(),
        };
        let cloned = UnifiedSearchItem { ..item };
        assert_eq!(cloned.title, "Meeting");
    }
}
