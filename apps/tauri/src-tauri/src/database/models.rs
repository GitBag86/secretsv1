#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Note {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notebook {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Todo {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CalendarEvent {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tag {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub color: String,
    pub created_at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_user() -> User {
        User { id: "u1".into(), email: "test@example.com".into(), name: Some("Test".into()), created_at: 1700000000, updated_at: 1700000000 }
    }

    fn make_note() -> Note {
        Note { id: "n1".into(), user_id: "u1".into(), notebook_id: Some("nb1".into()), title: "My Note".into(), content: "Hello world".into(), word_count: 2, reading_time: 1, is_pinned: false, is_archived: false, created_at: 1700000000, updated_at: 1700000000 }
    }

    fn make_todo() -> Todo {
        Todo { id: "t1".into(), user_id: "u1".into(), title: "My Todo".into(), description: Some("desc".into()), is_completed: false, priority: "high".into(), due_date: Some(1700100000), created_at: 1700000000, updated_at: 1700000000 }
    }

    fn make_calendar_event() -> CalendarEvent {
        CalendarEvent { id: "e1".into(), user_id: "u1".into(), title: "Meeting".into(), description: Some("Daily standup".into()), start_time: 1700000000, end_time: 1700003600, all_day: false, color: "#3b82f6".into(), rrule: None, parent_event_id: None, created_at: 1700000000, updated_at: 1700000000 }
    }

    fn make_tag() -> Tag {
        Tag { id: "tag1".into(), user_id: "u1".into(), name: "important".into(), color: "#ef4444".into(), created_at: 1700000000 }
    }

    // --- User tests ---

    #[test]
    fn user_serializes() {
        let user = make_user();
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn user_deserializes() {
        let json = r#"{"id":"u1","email":"test@example.com","name":"Test","created_at":1700000000,"updated_at":1700000000}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.id, "u1");
    }

    #[test]
    fn user_with_null_name() {
        let json = r#"{"id":"u1","email":"a@b.com","name":null,"created_at":0,"updated_at":0}"#;
        let user: User = serde_json::from_str(json).unwrap();
        assert!(user.name.is_none());
    }

    // --- Note tests ---

    #[test]
    fn note_serialize_roundtrip() {
        let note = make_note();
        let json = serde_json::to_string(&note).unwrap();
        let restored: Note = serde_json::from_str(&json).unwrap();
        assert_eq!(note, restored);
    }

    #[test]
    fn note_without_notebook() {
        let mut note = make_note();
        note.notebook_id = None;
        let json = serde_json::to_string(&note).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn note_pinned_and_archived() {
        let mut note = make_note();
        note.is_pinned = true;
        note.is_archived = true;
        assert!(note.is_pinned);
        assert!(note.is_archived);
    }

    // --- Todo tests ---

    #[test]
    fn todo_serialize_roundtrip() {
        let todo = make_todo();
        let json = serde_json::to_string(&todo).unwrap();
        let restored: Todo = serde_json::from_str(&json).unwrap();
        assert_eq!(todo, restored);
    }

    #[test]
    fn todo_completed() {
        let mut todo = make_todo();
        todo.is_completed = true;
        assert!(todo.is_completed);
    }

    #[test]
    fn todo_without_due_date() {
        let mut todo = make_todo();
        todo.due_date = None;
        assert!(todo.due_date.is_none());
    }

    #[test]
    fn todo_priority_values() {
        for priority in ["low", "medium", "high"] {
            let mut todo = make_todo();
            todo.priority = priority.into();
            assert_eq!(todo.priority, priority);
        }
    }

    // --- CalendarEvent tests ---

    #[test]
    fn calendar_event_serialize_roundtrip() {
        let event = make_calendar_event();
        let json = serde_json::to_string(&event).unwrap();
        let restored: CalendarEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(event, restored);
    }

    #[test]
    fn all_day_event() {
        let mut event = make_calendar_event();
        event.all_day = true;
        assert!(event.all_day);
    }

    #[test]
    fn event_start_before_end() {
        let event = make_calendar_event();
        assert!(event.start_time < event.end_time);
    }

    // --- Tag tests ---

    #[test]
    fn tag_serialize_roundtrip() {
        let tag = make_tag();
        let json = serde_json::to_string(&tag).unwrap();
        let restored: Tag = serde_json::from_str(&json).unwrap();
        assert_eq!(tag, restored);
    }

    // --- Clone tests ---

    #[test]
    fn all_models_clone() {
        let _ = make_user().clone();
        let _ = make_note().clone();
        let _ = make_todo().clone();
        let _ = make_calendar_event().clone();
        let _ = make_tag().clone();
    }
}
