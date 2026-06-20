use std::io::Write;
use std::process::{Command, Stdio};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng as ArOsRng},
};
use colored::*;
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

pub fn check_fzf_installed() -> bool {
    which::which("fzf").is_ok()
}

pub fn search_password(
    config_dir: &str,
    tag: &Option<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let websites = list_passwords(config_dir);
    if websites.is_empty() {
        println!("No passwords found in the config directory.");
        return Ok(None);
    }

    let websites = if let Some(t) = tag {
        websites
            .into_iter()
            .filter(|w| {
                if let Some((_, w_tag)) = w.split_once('-') {
                    w_tag == t
                } else {
                    false
                }
            })
            .map(|w| {
                if let Some((site, _)) = w.split_once('-') {
                    format!("{}", site)
                } else {
                    w.clone()
                }
            })
            .collect::<Vec<String>>()
    } else {
        websites
            .into_iter()
            .map(|w| {
                if let Some((site, tag)) = w.split_once('-') {
                    format!("({}) {}", tag, site)
                } else {
                    w.clone()
                }
            })
            .collect::<Vec<String>>()
    };

    if websites.is_empty() {
        println!("{}", "No passwords found for the specified tag.".red());
        return Ok(None);
    }

    let fzf_input = websites.join("\n");

    // 3. Spawn the fzf process
    // We inherit stderr so fzf can draw its interactive UI on the terminal screen,
    // while we pipe stdin (to send data) and stdout (to catch the choice).
    let mut child = Command::new("fzf")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    // 4. Write our database records to fzf's stdin asynchronously
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(fzf_input.as_bytes())?;
    }

    // 5. Wait for the user to make a selection and exit
    let output = child.wait_with_output()?;

    // 6. Handle the result based on the exit code
    if output.status.success() {
        let selection = String::from_utf8(output.stdout)?;
        let trimmed_selection = selection.trim();

        if trimmed_selection.is_empty() {
            println!("{}", "No selection made.".red());
            Ok(None)
        } else {
            Ok(Some(trimmed_selection.to_string()))
        }
    } else {
        // Exit code 130 typically means the user pressed Esc/Ctrl-C
        println!("{}", "Selection canceled or fzf failed.".red());
        Ok(None)
    }
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
