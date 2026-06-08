use std::sync::Arc;
use tauri::AppHandle;
use tauri::Manager;
use rusqlite::Connection;
use tokio::sync::Mutex;

pub struct DbPool(pub Arc<Mutex<Connection>>);

pub async fn init_pool(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;
    let db_path = app_dir.join("knowledge_base.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    let migrations = [
        include_str!("../../migrations/001_create_users.sql"),
        include_str!("../../migrations/002_create_notebooks.sql"),
        include_str!("../../migrations/003_create_notes.sql"),
        include_str!("../../migrations/004_create_todos.sql"),
        include_str!("../../migrations/005_create_calendar_events.sql"),
        include_str!("../../migrations/006_create_tags.sql"),
        include_str!("../../migrations/007_create_note_tags.sql"),
        include_str!("../../migrations/008_create_search_index.sql"),
        include_str!("../../migrations/009_create_encryption_keys.sql"),
        include_str!("../../migrations/010_create_sync_queue.sql"),
        include_str!("../../migrations/011_create_attachments.sql"),
        include_str!("../../migrations/012_create_app_settings.sql"),
        include_str!("../../migrations/013_create_device_info.sql"),
        include_str!("../../migrations/014_create_recurring_todos.sql"),
        include_str!("../../migrations/015_add_recurring_events.sql"),
    ];
    for sql in migrations {
        conn.execute_batch(sql)?;
    }
    app.manage(DbPool(Arc::new(Mutex::new(conn))));
    Ok(())
}

impl DbPool {
    pub async fn get(&self) -> Result<tokio::sync::MutexGuard<'_, Connection>, String> {
        Ok(self.0.lock().await)
    }
}
