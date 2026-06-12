#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SyncQueueItem {
    pub id: i64,
    pub user_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: String,
    pub payload: Option<String>,
    pub vector_clock: Option<String>,
    pub created_at: i64,
    pub synced: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(operation: &str) -> SyncQueueItem {
        SyncQueueItem {
            id: 1,
            user_id: "user-1".into(),
            entity_type: "note".into(),
            entity_id: "note-123".into(),
            operation: operation.into(),
            payload: Some(r#"{"title":"Test"}"#.into()),
            vector_clock: None,
            created_at: 1700000000,
            synced: false,
        }
    }

    #[test]
    fn create_item_with_all_fields() {
        let item = make_item("create");
        assert_eq!(item.id, 1);
        assert_eq!(item.entity_type, "note");
        assert_eq!(item.operation, "create");
        assert!(!item.synced);
    }

    #[test]
    fn serialize_deserialize() {
        let item = make_item("update");
        let json = serde_json::to_string(&item).unwrap();
        let restored: SyncQueueItem = serde_json::from_str(&json).unwrap();
        assert_eq!(item, restored);
    }

    #[test]
    fn sync_status_default() {
        let item = make_item("delete");
        assert!(!item.synced);
    }

    #[test]
    fn optional_fields() {
        let item = SyncQueueItem {
            id: 2,
            user_id: "u".into(),
            entity_type: "todo".into(),
            entity_id: "t-1".into(),
            operation: "create".into(),
            payload: None,
            vector_clock: Some(r#"{"node-a":5}"#.into()),
            created_at: 1700000000,
            synced: true,
        };
        assert!(item.payload.is_none());
        assert!(item.vector_clock.is_some());
        assert!(item.synced);
    }

    #[test]
    fn clone_produces_equal_item() {
        let item = make_item("create");
        let cloned = item.clone();
        assert_eq!(item, cloned);
    }
}
