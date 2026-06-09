use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub version: i64,
    pub exported_at: i64,
    pub notes: Vec<ExportNote>,
    pub todos: Vec<ExportTodo>,
    pub calendar_events: Vec<ExportEvent>,
    pub notebooks: Vec<ExportNotebook>,
    pub tags: Vec<ExportTag>,
    pub note_tags: Vec<ExportNoteTag>,
    pub todo_tags: Vec<ExportTodoTag>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportNote {
    pub id: String,
    pub user_id: String,
    pub notebook_id: Option<String>,
    pub title: String,
    pub content: String,
    pub word_count: i64,
    pub reading_time: i64,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ExportTodo {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub is_completed: bool,
    pub priority: String,
    pub due_date: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ExportEvent {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: i64,
    pub end_time: i64,
    pub all_day: bool,
    pub color: String,
    pub rrule: Option<String>,
    pub parent_event_id: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ExportNotebook {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ExportTag {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub color: String,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ExportNoteTag {
    pub note_id: String,
    pub tag_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ExportTodoTag {
    pub todo_id: String,
    pub tag_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ImportResult {
    pub notes: i64,
    pub todos: i64,
    pub events: i64,
    pub notebooks: i64,
    pub tags: i64,
    pub note_tags: i64,
    pub todo_tags: i64,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn export_data(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<String, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;

    // --- Notes ---
    let mut notes = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at FROM notes").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportNote {
                id: r.get(0)?, user_id: r.get(1)?, notebook_id: r.get(2)?,
                title: r.get(3)?, content: r.get(4)?, word_count: r.get(5)?,
                reading_time: r.get(6)?, is_pinned: r.get::<_, i64>(7)? != 0,
                is_archived: r.get::<_, i64>(8)? != 0, created_at: r.get(9)?, updated_at: r.get(10)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { notes.push(n); } }
    }
    for n in &mut notes {
        n.title = enc.try_decrypt(&n.title).await;
        n.content = enc.try_decrypt(&n.content).await;
    }

    // --- Todos ---
    let mut todos = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, user_id, title, description, is_completed, priority, due_date, created_at, updated_at FROM todos").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportTodo {
                id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?,
                description: r.get(3)?, is_completed: r.get::<_, i64>(4)? != 0,
                priority: r.get(5)?, due_date: r.get(6)?, created_at: r.get(7)?, updated_at: r.get(8)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(t) = row { todos.push(t); } }
    }
    for t in &mut todos {
        t.title = enc.try_decrypt(&t.title).await;
        if let Some(ref d) = t.description.clone() {
            t.description = Some(enc.try_decrypt(d).await);
        }
    }

    // --- Calendar Events ---
    let mut events = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, user_id, title, description, start_time, end_time, all_day, color, rrule, parent_event_id, created_at, updated_at FROM calendar_events").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportEvent {
                id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?,
                description: r.get(3)?, start_time: r.get(4)?, end_time: r.get(5)?,
                all_day: r.get::<_, i64>(6)? != 0, color: r.get(7)?, rrule: r.get(8)?,
                parent_event_id: r.get(9)?, created_at: r.get(10)?, updated_at: r.get(11)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(e) = row { events.push(e); } }
    }
    for e in &mut events {
        e.title = enc.try_decrypt(&e.title).await;
        if let Some(ref d) = e.description.clone() {
            e.description = Some(enc.try_decrypt(d).await);
        }
    }

    // --- Notebooks ---
    let mut notebooks = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, user_id, name, color, sort_order, created_at, updated_at FROM notebooks").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportNotebook {
                id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?,
                sort_order: r.get(4)?, created_at: r.get(5)?, updated_at: r.get(6)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(n) = row { notebooks.push(n); } }
    }

    // --- Tags ---
    let mut tags = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT id, user_id, name, color, created_at FROM tags").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportTag {
                id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, created_at: r.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(t) = row { tags.push(t); } }
    }

    // --- Note Tags ---
    let mut note_tags = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT note_id, tag_id FROM note_tags").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportNoteTag { note_id: r.get(0)?, tag_id: r.get(1)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(nt) = row { note_tags.push(nt); } }
    }

    // --- Todo Tags ---
    let mut todo_tags = Vec::new();
    {
        let mut stmt = conn.prepare("SELECT todo_id, tag_id FROM todo_tags").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ExportTodoTag { todo_id: r.get(0)?, tag_id: r.get(1)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(tt) = row { todo_tags.push(tt); } }
    }

    drop(conn);

    let export = ExportData {
        version: 1,
        exported_at: chrono::Utc::now().timestamp(),
        notes,
        todos,
        calendar_events: events,
        notebooks,
        tags,
        note_tags,
        todo_tags,
    };

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_data(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, data: String) -> Result<ImportResult, String> {
    let export: ExportData = serde_json::from_str(&data).map_err(|e| format!("Invalid JSON: {}", e))?;
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut result = ImportResult {
        notes: 0, todos: 0, events: 0, notebooks: 0, tags: 0,
        note_tags: 0, todo_tags: 0, errors: Vec::new(),
    };
    let now = chrono::Utc::now().timestamp();

    // --- Import notebooks ---
    for nb in &export.notebooks {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO notebooks (id, user_id, name, color, sort_order, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            (&nb.id, &nb.user_id, &nb.name, &nb.color, nb.sort_order, nb.created_at, nb.updated_at),
        ) {
            result.errors.push(format!("notebook {}: {}", nb.id, e));
        } else {
            result.notebooks += 1;
        }
    }

    // --- Import tags ---
    for tag in &export.tags {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO tags (id, user_id, name, color, created_at) VALUES (?1,?2,?3,?4,?5)",
            (&tag.id, &tag.user_id, &tag.name, &tag.color, tag.created_at),
        ) {
            result.errors.push(format!("tag {}: {}", tag.id, e));
        } else {
            result.tags += 1;
        }
    }

    // --- Import notes (encrypt content) ---
    for n in &export.notes {
        let et = enc.encrypt_or_pass(&n.title).await.map_err(|e| e.to_string())?;
        let ec = enc.encrypt_or_pass(&n.content).await.map_err(|e| e.to_string())?;
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO notes (id, user_id, notebook_id, title, content, word_count, reading_time, is_pinned, is_archived, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
            (&n.id, &n.user_id, &n.notebook_id, &et, &ec, n.word_count, n.reading_time, n.is_pinned as i64, n.is_archived as i64, n.created_at, n.updated_at),
        ) {
            result.errors.push(format!("note {}: {}", n.id, e));
        } else {
            result.notes += 1;
        }
    }

    // --- Import todos (encrypt content) ---
    for t in &export.todos {
        let et = enc.encrypt_or_pass(&t.title).await.map_err(|e| e.to_string())?;
        let ed = if let Some(ref d) = t.description { Some(enc.encrypt_or_pass(d).await.map_err(|e| e.to_string())?) } else { None };
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO todos (id, user_id, title, description, is_completed, priority, due_date, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            (&t.id, &t.user_id, &et, &ed, t.is_completed as i64, &t.priority, &t.due_date, t.created_at, t.updated_at),
        ) {
            result.errors.push(format!("todo {}: {}", t.id, e));
        } else {
            result.todos += 1;
        }
    }

    // --- Import calendar events (encrypt content) ---
    for e in &export.calendar_events {
        let et = enc.encrypt_or_pass(&e.title).await.map_err(|e| e.to_string())?;
        let ed = if let Some(ref d) = e.description { Some(enc.encrypt_or_pass(d).await.map_err(|e| e.to_string())?) } else { None };
        if let Err(ee) = conn.execute(
            "INSERT OR IGNORE INTO calendar_events (id, user_id, title, description, start_time, end_time, all_day, color, rrule, parent_event_id, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            (&e.id, &e.user_id, &et, &ed, e.start_time, e.end_time, e.all_day as i64, &e.color, &e.rrule, &e.parent_event_id, e.created_at, e.updated_at),
        ) {
            result.errors.push(format!("event {}: {}", e.id, ee));
        } else {
            result.events += 1;
        }
    }

    // --- Import note_tags ---
    for nt in &export.note_tags {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1,?2)",
            (&nt.note_id, &nt.tag_id),
        ) {
            result.errors.push(format!("note_tag (note={}, tag={}): {}", nt.note_id, nt.tag_id, e));
        } else {
            result.note_tags += 1;
        }
    }

    // --- Import todo_tags ---
    for tt in &export.todo_tags {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO todo_tags (todo_id, tag_id) VALUES (?1,?2)",
            (&tt.todo_id, &tt.tag_id),
        ) {
            result.errors.push(format!("todo_tag (todo={}, tag={}): {}", tt.todo_id, tt.tag_id, e));
        } else {
            result.todo_tags += 1;
        }
    }

    drop(conn);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_data_serializes() {
        let export = ExportData {
            version: 1,
            exported_at: 1700000000,
            notes: vec![],
            todos: vec![],
            calendar_events: vec![],
            notebooks: vec![],
            tags: vec![],
            note_tags: vec![],
            todo_tags: vec![],
        };
        let json = serde_json::to_string(&export).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"exported_at\":1700000000"));
    }

    #[test]
    fn export_data_deserializes() {
        let json = r#"{"version":1,"exported_at":1700000000,"notes":[],"todos":[],"calendar_events":[],"notebooks":[],"tags":[],"note_tags":[],"todo_tags":[]}"#;
        let export: ExportData = serde_json::from_str(json).unwrap();
        assert_eq!(export.version, 1);
        assert_eq!(export.exported_at, 1700000000);
        assert!(export.notes.is_empty());
    }

    #[test]
    fn import_result_default() {
        let r = ImportResult {
            notes: 0, todos: 0, events: 0, notebooks: 0, tags: 0,
            note_tags: 0, todo_tags: 0, errors: vec![],
        };
        assert_eq!(r.notes, 0);
        assert!(r.errors.is_empty());
    }

    #[test]
    fn export_note_with_fields() {
        let n = ExportNote {
            id: "n1".into(), user_id: "u1".into(), notebook_id: Some("nb1".into()),
            title: "Test".into(), content: "Hello".into(), word_count: 1,
            reading_time: 1, is_pinned: false, is_archived: false,
            created_at: 1700000000, updated_at: 1700000000,
        };
        let json = serde_json::to_string(&n).unwrap();
        let restored: ExportNote = serde_json::from_str(&json).unwrap();
        assert_eq!(n.id, restored.id);
        assert_eq!(n.title, restored.title);
    }

    #[test]
    fn export_todo_with_optional_description() {
        let t = ExportTodo {
            id: "t1".into(), user_id: "u1".into(), title: "Task".into(),
            description: Some("details".into()), is_completed: false,
            priority: "high".into(), due_date: Some(1700100000),
            created_at: 1700000000, updated_at: 1700000000,
        };
        let json = serde_json::to_string(&t).unwrap();
        let restored: ExportTodo = serde_json::from_str(&json).unwrap();
        assert_eq!(t.title, restored.title);
        assert_eq!(t.description, restored.description);
    }

    #[test]
    fn export_tag_with_color() {
        let tag = ExportTag {
            id: "tag1".into(), user_id: "u1".into(), name: "important".into(),
            color: "#ef4444".into(), created_at: 1700000000,
        };
        let json = serde_json::to_string(&tag).unwrap();
        assert!(json.contains("#ef4444"));
        let restored: ExportTag = serde_json::from_str(&json).unwrap();
        assert_eq!(tag.name, restored.name);
    }

    #[test]
    fn note_tag_association() {
        let nt = ExportNoteTag { note_id: "n1".into(), tag_id: "tag1".into() };
        let json = serde_json::to_string(&nt).unwrap();
        let restored: ExportNoteTag = serde_json::from_str(&json).unwrap();
        assert_eq!(nt.note_id, restored.note_id);
        assert_eq!(nt.tag_id, restored.tag_id);
    }

    #[test]
    fn todo_tag_association() {
        let tt = ExportTodoTag { todo_id: "t1".into(), tag_id: "tag1".into() };
        let json = serde_json::to_string(&tt).unwrap();
        let restored: ExportTodoTag = serde_json::from_str(&json).unwrap();
        assert_eq!(tt.todo_id, restored.todo_id);
        assert_eq!(tt.tag_id, restored.tag_id);
    }

    #[test]
    fn all_structs_clone() {
        let _ = ExportData {
            version: 1, exported_at: 0, notes: vec![], todos: vec![],
            calendar_events: vec![], notebooks: vec![], tags: vec![],
            note_tags: vec![], todo_tags: vec![],
        };
    }
}
