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
