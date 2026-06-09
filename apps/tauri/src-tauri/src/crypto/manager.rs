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

    /// Derive and set the encryption key from a password and a stored salt string.
    /// The stored salt may be legacy SHA-256 (raw hex) or new Argon2id ("argon2id:" prefix).
    pub async fn set_key_from_stored_salt(&self, password: &str, stored_salt: &str) -> Result<(), String> {
        let (derived, _) = key_derivation::derive_key_from_stored_salt(password, stored_salt)?;
        *self.key.lock().await = Some(derived);
        Ok(())
    }

    /// Derive and set the encryption key using Argon2id (for new passwords/rotation).
    pub async fn set_key_argon2id(&self, password: &str, salt: &[u8]) -> Result<(String), String> {
        let (derived, versioned_salt) = key_derivation::derive_key_argon2id(password, salt)?;
        *self.key.lock().await = Some(derived);
        Ok(versioned_salt)
    }

    /// Legacy: set key from raw salt bytes using SHA-256 (kept for backward compat).
    pub async fn set_key(&self, password: &str, salt: &[u8]) {
        let (derived, _) = key_derivation::derive_key_sha256(password, salt);
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

    /// Encrypt raw binary data (for attachments). Returns nonce||ciphertext.
    pub async fn encrypt_raw(&self, plaintext: &[u8]) -> Result<Vec<u8>, String> {
        let guard = self.key.lock().await;
        let key = guard.as_ref().ok_or_else(|| "Database locked, cannot encrypt".to_string())?;
        aes_gcm::encrypt(key, plaintext)
    }

    /// Decrypt raw binary data (for attachments). Input is nonce||ciphertext.
    pub async fn decrypt_raw(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let guard = self.key.lock().await;
        let key = guard.as_ref().ok_or_else(|| "Database locked, cannot decrypt".to_string())?;
        aes_gcm::decrypt(key, data)
    }

    pub async fn try_decrypt(&self, data: &str) -> String {
        if !data.starts_with(ENC_PREFIX) {
            return data.to_string();
        }
        self.decrypt_to_string(data).await.unwrap_or_else(|_| "[encrypted]".to_string())
    }

    /// Encrypt plaintext, returning the plaintext unchanged if the database is locked.
    /// WARNING: If encryption fails for any reason other than being locked, this propagates the error
    /// instead of silently storing unencrypted data. Previously this silently fell back to plaintext.
    pub async fn encrypt_or_pass(&self, plaintext: &str) -> Result<String, String> {
        if self.is_locked().await {
            return Ok(plaintext.to_string());
        }
        self.encrypt(plaintext).await
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
        assert_eq!(mgr.encrypt_or_pass("hello").await.unwrap(), "hello");
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
