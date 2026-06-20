use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng as ArOsRng},
};
use keyring::{Entry, Error};

const SERVICE_NAME: &str = "silicate";
const USERNAME: &str = "default";

/// Encrypts the given plaintext using AES-256-GCM. Returns the ciphertext and the nonce used for encryption.
pub fn encrypt_passwd(
    key_bytes: &[u8; 32],
    plaintext: String,
) -> Result<(Vec<u8>, [u8; 12]), aes_gcm::Error> {
    // Changed [u8; 12] to Vec<u8>
    let cipher = Aes256Gcm::new_from_slice(key_bytes).unwrap();

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    Ok((ciphertext, nonce_bytes)) // Removed the broken .try_into().unwrap()
}

/// Decrypts the given ciphertext using AES-256-GCM. Requires the same key and nonce used for encryption.
pub fn decrypt_passwd(
    key_bytes: &[u8; 32],
    ciphertext: Vec<u8>,
    nonce_bytes: [u8; 12],
) -> Result<String, aes_gcm::Error> {
    let cipher = Aes256Gcm::new_from_slice(key_bytes).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext.as_ref())?;
    let plaintext = String::from_utf8(plaintext_bytes).unwrap();
    Ok(plaintext)
}

/// Generates a random 256-bit key for AES encryption.
pub fn generate_key() -> [u8; 32] {
    let key = Aes256Gcm::generate_key(OsRng);
    key.into()
}

/// Generates a fallback key using a password-based key derivation.
/// This is used when the user doesn't have a secure key management solution in place.
/// Returns the derived key and the salt used for hashing.
pub fn generate_fallback_key(password: &str) -> ([u8; 32], [u8; 16]) {
    let salt = SaltString::generate(&mut ArOsRng);
    let argon2 = Argon2::default(); // 32-byte output by default
    let hashed = argon2.hash_password(password.as_bytes(), &salt).unwrap();
    let key_bytes: [u8; 32] = hashed.hash.unwrap().as_bytes().try_into().unwrap();
    let mut salt_bytes = [0u8; 16];
    salt.decode_b64(&mut salt_bytes).unwrap();
    (key_bytes.try_into().unwrap(), salt_bytes)
}

/// This function will take a salt and a password and derive the same key as the generate_fallback_key function.
/// This is used for retrieving the key when the user doesn't have a secure key management solution in place.
pub fn derive_key_from_password(password: &str, salt: &[u8; 16]) -> [u8; 32] {
    let salt_string = SaltString::encode_b64(salt).unwrap();
    let argon2 = Argon2::default(); // 32-byte output by default
    let hashed = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .unwrap();
    let key_bytes: [u8; 32] = hashed.hash.unwrap().as_bytes().try_into().unwrap();
    key_bytes.try_into().unwrap()
}

/// This puts a randomly generated key into the system's keyring.
pub fn store_key_in_keyring(key: &[u8; 32]) -> Result<(), Error> {
    let entry = Entry::new(SERVICE_NAME, USERNAME)?;
    entry.set_password(&hex::encode(key))?;
    Ok(())
}

/// This retrieves the key from the system's keyring.
pub fn retrieve_key_from_keyring() -> Result<[u8; 32], Error> {
    let entry = Entry::new(SERVICE_NAME, USERNAME)?;
    let key_hex = entry.get_password()?;
    let key_bytes = hex::decode(key_hex).unwrap();
    Ok(key_bytes.try_into().unwrap())
}

/// This function checks if a keyring is available and can be accessed.
/// This will be for checking if the user has a secure key management solution in place.
pub fn is_keyring_available() -> bool {
    let entry = Entry::new(SERVICE_NAME, USERNAME);
    entry.is_ok()
}

pub fn list_passwords(config_dir: &str) -> Vec<String> {
    let mut websites = Vec::new();
    if let Ok(entries) = std::fs::read_dir(config_dir) {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if filename.ends_with(".bin") && filename != "salt.bin" {
                    websites.push(filename.trim_end_matches(".bin").to_string());
                }
            }
        }
    }
    websites
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = "This is a test password.".to_string();
        let (ciphertext, nonce) = encrypt_passwd(&key, plaintext.clone()).unwrap();
        let decrypted = decrypt_passwd(&key, ciphertext.to_vec(), nonce).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_fallback_key_derivation() {
        let password = "testpassword";
        let (derived_key, salt) = generate_fallback_key(password);
        let derived_key_again = derive_key_from_password(password, &salt);
        assert_eq!(derived_key, derived_key_again);
    }

    #[test]
    fn test_keyring_storage() {
        // Just so the code doesn't fail if no keyring
        match is_keyring_available() {
            true => (),
            false => {
                println!("Keyring not available, skipping keyring storage test.");
                assert!(true);
            }
        }
        let key = generate_key();
        store_key_in_keyring(&key).unwrap();
        let retrieved_key = retrieve_key_from_keyring().unwrap();
        assert_eq!(key, retrieved_key);
    }
}
