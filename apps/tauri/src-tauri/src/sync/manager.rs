use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncManager {
    pub device_id: String,
    pub last_sync_at: Option<i64>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self { device_id: uuid::Uuid::new_v4().to_string(), last_sync_at: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sync_manager() {
        let mgr = SyncManager::new();
        assert!(!mgr.device_id.is_empty());
        assert!(mgr.last_sync_at.is_none());
    }

    #[test]
    fn device_id_is_uuid() {
        let mgr = SyncManager::new();
        // UUIDs are 36 characters with hyphens (e.g., "550e8400-e29b-41d4-a716-446655440000")
        assert_eq!(mgr.device_id.len(), 36);
        assert_eq!(mgr.device_id.chars().filter(|&c| c == '-').count(), 4);
    }

    #[test]
    fn unique_device_ids() {
        let mgr1 = SyncManager::new();
        let mgr2 = SyncManager::new();
        assert_ne!(mgr1.device_id, mgr2.device_id);
    }
}
