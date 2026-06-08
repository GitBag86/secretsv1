mod commands;
mod crypto;
mod database;
mod sync;

use tauri::Manager;
use crate::crypto::manager::EncryptionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_sql::Builder::default().build())
        .setup(|app| {
            app.manage(EncryptionManager::new());
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = database::pool::init_pool(&app_handle).await {
                    log::error!("Failed to initialize database: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::register,
            commands::auth::login,
            commands::auth::unlock_database,
            commands::auth::lock_database,
            commands::auth::check_session,
            commands::auth::get_session_timeout,
            commands::auth::set_session_timeout,
            commands::auth::refresh_session,
            commands::auth::logout,
            commands::notes::list_notes,
            commands::notes::get_note,
            commands::notes::create_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::search_notes,
            commands::todos::list_todos,
            commands::todos::create_todo,
            commands::todos::update_todo,
            commands::todos::delete_todo,
            commands::todos::bulk_update_todos,
            commands::todos::bulk_delete_todos,
            commands::calendar::list_calendar_events,
            commands::calendar::create_calendar_event,
            commands::calendar::update_calendar_event,
            commands::calendar::delete_calendar_event,
            commands::encryption::set_master_password,
            commands::encryption::get_encryption_salt,
            commands::encryption::rotate_encryption_key,
            commands::notebooks::list_notebooks,
            commands::notebooks::create_notebook,
            commands::notebooks::update_notebook,
            commands::notebooks::delete_notebook,
            commands::sync::sync_push,
            commands::sync::sync_pull,
            commands::sync::sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
