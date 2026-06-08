use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupabaseClient {
    pub url: String,
    pub key: String,
}

impl SupabaseClient {
    pub fn new(url: String, key: String) -> Self { Self { url, key } }
    pub async fn push_entity(&self, _table: &str, _data: &Value) -> Result<(), String> { Ok(()) }
    pub async fn pull_entities(&self, _table: &str, _since: Option<i64>) -> Result<Vec<Value>, String> { Ok(vec![]) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_client() {
        let client = SupabaseClient::new(
            "https://example.supabase.co".into(),
            "test-key".into(),
        );
        assert_eq!(client.url, "https://example.supabase.co");
        assert_eq!(client.key, "test-key");
    }

    #[test]
    fn serialize_deserialize() {
        let client = SupabaseClient::new("https://a.b".into(), "k".into());
        let json = serde_json::to_string(&client).unwrap();
        let restored: SupabaseClient = serde_json::from_str(&json).unwrap();
        assert_eq!(client, restored);
    }

    #[test]
    fn clone_produces_equal() {
        let client = SupabaseClient::new("url".into(), "key".into());
        let cloned = client.clone();
        assert_eq!(client, cloned);
    }
}
