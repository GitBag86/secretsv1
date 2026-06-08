use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use rand::RngCore;

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| e.to_string())?;
    let mut result = nonce_bytes.to_vec();
    result.extend(ciphertext);
    Ok(result)
}

pub fn decrypt(key: &[u8; 32], data: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() < 12 { return Err("Data too short".into()); }
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&data[..12]);
    let ciphertext = &data[12..];
    cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [42u8; 32];
        let plaintext = b"Hello, world! This is a secret message.";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_empty_plaintext() {
        let key = [1u8; 32];
        let plaintext = b"";
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_binary_data() {
        let key = [99u8; 32];
        let plaintext: Vec<u8> = (0..=255).collect();
        let encrypted = encrypt(&key, &plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_decryption() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let plaintext = b"secret data";
        let encrypted = encrypt(&key1, plaintext).unwrap();
        let result = decrypt(&key2, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = [5u8; 32];
        let plaintext = b"original data";
        let mut encrypted = encrypt(&key, plaintext).unwrap();
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF; // flip bits
        let result = decrypt(&key, &encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn too_short_data_fails() {
        let key = [3u8; 32];
        let short_data = [0u8; 11]; // less than 12 bytes
        let result = decrypt(&key, &short_data);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Data too short");
    }

    #[test]
    fn exactly_12_bytes_fails_with_no_ciphertext() {
        let key = [7u8; 32];
        let data = [0u8; 12]; // just nonce, no ciphertext
        let result = decrypt(&key, &data);
        assert!(result.is_err());
    }

    #[test]
    fn different_encryptions_produce_different_ciphertexts() {
        let key = [10u8; 32];
        let plaintext = b"same message";
        let enc1 = encrypt(&key, plaintext).unwrap();
        let enc2 = encrypt(&key, plaintext).unwrap();
        // Different nonces should produce different ciphertexts
        assert_ne!(enc1, enc2);
    }

    #[test]
    fn large_plaintext_encrypts_decrypts() {
        let key = [0u8; 32];
        let plaintext = vec![0xABu8; 100_000]; // 100KB
        let encrypted = encrypt(&key, &plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
