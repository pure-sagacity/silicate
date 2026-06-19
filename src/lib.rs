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
    let cipher = Aes256Gcm::new_from_slice(key_bytes).unwrap();

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes())?;
    Ok((ciphertext, nonce_bytes))
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
