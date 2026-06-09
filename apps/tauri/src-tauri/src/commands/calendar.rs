use tauri::State;
use crate::database::pool::DbPool;
use crate::crypto::manager::EncryptionManager;
use crate::sync::enqueue_sync;
use crate::commands::helpers;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct CalendarEvent { pub id: String, pub user_id: String, pub title: String, pub description: Option<String>, pub start_time: i64, pub end_time: i64, pub all_day: bool, pub color: String, pub rrule: Option<String>, pub parent_event_id: Option<String>, pub created_at: i64, pub updated_at: i64 }

fn validate_event_title(title: &str) -> Result<(), String> {
    if title.is_empty() { return Err("Title cannot be empty".into()); }
    if title.len() > 10000 { return Err("Title too long (max 10000 chars)".into()); }
    if title.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err("Title contains invalid control characters".into());
    }
    Ok(())
}

fn validate_event_description(desc: &str) -> Result<(), String> {
    if desc.len() > 100_000 { return Err("Description too long (max 100KB)".into()); }
    Ok(())
}

#[tauri::command]
pub async fn list_calendar_events(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>) -> Result<Vec<CalendarEvent>, String> {
    let mut events = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare("SELECT id, user_id, title, description, start_time, end_time, all_day, color, rrule, parent_event_id, created_at, updated_at FROM calendar_events ORDER BY start_time ASC").map_err(|e| e.to_string())?;
        let mut result = Vec::new();
        let rows = stmt.query_map([], |r| {
            Ok(CalendarEvent { id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?, start_time: r.get(4)?, end_time: r.get(5)?, all_day: r.get::<_, i64>(6)? != 0, color: r.get(7)?, rrule: r.get(8)?, parent_event_id: r.get(9)?, created_at: r.get(10)?, updated_at: r.get(11)? })
        }).map_err(|e| e.to_string())?;
        for row in rows { if let Ok(e) = row { result.push(e); } }
        result
    };
    helpers::decrypt_events(&enc, &mut events).await;
    Ok(events)
}

#[tauri::command]
pub async fn create_calendar_event(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, title: String, start_time: i64, end_time: i64, description: Option<String>, all_day: Option<bool>, color: Option<String>, rrule: Option<String>) -> Result<CalendarEvent, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    validate_event_title(&title)?;
    if let Some(ref d) = description { validate_event_description(d)?; }
    if start_time >= end_time { return Err("start_time must be before end_time".into()); }
    let id = uuid::Uuid::new_v4().to_string();
    let user_id = "local-user".to_string();
    let ad = all_day.unwrap_or(false);
    let c = color.unwrap_or_else(|| "#3b82f6".into());
    let et = enc.encrypt_or_pass(&title).await.map_err(|e| e.to_string())?;
    let ed = if let Some(ref d) = description { Some(enc.encrypt_or_pass(d).await.map_err(|e| e.to_string())?) } else { None };
    let now = chrono::Utc::now().timestamp();
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("INSERT INTO calendar_events (id, user_id, title, description, start_time, end_time, all_day, color, rrule, created_at, updated_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)", (&id, &user_id, &et, &ed, start_time, end_time, ad as i64, &c, &rrule, now, now)).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({"id": &id, "title": &title, "description": &description, "start_time": start_time, "end_time": end_time, "all_day": ad, "color": &c, "rrule": &rrule, "created_at": now, "updated_at": now});
    enqueue_sync(&conn, "event", &id, "create", Some(&payload.to_string())).ok();
    drop(conn);
    Ok(CalendarEvent { id, user_id, title, description, start_time, end_time, all_day: ad, color: c, rrule, parent_event_id: None, created_at: now, updated_at: now })
}

#[tauri::command]
pub async fn update_calendar_event(pool: State<'_, DbPool>, enc: State<'_, EncryptionManager>, id: String, title: Option<String>, description: Option<String>, start_time: Option<i64>, end_time: Option<i64>, all_day: Option<bool>, color: Option<String>, rrule: Option<String>) -> Result<CalendarEvent, String> {
    helpers::require_valid_session(&pool, &enc).await?;
    if let Some(ref t) = title { validate_event_title(t)?; }
    if let Some(ref d) = description { validate_event_description(d)?; }
    let (existing, conn) = {
        let conn = pool.get().await.map_err(|e| e.to_string())?;
        let existing = conn.query_row("SELECT id, user_id, title, description, start_time, end_time, all_day, color, rrule, parent_event_id, created_at, updated_at FROM calendar_events WHERE id = ?1", [&id], |r| {
            Ok(CalendarEvent { id: r.get(0)?, user_id: r.get(1)?, title: r.get(2)?, description: r.get(3)?, start_time: r.get(4)?, end_time: r.get(5)?, all_day: r.get::<_, i64>(6)? != 0, color: r.get(7)?, rrule: r.get(8)?, parent_event_id: r.get(9)?, created_at: r.get(10)?, updated_at: r.get(11)? })
        }).map_err(|e| e.to_string())?;
        (existing, conn)
    };
    let stored_t = if let Some(ref nt) = title { enc.encrypt_or_pass(nt).await.map_err(|e| e.to_string())? } else { existing.title.clone() };
    let stored_d = if let Some(ref nd) = description { Some(enc.encrypt_or_pass(nd).await.map_err(|e| e.to_string())?) } else { existing.description.clone() };
    let resp_t = enc.try_decrypt(&stored_t).await;
    let resp_d = if let Some(ref d) = stored_d { Some(enc.try_decrypt(d).await) } else { None };
    let st = start_time.unwrap_or(existing.start_time);
    let et = end_time.unwrap_or(existing.end_time);
    let ad = all_day.unwrap_or(existing.all_day);
    let c = color.unwrap_or(existing.color);
    let rr = rrule.or(existing.rrule);
    let now = chrono::Utc::now().timestamp();
    conn.execute("UPDATE calendar_events SET title=?1, description=?2, start_time=?3, end_time=?4, all_day=?5, color=?6, rrule=?7, updated_at=?8 WHERE id=?9", (&stored_t, &stored_d, st, et, ad as i64, &c, &rr, now, &id)).map_err(|e| e.to_string())?;
    let payload = serde_json::json!({"id": &id, "title": &resp_t, "description": &resp_d, "start_time": st, "end_time": et, "all_day": ad, "color": &c, "rrule": &rr, "updated_at": now});
    enqueue_sync(&conn, "event", &id, "update", Some(&payload.to_string())).ok();
    drop(conn);
    Ok(CalendarEvent { id, user_id: existing.user_id, title: resp_t, description: resp_d, start_time: st, end_time: et, all_day: ad, color: c, rrule: rr, parent_event_id: existing.parent_event_id, created_at: existing.created_at, updated_at: now })
}

#[tauri::command]
pub async fn delete_calendar_event(pool: State<'_, DbPool>, id: String) -> Result<(), String> {
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM calendar_events WHERE id = ?1", [&id]).map_err(|e| e.to_string())?;
    enqueue_sync(&conn, "event", &id, "delete", None).ok();
    drop(conn);
    Ok(())
}
