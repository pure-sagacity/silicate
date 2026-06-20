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
use keyring::Entry;
use std::io::Write;
use std::process::{Command, Stdio};

const SERVICE_NAME: &str = "silicate";
const USERNAME: &str = "default";

#[derive(Debug)]
pub enum SilicateError {
    KeyringError(keyring::Error),
    IoError(std::io::Error),
    HexError(hex::FromHexError),
    SerdeJsonError(serde_json::Error),
    Argon2Error(argon2::password_hash::Error),
    AesGcmError(aes_gcm::Error),
    AesInvalidKeyLengthError(aes_gcm::aes::cipher::InvalidLength),
    StdioError(std::io::Error),
    Utf8Error(std::string::FromUtf8Error),
    Argon2PasswordHashError(argon2::password_hash::Error),
    TryFromSliceError(std::array::TryFromSliceError),
}

impl From<keyring::Error> for SilicateError {
    fn from(err: keyring::Error) -> SilicateError {
        SilicateError::KeyringError(err)
    }
}

impl From<std::io::Error> for SilicateError {
    fn from(err: std::io::Error) -> SilicateError {
        SilicateError::IoError(err)
    }
}

impl From<hex::FromHexError> for SilicateError {
    fn from(err: hex::FromHexError) -> SilicateError {
        SilicateError::HexError(err)
    }
}

impl From<serde_json::Error> for SilicateError {
    fn from(err: serde_json::Error) -> SilicateError {
        SilicateError::SerdeJsonError(err)
    }
}

impl From<argon2::password_hash::Error> for SilicateError {
    fn from(err: argon2::password_hash::Error) -> SilicateError {
        SilicateError::Argon2Error(err)
    }
}

impl From<aes_gcm::Error> for SilicateError {
    fn from(err: aes_gcm::Error) -> SilicateError {
        SilicateError::AesGcmError(err)
    }
}

impl From<std::string::FromUtf8Error> for SilicateError {
    fn from(err: std::string::FromUtf8Error) -> SilicateError {
        SilicateError::Utf8Error(err)
    }
}

impl From<aes_gcm::aes::cipher::InvalidLength> for SilicateError {
    fn from(err: aes_gcm::aes::cipher::InvalidLength) -> SilicateError {
        SilicateError::AesInvalidKeyLengthError(err)
    }
}

impl From<std::array::TryFromSliceError> for SilicateError {
    fn from(err: std::array::TryFromSliceError) -> SilicateError {
        SilicateError::TryFromSliceError(err)
    }
}

impl From<Vec<u8>> for SilicateError {
    fn from(err: Vec<u8>) -> SilicateError {
        SilicateError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Vec<u8> error: {:?}", err),
        ))
    }
}

impl std::fmt::Display for SilicateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SilicateError::KeyringError(e) => write!(f, "Keyring error: {}", e),
            SilicateError::IoError(e) => write!(f, "I/O error: {}", e),
            SilicateError::HexError(e) => write!(f, "Hex decoding error: {}", e),
            SilicateError::SerdeJsonError(e) => {
                write!(f, "JSON serialization/deserialization error: {}", e)
            }
            SilicateError::Argon2Error(e) => write!(f, "Argon2 error: {}", e),
            SilicateError::AesGcmError(e) => {
                write!(f, "AES-GCM encryption/decryption error: {}", e)
            }
            SilicateError::Utf8Error(e) => write!(f, "UTF-8 conversion error: {}", e),
            SilicateError::AesInvalidKeyLengthError(e) => {
                write!(f, "AES invalid key length error: {}", e)
            }
            SilicateError::TryFromSliceError(e) => write!(f, "TryFromSlice error: {}", e),
            SilicateError::Argon2PasswordHashError(e) => {
                write!(f, "Argon2 password hash error: {}", e)
            }
            SilicateError::StdioError(e) => write!(f, "Stdio error: {}", e),
        }
    }
}

