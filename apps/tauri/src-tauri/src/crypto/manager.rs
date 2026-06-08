use tokio::sync::Mutex;
use crate::crypto::aes_gcm;
use crate::crypto::key_derivation;

const ENC_PREFIX: &str = "$enc$";

pub struct EncryptionManager {
    key: Mutex<Option<[u8; 32]>>,
}

impl EncryptionManager {
    pub fn new() -> Self {
        Self { key: Mutex::new(None) }
    }

    pub async fn set_key(&self, password: &str, salt: &[u8]) {
        let derived = key_derivation::derive_key(password, salt);
        *self.key.lock().await = Some(derived);
    }

    pub async fn clear_key(&self) {
        *self.key.lock().await = None;
    }

    pub async fn is_locked(&self) -> bool {
        self.key.lock().await.is_none()
    }

    pub async fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        let guard = self.key.lock().await;
        let key = guard.as_ref().ok_or_else(|| "Database locked, cannot encrypt".to_string())?;
        let encrypted = aes_gcm::encrypt(key, plaintext.as_bytes())?;
        Ok(format!("{}{}", ENC_PREFIX, hex::encode(encrypted)))
    }

    pub async fn decrypt_to_string(&self, data: &str) -> Result<String, String> {
        let rest = data.strip_prefix(ENC_PREFIX).ok_or_else(|| "Not encrypted".to_string())?;
        let guard = self.key.lock().await;
        let key = guard.as_ref().ok_or_else(|| "Database locked, cannot decrypt".to_string())?;
        let raw = hex::decode(rest).map_err(|e| e.to_string())?;
        let decrypted = aes_gcm::decrypt(key, &raw)?;
        String::from_utf8(decrypted).map_err(|e| e.to_string())
    }

    pub async fn try_decrypt(&self, data: &str) -> String {
        if !data.starts_with(ENC_PREFIX) {
            return data.to_string();
        }
        self.decrypt_to_string(data).await.unwrap_or_else(|_| "[encrypted]".to_string())
    }

    pub async fn encrypt_or_pass(&self, plaintext: &str) -> String {
        if self.is_locked().await {
            return plaintext.to_string();
        }
        self.encrypt(plaintext).await.unwrap_or_else(|_| plaintext.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::generate_salt;

    #[tokio::test]
    async fn roundtrip() {
        let mgr = EncryptionManager::new();
        let salt = generate_salt();
        mgr.set_key("password", &salt).await;
        let encrypted = mgr.encrypt("hello world").await.unwrap();
        assert!(encrypted.starts_with("$enc$"));
        let decrypted = mgr.decrypt_to_string(&encrypted).await.unwrap();
        assert_eq!(decrypted, "hello world");
    }

    #[tokio::test]
    async fn locked_returns_plaintext() {
        let mgr = EncryptionManager::new();
        let result = mgr.encrypt("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn try_decrypt_plaintext() {
        let mgr = EncryptionManager::new();
        assert_eq!(mgr.try_decrypt("hello").await, "hello");
    }

    #[tokio::test]
    async fn encrypt_or_pass_when_locked() {
        let mgr = EncryptionManager::new();
        assert_eq!(mgr.encrypt_or_pass("hello").await, "hello");
    }

    #[tokio::test]
    async fn clear_key_locks() {
        let mgr = EncryptionManager::new();
        let salt = generate_salt();
        mgr.set_key("p", &salt).await;
        assert!(!mgr.is_locked().await);
        mgr.clear_key().await;
        assert!(mgr.is_locked().await);
    }

    #[tokio::test]
    async fn wrong_password_fails() {
        let mgr = EncryptionManager::new();
        let salt = generate_salt();
        mgr.set_key("password1", &salt).await;
        let encrypted = mgr.encrypt("secret").await.unwrap();
        drop(mgr);
        let mgr2 = EncryptionManager::new();
        mgr2.set_key("password2", &salt).await;
        let result = mgr2.decrypt_to_string(&encrypted).await;
        assert!(result.is_err());
    }
}
