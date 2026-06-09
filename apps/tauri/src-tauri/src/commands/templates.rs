use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use crate::sync::enqueue_sync;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Template {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub content: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[tauri::command]
pub async fn list_templates(pool: State<'_, DbPool>) -> Result<Vec<Template>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, user_id, name, content, created_at, updated_at FROM templates ORDER BY name ASC").map_err(|e| e.to_string())?;
    let mut templates = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(Template { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, content: r.get(3)?, created_at: r.get(4)?, updated_at: r.get(5)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(t) = row { templates.push(t); } }
    Ok(templates)
}

#[tauri::command]
pub async fn create_template(pool: State<'_, DbPool>, name: String, content: String) -> Result<Template, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = "local-user".to_string();
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO templates (id, user_id, name, content, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6)",
        (&id, &user_id, &name, &content, now, now)).map_err(|e| e.to_string())?;
    drop(conn);
    Ok(Template { id, user_id, name, content, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn delete_template(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM templates WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    drop(conn);
    Ok(())
}

#[tauri::command]
pub async fn create_note_from_template(
    pool: State<'_, DbPool>,
    enc: State<'_, EncryptionManager>,
    template_id: String,
    title: Option<String>,
    notebook_id: Option<String>,
) -> Result<crate::commands::notes::Note, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let tmpl: Template = conn.query_row("SELECT id, user_id, name, content, created_at, updated_at FROM templates WHERE id = ?1", [&template_id], |r| {
        Ok(Template { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, content: r.get(3)?, created_at: r.get(4)?, updated_at: r.get(5)? })
    }).map_err(|_| "Template not found".to_string())?;
    drop(conn);

    let note_title = title.unwrap_or_else(|| format!("From: {}", tmpl.name));
    crate::commands::notes::create_note(pool, enc, note_title, tmpl.content, notebook_id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_template() -> Template {
        Template {
            id: "tmpl-1".into(),
            user_id: "u1".into(),
            name: "Meeting Notes".into(),
            content: "<h1>Meeting</h1><p>Date: </p><ul><li></li></ul>".into(),
            created_at: 1700000000,
            updated_at: 1700000000,
        }
    }

    #[test]
    fn template_serialize_roundtrip() {
        let t = make_template();
        let json = serde_json::to_string(&t).unwrap();
        let restored: Template = serde_json::from_str(&json).unwrap();
        assert_eq!(t.id, restored.id);
        assert_eq!(t.name, restored.name);
    }

    #[test]
    fn template_fields() {
        let t = make_template();
        assert_eq!(t.name, "Meeting Notes");
        assert!(t.content.contains("<h1>Meeting</h1>"));
    }

    #[test]
    fn template_clone() {
        let t = make_template();
        assert_eq!(t, t.clone());
    }
}
