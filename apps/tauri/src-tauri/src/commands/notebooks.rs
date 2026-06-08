use tauri::State;
use crate::database::pool::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Notebook { pub id: String, pub user_id: String, pub name: String, pub color: String, pub sort_order: i64, pub created_at: i64, pub updated_at: i64 }

#[tauri::command]
pub async fn list_notebooks(pool: State<'_, DbPool>) -> Result<Vec<Notebook>, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare("SELECT id, user_id, name, color, sort_order, created_at, updated_at FROM notebooks ORDER BY sort_order ASC, name ASC").map_err(|e| e.to_string())?;
    let mut notebooks = Vec::new();
    let rows = stmt.query_map([], |r| {
        Ok(Notebook { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, sort_order: r.get(4)?, created_at: r.get(5)?, updated_at: r.get(6)? })
    }).map_err(|e| e.to_string())?;
    for row in rows { if let Ok(n) = row { notebooks.push(n); } }
    Ok(notebooks)
}

#[tauri::command]
pub async fn create_notebook(pool: State<'_, DbPool>, name: String, color: Option<String>) -> Result<Notebook, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = "local-user".to_string();
    let c = color.unwrap_or_else(|| "#3b82f6".into());
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let max_order: i64 = conn.query_row("SELECT COALESCE(MAX(sort_order), -1) FROM notebooks WHERE user_id = ?1", [&user_id], |r| r.get(0)).unwrap_or(0);
    conn.execute("INSERT INTO notebooks (id, user_id, name, color, sort_order, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7)", (&id, &user_id, &name, &c, max_order + 1, now, now)).map_err(|e| e.to_string())?;
    Ok(Notebook { id, user_id, name, color: c, sort_order: max_order + 1, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn update_notebook(pool: State<'_, DbPool>, id: String, name: Option<String>, color: Option<String>, sort_order: Option<i64>) -> Result<Notebook, String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let existing: Notebook = conn.query_row("SELECT id, user_id, name, color, sort_order, created_at, updated_at FROM notebooks WHERE id = ?1", [&id], |r| {
        Ok(Notebook { id: r.get(0)?, user_id: r.get(1)?, name: r.get(2)?, color: r.get(3)?, sort_order: r.get(4)?, created_at: r.get(5)?, updated_at: r.get(6)? })
    }).map_err(|e| e.to_string())?;
    let n = name.unwrap_or(existing.name);
    let c = color.unwrap_or(existing.color);
    let so = sort_order.unwrap_or(existing.sort_order);
    let now = chrono::Utc::now().timestamp();
    conn.execute("UPDATE notebooks SET name=?1, color=?2, sort_order=?3, updated_at=?4 WHERE id=?5", (&n, &c, so, now, &id)).map_err(|e| e.to_string())?;
    Ok(Notebook { id, user_id: existing.user_id, name: n, color: c, sort_order: so, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub async fn delete_notebook(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("UPDATE notes SET notebook_id = NULL WHERE notebook_id = ?1", [&id]).map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notebooks WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    Ok(())
}
