use crate::crypto::manager::EncryptionManager;
use crate::database::pool::DbPool;

/// Verify that the database is unlocked and the session has not expired.
/// If the session has expired, the encryption key is cleared to enforce the lock.
/// Also verifies the session HMAC to detect DB tampering.
pub async fn require_valid_session(pool: &DbPool, enc: &EncryptionManager) -> Result<(), String> {
    // First check if the encryption manager is locked
    if enc.is_locked().await {
        return Err("Database is locked. Unlock first.".to_string());
    }

    // Then check if the session has expired
    let conn = pool.get().await.map_err(|e| e.to_string())?;
    let unlocked_at_str: Result<String, _> = conn.query_row(
        "SELECT value FROM app_settings WHERE key = 'session_unlocked_at'",
        [],
        |r| r.get(0)
    );

    match unlocked_at_str {
        Ok(unlocked_at_str) => {
            // Verify HMAC to detect DB tampering
            let stored_hmac: Result<String, _> = conn.query_row(
                "SELECT value FROM app_settings WHERE key = 'session_hmac'",
                [], |r| r.get(0)
            );
            if let Ok(hmac) = stored_hmac {
                if let Ok(false) = enc.verify_session_hmac(&unlocked_at_str, &hmac).await {
                    // DB tampered — clear session immediately
                    drop(conn);
                    enc.clear_key().await;
                    enc.clear_session_secret().await;
                    return Err("Session tampered. Please unlock again.".to_string());
                }
            }

            let unlocked_at: i64 = unlocked_at_str.parse().map_err(|_| "Invalid timestamp".to_string())?;
            let timeout: i64 = conn.query_row(
                "SELECT COALESCE((SELECT value FROM app_settings WHERE key = 'session_timeout'), '15')",
                [],
                |r| r.get::<_, String>(0)
            ).map(|v| v.parse().unwrap_or(15)).unwrap_or(15);
            let elapsed = chrono::Utc::now().timestamp() - unlocked_at;
            if elapsed >= timeout * 60 {
                // Session expired — clear the encryption key to enforce the lock
                drop(conn);
                enc.clear_key().await;
                enc.clear_session_secret().await;
                return Err("Session expired. Please unlock the database again.".to_string());
            }
            Ok(())
        }
        // No session_unlocked_at row — allow (legacy support for existing data)
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}

/// Strip HTML tags from a string (simple tag removal).
pub fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for c in s.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ => if !in_tag { out.push(c); }
        }
    }
    out
}

/// Generate a search snippet around the first occurrence of `query` in `text`.
/// Returns (snippet, match_found).
pub fn make_snippet(text: &str, query: &str, context_before: usize, context_after: usize) -> (String, bool) {
    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();
    match lower_text.find(&lower_query) {
        Some(idx) => {
            let start = idx.saturating_sub(context_before);
            let end = (idx + lower_query.len() + context_after).min(text.len());
            let prefix = if start > 0 { "..." } else { "" };
            let suffix = if end < text.len() { "..." } else { "" };
            (format!("{}{}{}", prefix, &text[start..end], suffix), true)
        }
        None => (String::new(), false),
    }
}

/// Decrypt a note's title and content in-place.
pub async fn decrypt_note(enc: &EncryptionManager, note: &mut crate::commands::notes::Note) {
    note.title = enc.try_decrypt(&note.title).await;
    note.content = enc.try_decrypt(&note.content).await;
}

/// Decrypt a todo's title and description in-place.
pub async fn decrypt_todo(enc: &EncryptionManager, todo: &mut crate::commands::todos::Todo) {
    todo.title = enc.try_decrypt(&todo.title).await;
    if let Some(ref d) = todo.description.clone() {
        todo.description = Some(enc.try_decrypt(d).await);
    }
}

/// Decrypt a calendar event's title and description in-place.
pub async fn decrypt_event(enc: &EncryptionManager, event: &mut crate::commands::calendar::CalendarEvent) {
    event.title = enc.try_decrypt(&event.title).await;
    if let Some(ref d) = event.description.clone() {
        event.description = Some(enc.try_decrypt(d).await);
    }
}

/// Batch-decrypt a list of calendar events.
pub async fn decrypt_events(enc: &EncryptionManager, events: &mut [crate::commands::calendar::CalendarEvent]) {
    for e in events {
        decrypt_event(enc, e).await;
    }
}

/// Batch-decrypt a list of todos.
pub async fn decrypt_todos(enc: &EncryptionManager, todos: &mut [crate::commands::todos::Todo]) {
    for t in todos {
        decrypt_todo(enc, t).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<p>Hello <b>world</b></p>"), "Hello world");
    }

    #[test]
    fn strip_html_no_tags() {
        assert_eq!(strip_html("plain text"), "plain text");
    }

    #[test]
    fn strip_html_empty() {
        assert_eq!(strip_html(""), "");
    }

    #[test]
    fn strip_html_nested_tags() {
        assert_eq!(strip_html("<div><span>test</span></div>"), "test");
    }

    #[test]
    fn make_snippet_finds_match() {
        let (snippet, found) = make_snippet("Hello world, this is a test", "world", 5, 10);
        assert!(found);
        assert!(snippet.contains("world"));
    }

    #[test]
    fn make_snippet_no_match() {
        let (snippet, found) = make_snippet("Hello world", "xyz", 5, 10);
        assert!(!found);
        assert!(snippet.is_empty());
    }

    #[test]
    fn make_snippet_at_start() {
        let (snippet, found) = make_snippet("Hello world", "Hello", 5, 5);
        assert!(found);
        assert!(!snippet.starts_with("..."));
    }

    #[test]
    fn make_snippet_at_end() {
        let (snippet, found) = make_snippet("Hello world", "world", 5, 5);
        assert!(found);
        assert!(!snippet.ends_with("..."));
    }
}
