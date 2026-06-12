use tokio::sync::Mutex;
use sha2::{Sha256, Digest};
use crate::crypto::aes_gcm;
use crate::crypto::key_derivation;

const ENC_PREFIX: &str = "$enc$";

pub struct EncryptionManager {
    key: Mutex<Option<[u8; 32]>>,
    session_secret: Mutex<Option<[u8; 32]>>,
}

impl EncryptionManager {
    pub fn new() -> Self {
        Self { key: Mutex::new(None), session_secret: Mutex::new(None) }
    }

    /// Derive and set the encryption key from a password and a stored salt string.
    /// The stored salt may be legacy SHA-256 (raw hex) or new Argon2id ("argon2id:" prefix).
    pub async fn set_key_from_stored_salt(&self, password: &str, stored_salt: &str) -> Result<(), String> {
        let (derived, _) = key_derivation::derive_key_from_stored_salt(password, stored_salt)?;
        *self.key.lock().await = Some(derived);
        Ok(())
    }

    /// Derive and set the encryption key using Argon2id (for new passwords/rotation).
    pub async fn set_key_argon2id(&self, password: &str, salt: &[u8]) -> Result<String, String> {
        let (derived, versioned_salt) = key_derivation::derive_key_argon2id(password, salt)?;
        *self.key.lock().await = Some(derived);
        Ok(versioned_salt)
    }

    /// Legacy: set key from raw salt bytes using SHA-256 (kept for backward compat, tests only).
    #[cfg(test)]
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

    /// Get a copy of the current encryption key, if set.
    pub async fn get_key_copy(&self) -> Option<[u8; 32]> {
        *self.key.lock().await
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

    /// Encrypt plaintext. Returns an error if the database is locked (fail-closed).
    /// This prevents any sensitive data from being stored unencrypted.
    pub async fn encrypt_or_pass(&self, plaintext: &str) -> Result<String, String> {
        self.encrypt(plaintext).await
    }

    // --- Session HMAC ---

    /// Generate a new random session secret and store it in memory.
    /// Returns the secret bytes (also stored internally for HMAC verification).
    pub async fn set_session_secret(&self) -> [u8; 32] {
        let mut secret = [0u8; 32];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut secret);
        *self.session_secret.lock().await = Some(secret);
        secret
    }

    /// Clear the session secret from memory (called on lock).
    pub async fn clear_session_secret(&self) {
        *self.session_secret.lock().await = None;
    }

    /// Compute HMAC-SHA256 of a timestamp using the in-memory session secret.
    /// Returns hex-encoded hash. Used to detect DB tampering of session_unlocked_at.
    pub async fn compute_session_hmac(&self, timestamp: &str) -> Result<String, String> {
        let guard = self.session_secret.lock().await;
        let secret = guard.as_ref().ok_or_else(|| "No session secret — database is locked".to_string())?;
        Ok(Self::hmac_sha256(secret, timestamp))
    }

    /// Verify that a stored HMAC matches the expected value for a given timestamp.
    /// Returns true if valid, false if the DB has been tampered with.
    pub async fn verify_session_hmac(&self, timestamp: &str, stored_hmac: &str) -> Result<bool, String> {
        let expected = self.compute_session_hmac(timestamp).await?;
        Ok(expected == stored_hmac)
    }

    /// Raw HMAC-SHA256 computation: hash(secret || message).
    fn hmac_sha256(secret: &[u8; 32], message: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(secret);
        hasher.update(message.as_bytes());
        hex::encode(hasher.finalize())
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
    async fn encrypt_or_pass_when_locked_returns_error() {
        let mgr = EncryptionManager::new();
        assert!(mgr.encrypt_or_pass("hello").await.is_err());
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

    #[tokio::test]
    async fn session_hmac_roundtrip() {
        let mgr = EncryptionManager::new();
        let secret = mgr.set_session_secret().await;
        assert_eq!(secret.len(), 32);

        let ts = "1700000000";
        let hmac = mgr.compute_session_hmac(ts).await.unwrap();
        assert!(!hmac.is_empty());

        // Verify should succeed
        assert!(mgr.verify_session_hmac(ts, &hmac).await.unwrap());
        // Verify should fail with wrong HMAC
        assert!(!mgr.verify_session_hmac(ts, "deadbeef").await.unwrap());
        // Verify should fail with wrong timestamp
        assert!(!mgr.verify_session_hmac("9999999999", &hmac).await.unwrap());
    }

    #[tokio::test]
    async fn session_hmac_deterministic() {
        let mgr = EncryptionManager::new();
        mgr.set_session_secret().await;
        let ts = "1700000000";
        let h1 = mgr.compute_session_hmac(ts).await.unwrap();
        let h2 = mgr.compute_session_hmac(ts).await.unwrap();
        assert_eq!(h1, h2);
    }

    #[tokio::test]
    async fn session_secret_clear() {
        let mgr = EncryptionManager::new();
        mgr.set_session_secret().await;
        let _hmac1 = mgr.compute_session_hmac("123").await.unwrap();
        mgr.clear_session_secret().await;
        assert!(mgr.compute_session_hmac("123").await.is_err());
    }

    #[tokio::test]
    async fn different_secrets_different_hmacs() {
        let mgr1 = EncryptionManager::new();
        mgr1.set_session_secret().await;
        let h1 = mgr1.compute_session_hmac("123").await.unwrap();

        let mgr2 = EncryptionManager::new();
        mgr2.set_session_secret().await;
        let h2 = mgr2.compute_session_hmac("123").await.unwrap();

        assert_ne!(h1, h2);
    }
}
