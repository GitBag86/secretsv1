use rand::RngCore;
use argon2::{Argon2, Algorithm, Version, Params};

/// Salt prefix to identify which KDF was used to derive the key.
/// New keys use "argon2id:" prefix. Legacy keys have no prefix (SHA-256).
pub const ARGON2ID_PREFIX: &str = "argon2id:";

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Derive a 32-byte encryption key using Argon2id (memory-hard, GPU-resistant).
/// Returns (key, versioned_salt_hex) where versioned_salt_hex includes the "argon2id:" prefix.
pub fn derive_key_argon2id(password: &str, salt: &[u8]) -> Result<([u8; 32], String), String> {
    // Argon2id with OWASP-recommended parameters:
    // m=65536 (64 MB), t=3 iterations, p=4 parallelism, output=32 bytes
    let params = Params::new(65_536, 3, 4, Some(32))
        .map_err(|e| format!("Argon2 params error: {}", e))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::Version0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Argon2id key derivation failed: {}", e))?;

    let versioned_salt = format!("{}{}", ARGON2ID_PREFIX, hex::encode(salt));
    Ok((key, versioned_salt))
}

/// Derive a 32-byte encryption key using legacy SHA-256 (insecure, kept for backward compat).
/// Returns (key, versioned_salt_hex) with no prefix (legacy format).
pub fn derive_key_sha256(password: &str, salt: &[u8]) -> ([u8; 32], String) {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);

    // Legacy format: raw hex without prefix
    let versioned_salt = hex::encode(salt);
    (key, versioned_salt)
}

/// Detect which KDF version a stored salt uses, then derive the key accordingly.
pub fn derive_key_from_stored_salt(password: &str, stored_salt: &str) -> Result<([u8; 32], String), String> {
    if stored_salt.starts_with(ARGON2ID_PREFIX) {
        let raw_salt = hex::decode(stored_salt.strip_prefix(ARGON2ID_PREFIX).unwrap())
            .map_err(|e| format!("Invalid salt hex: {}", e))?;
        derive_key_argon2id(password, &raw_salt)
    } else {
        // Legacy SHA-256 salt (no prefix)
        let raw_salt = hex::decode(stored_salt)
            .map_err(|e| format!("Invalid salt hex: {}", e))?;
        let (key, _) = derive_key_sha256(password, &raw_salt);
        // Re-encode with the same legacy format so callers see no change
        let versioned_salt = hex::encode(&raw_salt);
        Ok((key, versioned_salt))
    }
}

/// Legacy SHA-256 derivation — kept only for reading old data.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_salt_produces_32_bytes() {
        let salt = generate_salt();
        assert_eq!(salt.len(), 32);
    }

    #[test]
    fn generate_salt_is_random() {
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        assert_ne!(salt1, salt2);
    }

    #[test]
    fn argon2id_derive_key_produces_32_bytes() {
        let salt = generate_salt();
        let (key, versioned) = derive_key_argon2id("password", &salt).unwrap();
        assert_eq!(key.len(), 32);
        assert!(versioned.starts_with(ARGON2ID_PREFIX));
    }

    #[test]
    fn argon2id_derive_key_deterministic() {
        let salt = [1u8; 32];
        let (key1, _) = derive_key_argon2id("password", &salt).unwrap();
        let (key2, _) = derive_key_argon2id("password", &salt).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn argon2id_different_passwords_produce_different_keys() {
        let salt = [0u8; 32];
        let (key1, _) = derive_key_argon2id("password1", &salt).unwrap();
        let (key2, _) = derive_key_argon2id("password2", &salt).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn argon2id_different_salts_produce_different_keys() {
        let salt1 = [1u8; 32];
        let salt2 = [2u8; 32];
        let (key1, _) = derive_key_argon2id("password", &salt1).unwrap();
        let (key2, _) = derive_key_argon2id("password", &salt2).unwrap();
        assert_ne!(key1, key2);
    }

    #[test]
    fn sha256_derive_key_produces_32_bytes() {
        let salt = generate_salt();
        let (key, versioned) = derive_key_sha256("password", &salt);
        assert_eq!(key.len(), 32);
        // Legacy format has no prefix
        assert!(!versioned.starts_with(ARGON2ID_PREFIX));
    }

    #[test]
    fn sha256_known_vector() {
        let salt = [0u8; 32];
        let (key, _) = derive_key_sha256("hello", &salt);
        let hex_key = hex::encode(key);
        assert_eq!(hex_key, "6728ef7b85f1a16f9a7dccb26de33b39e25c4ef2f148d2b531c8cf0e3a1f8280");
    }

    #[test]
    fn derive_key_from_stored_salt_argon2id() {
        let salt = [42u8; 32];
        let (_, stored) = derive_key_argon2id("test", &salt).unwrap();
        let (key1, _) = derive_key_argon2id("test", &salt).unwrap();
        let (key2, _) = derive_key_from_stored_salt("test", &stored).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn derive_key_from_stored_salt_sha256_legacy() {
        let salt = [42u8; 32];
        let (key1, _) = derive_key_sha256("test", &salt);
        let stored = hex::encode(&salt);
        let (key2, _) = derive_key_from_stored_salt("test", &stored).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn empty_password_works() {
        let salt = [0u8; 32];
        let (key, _) = derive_key_argon2id("", &salt).unwrap();
        assert_eq!(key.len(), 32);
    }
}
