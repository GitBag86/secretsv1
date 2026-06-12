use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SupabaseClient {
    pub url: String,
    pub key: String,
}

impl SupabaseClient {
    pub fn new(url: String, key: String) -> Self { Self { url, key } }

    fn api_url(&self, table: &str) -> String {
        format!("{}/rest/v1/{}", self.url.trim_end_matches('/'), table)
    }

    fn headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("apikey", self.key.parse().unwrap());
        headers.insert("Authorization", format!("Bearer {}", self.key).parse().unwrap());
        headers.insert("Content-Type", "application/json".parse().unwrap());
        headers
    }

    /// Push (upsert) an entity to Supabase via POST
    pub async fn upsert_entity(&self, table: &str, data: &Value) -> Result<(), String> {
        let client = reqwest::Client::new();
        let resp = client
            .post(self.api_url(table))
            .headers(self.headers())
            .header("Prefer", "resolution=merge-duplicates")
            .json(data)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase push failed ({}): {}", status, body));
        }
        Ok(())
    }

    /// Delete an entity from Supabase by ID
    pub async fn delete_entity(&self, table: &str, id: &str) -> Result<(), String> {
        let client = reqwest::Client::new();
        let url = format!("{}?id=eq.{}", self.api_url(table), id);
        let resp = client
            .delete(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() && resp.status().as_u16() != 404 {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase delete failed ({}): {}", status, body));
        }
        Ok(())
    }

    /// Pull entities from Supabase updated since a given timestamp
    pub async fn pull_entities(&self, table: &str, since: Option<i64>) -> Result<Vec<Value>, String> {
        let client = reqwest::Client::new();
        let mut url = format!("{}?select=*&order=updated_at.desc", self.api_url(table));
        if let Some(ts) = since {
            url.push_str(&format!("&updated_at=gt.{}", ts));
        }
        let resp = client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Supabase pull failed ({}): {}", status, body));
        }

        let data: Vec<Value> = resp.json().await.map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(data)
    }

    /// Test connection by fetching one row
    pub async fn test_connection(&self) -> Result<bool, String> {
        let client = reqwest::Client::new();
        let url = format!("{}?select=id&limit=1", self.api_url("notes"));
        let resp = client
            .get(&url)
            .headers(self.headers())
            .send()
            .await
            .map_err(|e| format!("Connection test failed: {}", e))?;
        Ok(resp.status().is_success())
    }
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

    #[test]
    fn api_url_format() {
        let client = SupabaseClient::new("https://proj.supabase.co".into(), "k".into());
        assert_eq!(client.api_url("notes"), "https://proj.supabase.co/rest/v1/notes");
    }

    #[test]
    fn api_url_trailing_slash_stripped() {
        let client = SupabaseClient::new("https://proj.supabase.co/".into(), "k".into());
        assert_eq!(client.api_url("notes"), "https://proj.supabase.co/rest/v1/notes");
    }
}
