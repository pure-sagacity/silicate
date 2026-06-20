mod json;

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
use std::io::Write;
use std::process::{Command, Stdio};

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

/// This function lists all the password files in the config directory, excluding the salt file.
/// It returns a vector of website names (without the .bin extension).
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

/// This function checks if fzf is installed on the system by trying to find its path.
pub fn check_fzf_installed() -> bool {
    which::which("fzf").is_ok()
}

/// This function takes the config directory and an optional tag, lists the passwords, filters them by tag if provided,
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

/// This function generates a random password of the specified length. If use_symbols is true, it includes symbols in the password.
pub fn generate_password(length: usize, use_symbols: bool) -> String {
    if length == 0 {
        return String::new();
    }

    let letters_and_digits = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let symbols = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

    if use_symbols {
        let mut combined = Vec::with_capacity(letters_and_digits.len() + symbols.len());
        combined.extend_from_slice(letters_and_digits);
        combined.extend_from_slice(symbols);
        return sample_from_pool(&combined, length);
    }

    sample_from_pool(letters_and_digits, length)
}

fn sample_from_pool(pool: &[u8], length: usize) -> String {
    let mut rng = OsRng;
    let mut result = String::with_capacity(length);

    let pool_len = pool.len() as u32;
    // To prevent modulo bias, calculate the maximum allowable value
    // that fits perfectly into multiples of our pool length.
    let zone = u32::MAX - (u32::MAX % pool_len);

    while result.len() < length {
        // Use RngCore's next_u32 directly (always available on OsRng)
        let random_val = rng.next_u32();

        // Rejection sampling: if it falls in the biased remainder zone, skip it
        if random_val < zone {
            let idx = (random_val % pool_len) as usize;
            result.push(pool[idx] as char);
        }
    }

    result
}

pub fn update_password(
    config_dir: &str,
    key: &[u8; 32],
    website: &str,
    tag: Option<&str>,
    new_plaintext: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let (new_ciphertext, new_nonce) =
        encrypt_passwd(key, new_plaintext).map_err(|e| format!("Encryption failed: {:?}", e))?;
    let mut combined_data = Vec::new();
    combined_data.extend_from_slice(&new_nonce);
    combined_data.extend_from_slice(&new_ciphertext);
    update_entry(config_dir, website, tag, &hex::encode(combined_data))?;
    Ok(())
}

fn update_entry(
    config_dir: &str,
    website: &str,
    tag: Option<&str>,
    new_data: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let filename = if let Some(t) = tag {
        format!("{}-{}.bin", website, t)
    } else {
        format!("{}.bin", website)
    };
    let filepath = std::path::Path::new(config_dir).join(filename);
    std::fs::write(filepath, new_data)?;
    Ok(())
}

pub fn find_password_file(config_dir: &str, target_website: &str) -> Option<String> {
    let passwords = list_passwords(config_dir);

    passwords.into_iter().find(|filename| {
        // If it's an exact match (no tag)
        if filename == target_website {
            return true;
        }

        // If it has a tag, check if the part before the first '-' matches
        if let Some((base_website, _tag)) = filename.split_once('-') {
            if base_website == target_website {
                return true;
            }
        }

        false
    })
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
        let password = "test_password";
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

    #[test]
    fn test_password_generation() {
        let password = generate_password(16, true);
        assert_eq!(password.len(), 16);
        assert!(
            password
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        );
    }

    #[test]
    fn password_generation_no_symbols() {
        let password = generate_password(16, false);
        assert_eq!(password.len(), 16);
        assert!(
            !password
                .chars()
                .any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c))
        );
    }
}