/// Encrypts the given plaintext using AES-256-GCM. Returns the ciphertext and the nonce used for encryption.
pub fn encrypt_passwd(
    key_bytes: &[u8; 32],
    plaintext: String,
) -> Result<(Vec<u8>, [u8; 12]), SilicateError> {
    // Changed [u8; 12] to Vec<u8>
    let cipher = Aes256Gcm::new_from_slice(key_bytes)?;

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
) -> Result<String, SilicateError> {
    let cipher = Aes256Gcm::new_from_slice(key_bytes)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext_bytes = cipher.decrypt(nonce, ciphertext.as_ref())?;
    let plaintext = String::from_utf8(plaintext_bytes)?;
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
pub fn generate_fallback_key(password: &str) -> Result<([u8; 32], [u8; 16]), SilicateError> {
    let salt = SaltString::generate(&mut ArOsRng);
    let argon2 = Argon2::default(); // 32-byte output by default
    let hashed = argon2.hash_password(password.as_bytes(), &salt)?;
    let key_bytes: [u8; 32] = hashed
        .hash
        .ok_or_else(|| {
            SilicateError::Argon2PasswordHashError(argon2::password_hash::Error::Password)
        })?
        .as_bytes()
        .try_into()?;
    let mut salt_bytes = [0u8; 16];
    salt.decode_b64(&mut salt_bytes)?;
    Ok((key_bytes, salt_bytes))
}

/// This function will take a salt and a password and derive the same key as the generate_fallback_key function.
/// This is used for retrieving the key when the user doesn't have a secure key management solution in place.
pub fn derive_key_from_password(
    password: &str,
    salt: &[u8; 16],
) -> Result<[u8; 32], SilicateError> {
    let salt_string = SaltString::encode_b64(salt)?;
    let argon2 = Argon2::default(); // 32-byte output by default
    let hashed = argon2.hash_password(password.as_bytes(), &salt_string)?;
    let key_bytes: [u8; 32] = hashed
        .hash
        .ok_or_else(|| {
            SilicateError::Argon2PasswordHashError(argon2::password_hash::Error::Password)
        })?
        .as_bytes()
        .try_into()?;
    Ok(key_bytes)
}

/// This puts a randomly generated key into the system's keyring.
pub fn store_key_in_keyring(key: &[u8; 32]) -> Result<(), SilicateError> {
    let entry = Entry::new(SERVICE_NAME, USERNAME)?;
    entry.set_password(&hex::encode(key))?;
    Ok(())
}

/// This retrieves the key from the system's keyring.
pub fn retrieve_key_from_keyring() -> Result<[u8; 32], SilicateError> {
    let entry = Entry::new(SERVICE_NAME, USERNAME)?;
    let key_hex = entry.get_password()?;
    let key_bytes: [u8; 32] = hex::decode(key_hex)?.try_into()?;
    Ok(key_bytes)
}

/// This function checks if a keyring is available and can be accessed.
/// This will be for checking if the user has a secure key management solution in place.
pub fn is_keyring_available() -> bool {
    let entry = Entry::new(SERVICE_NAME, USERNAME);
    entry.is_ok()
}

/// This function lists all the password files in the config directory, excluding the salt file.
/// It returns a vector of website names (without the .bin extension).
pub fn list_passwords(config_dir: &str) -> Result<Vec<String>, SilicateError> {
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
    Ok(websites)
}

/// This function checks if fzf is installed on the system by trying to find its path.
pub fn check_fzf_installed() -> bool {
    which::which("fzf").is_ok()
}

/// This function takes the config directory and an optional tag, lists the passwords, filters them by tag if provided,
pub fn search_password(
    config_dir: &str,
    tag: &Option<String>,
) -> Result<Option<String>, SilicateError> {
    let websites = list_passwords(config_dir)?;
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

pub fn find_password_file(
    config_dir: &str,
    target_website: &str,
) -> Result<Option<String>, SilicateError> {
    let passwords = list_passwords(config_dir)?;

    Ok(passwords.into_iter().find(|filename| {
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
    }))
}

/// This function will export the key from the keyring to a file in the config directory.
/// This is for users who need to backup their key or export a key that was generated on a different machine.
pub fn export_key(file_path: &Option<String>) -> Result<(), SilicateError> {
    let key = retrieve_key_from_keyring()?;
    let path = file_path.as_ref().map_or_else(
        || {
            format!(
                "./key-{}.bin",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
            )
        },
        |p| p.clone(),
    );
    std::fs::write(&path, key).unwrap();
    println!("Key exported to {}", path);
    Ok(())
}

/// This function imports the key from a file and stores it in the keyring.
/// This is for users who need to restore a key from a backup or import a key that was generated on a different machine.
pub fn import_key(file_path: &str) -> Result<(), SilicateError> {
    let key_bytes = std::fs::read(file_path).unwrap();
    let key: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| "Invalid key file: expected 32 bytes")
        .unwrap();
    store_key_in_keyring(&key)?;
    println!("Key imported and stored in keyring.");
    Ok(())
}

/// This function will rename a password file in the config directory.
/// Retagging will be handled in another function.
pub fn rename_password_file(
    config_dir: &str,
    old_website: &str,
    new_website: &str,
) -> Result<(), SilicateError> {
    let old_file_path = find_password_file(config_dir, old_website)?.ok_or_else(|| {
        SilicateError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Password file for '{}' not found", old_website),
        ))
    })?;

    let (_, tag) = if let Some((base, t)) = old_file_path.split_once('-') {
        (base, Some(t))
    } else {
        (old_file_path.as_str(), None)
    };

    let old_file_path = std::path::Path::new(config_dir).join(&old_file_path);

    let new_file_name = if let Some(t) = tag {
        format!("{}-{}.bin", new_website, t)
    } else {
        format!("{}.bin", new_website)
    };

    let new_file_path = std::path::Path::new(config_dir).join(new_file_name);

    std::fs::rename(old_file_path, new_file_path)?;
    Ok(())
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
        let (derived_key, salt) = generate_fallback_key(password).unwrap();
        let derived_key_again = derive_key_from_password(password, &salt).unwrap();
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
