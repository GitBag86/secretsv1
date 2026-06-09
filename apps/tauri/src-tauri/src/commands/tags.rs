use tauri::State;
use crate::database::pool::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Tag { pub id: String, pub user_id: String, pub name: String, pub color: String, pub created_at: i64 }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NoteTag { pub note_id: String, pub tag_id: String }

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct TodoTag { pub todo_id: String, pub tag_id: String }

#[tauri::command]
pub async fn list_tags(pool: State<'_, DbPool>) -> Result<Vec<Tag>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, user_id, name, color, created_at FROM tags ORDER BY name ASC").map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(Tag { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(t) = row { tags.push(t); } }
    Ok(tags)
}

#[tauri::command]
pub async fn create_tag(pool: State<'_, DbPool>, name: String, color: Option<String>) -> Result<Tag, String> {
    let user_id = "local-user".to_string();
    let c = color.unwrap_or_else(|| "#6b7280".into());
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    // Check if tag with this name already exists
    let existing: Result<Tag, _> = conn.query_row(
        "SELECT id, user_id, name, color, created_at FROM tags WHERE user_id = ?1 AND name = ?2",
        [&user_id, &name],
        |r| Ok(Tag { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    );
    if let Ok(tag) = existing {
        return Ok(tag);
    }
    // Create new tag
    let id = uuid::Uuid::new_v4().to_string();
    conn.execute("INSERT INTO tags (id, user_id, name, color, created_at) VALUES (?1,?2,?3,?4,?5)", (&id, &user_id, &name, &c, now)).map_err(|e| e.to_string())?;
    Ok(Tag { id, user_id, name, color: c, created_at: now })
}

#[tauri::command]
pub async fn update_tag(pool: State<'_, DbPool>, id: String, name: Option<String>, color: Option<String>) -> Result<Tag, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let existing: Tag = conn.query_row("SELECT id, user_id, name, color, created_at FROM tags WHERE id = ?1", [&id], |r| {
        Ok(Tag { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    }).map_err(|e| e.to_string())?;
    let n = name.unwrap_or(existing.name);
    let c = color.unwrap_or(existing.color);
    conn.execute("UPDATE tags SET name=?1, color=?2 WHERE id=?3", (&n, &c, &id)).map_err(|e| e.to_string())?;
    Ok(Tag { id, user_id: existing.user_id, name: n, color: c, created_at: existing.created_at })
}

#[tauri::command]
pub async fn delete_tag(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM note_tags WHERE tag_id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM tags WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_note_tags(pool: State<'_, DbPool>, note_id: String) -> Result<Vec<Tag>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.user_id, t.name, t.color, t.created_at FROM tags t \
         INNER JOIN note_tags nt ON nt.tag_id = t.id WHERE nt.note_id = ?1 ORDER BY t.name"
    ).map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    let rows = stmt.query_map([&note_id], |r| {
        Ok(Tag { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(t) = row { tags.push(t); } }
    Ok(tags)
}

#[tauri::command]
pub async fn set_note_tags(pool: State<'_, DbPool>, note_id: String, tag_ids: Vec<String>) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM note_tags WHERE note_id = ?1", [&note_id]).map_err(|e| e.to_string())?;
    for tag_id in &tag_ids {
        conn.execute("INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)", (&note_id, tag_id)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_notes_with_tag(pool: State<'_, DbPool>, tag_id: String) -> Result<Vec<String>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT note_id FROM note_tags WHERE tag_id = ?1").map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    let rows = stmt.query_map([&tag_id], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(id) = row { ids.push(id); } }
    Ok(ids)
}

// --- Todo Tags ---

#[tauri::command]
pub async fn get_todo_tags(pool: State<'_, DbPool>, todo_id: String) -> Result<Vec<Tag>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT t.id, t.user_id, t.name, t.color, t.created_at FROM tags t \
         INNER JOIN todo_tags tt ON tt.tag_id = t.id WHERE tt.todo_id = ?1 ORDER BY t.name"
    ).map_err(|e| e.to_string())?;
    let mut tags = Vec::new();
    let rows = stmt.query_map([&todo_id], |r| {
        Ok(Tag { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(t) = row { tags.push(t); } }
    Ok(tags)
}

#[tauri::command]
pub async fn set_todo_tags(pool: State<'_, DbPool>, todo_id: String, tag_ids: Vec<String>) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM todo_tags WHERE todo_id = ?1", [&todo_id]).map_err(|e| e.to_string())?;
    for tag_id in &tag_ids {
        conn.execute("INSERT OR IGNORE INTO todo_tags (todo_id, tag_id) VALUES (?1, ?2)", (&todo_id, tag_id)).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_todos_with_tag(pool: State<'_, DbPool>, tag_id: String) -> Result<Vec<String>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT todo_id FROM todo_tags WHERE tag_id = ?1").map_err(|e| e.to_string())?;
    let mut ids = Vec::new();
    let rows = stmt.query_map([&tag_id], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(id) = row { ids.push(id); } }
    Ok(ids)
}

#[tauri::command]
pub async fn list_all_todo_tags(pool: State<'_, DbPool>) -> Result<Vec<TodoTag>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT todo_id, tag_id FROM todo_tags").map_err(|e| e.to_string())?;
    let mut todo_tags = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(TodoTag { todo_id: r.get(0)?, tag_id: r.get(1)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(tt) = row { todo_tags.push(tt); } }
    Ok(todo_tags)
}

#[tauri::command]
pub async fn list_all_note_tags(pool: State<'_, DbPool>) -> Result<Vec<NoteTag>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT note_id, tag_id FROM note_tags").map_err(|e| e.to_string())?;
    let mut note_tags = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(NoteTag { note_id: r.get(0)?, tag_id: r.get(1)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(nt) = row { note_tags.push(nt); } }
    Ok(note_tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_note_tag() -> NoteTag {
        NoteTag { note_id: "note-1".into(), tag_id: "tag-1".into() }
    }

    fn make_todo_tag() -> TodoTag {
        TodoTag { todo_id: "todo-1".into(), tag_id: "tag-1".into() }
    }

    #[test]
    fn note_tag_serialize_roundtrip() {
        let nt = make_note_tag();
        let json = serde_json::to_string(&nt).unwrap();
        let restored: NoteTag = serde_json::from_str(&json).unwrap();
        assert_eq!(nt, restored);
    }

    #[test]
    fn note_tag_fields() {
        let nt = make_note_tag();
        assert_eq!(nt.note_id, "note-1");
        assert_eq!(nt.tag_id, "tag-1");
    }

    #[test]
    fn todo_tag_serialize_roundtrip() {
        let tt = make_todo_tag();
        let json = serde_json::to_string(&tt).unwrap();
        let restored: TodoTag = serde_json::from_str(&json).unwrap();
        assert_eq!(tt, restored);
    }

    #[test]
    fn todo_tag_fields() {
        let tt = make_todo_tag();
        assert_eq!(tt.todo_id, "todo-1");
        assert_eq!(tt.tag_id, "tag-1");
    }

    #[test]
    fn clone_produces_equal() {
        let nt = make_note_tag();
        let tt = make_todo_tag();
        assert_eq!(nt, nt.clone());
        assert_eq!(tt, tt.clone());
    }
}

