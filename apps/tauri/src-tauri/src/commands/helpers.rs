use crate::crypto::manager::EncryptionManager;

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
