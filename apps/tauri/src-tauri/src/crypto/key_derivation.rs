use rand::RngCore;

pub fn generate_salt() -> [u8; 32] {
    let mut salt = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

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
    fn derive_key_produces_32_bytes() {
        let salt = generate_salt();
        let key = derive_key("password", &salt);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn derive_key_deterministic() {
        let salt = [1u8; 32];
        let key1 = derive_key("password", &salt);
        let key2 = derive_key("password", &salt);
        assert_eq!(key1, key2);
    }

    #[test]
    fn different_passwords_produce_different_keys() {
        let salt = [0u8; 32];
        let key1 = derive_key("password1", &salt);
        let key2 = derive_key("password2", &salt);
        assert_ne!(key1, key2);
    }

    #[test]
    fn different_salts_produce_different_keys() {
        let salt1 = [1u8; 32];
        let salt2 = [2u8; 32];
        let key1 = derive_key("password", &salt1);
        let key2 = derive_key("password", &salt2);
        assert_ne!(key1, key2);
    }

    #[test]
    fn empty_password_works() {
        let salt = [0u8; 32];
        let key = derive_key("", &salt);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn known_vector_sha256() {
        // SHA-256("hello" || [0u8; 32]) = known value
        let salt = [0u8; 32];
        let key = derive_key("hello", &salt);
        let hex_key = hex::encode(key);
        // sha256("hello" + 32 zero bytes)
        assert_eq!(hex_key, "6728ef7b85f1a16f9a7dccb26de33b39e25c4ef2f148d2b531c8cf0e3a1f8280");
    }
}
