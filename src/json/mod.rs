use serde::{Deserialize, Serialize};
use silicate::SilicateError;

use crate::find_password_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub tag: Option<String>,
    pub ciphertext: String,
}

/// This function reads the secrets from the files in the config directory and returns a vector of Secret structs.
pub fn get_secrets(config_dir: &str, websites: Vec<String>) -> Result<Vec<Secret>, SilicateError> {
    let mut secrets = Vec::new();
    for website in websites {
        // (e.g., "github" or "github-tag")
        let file_identifier = find_password_file(config_dir, &website.to_string())?
            .ok_or("unknown".to_string())
            .unwrap();

        let full_path = format!("{}{}.bin", config_dir, file_identifier);

        let mut split_identifier = file_identifier.split('-');
        let name = split_identifier.next().unwrap().to_string();
        let tag = split_identifier.next().map(|s| s.to_string());

        let binary_data = std::fs::read(&full_path)?;
        let ciphertext = hex::encode(binary_data);

        secrets.push(Secret {
            name,
            tag,
            ciphertext,
        });
    }

    Ok(secrets)
}

/// This function writes the secrets in memory to files in the config directory.
pub fn write_secrets(
    config_dir: &str,
    secrets: Vec<Secret>,
) -> Result<(), Box<dyn std::error::Error>> {
    for secret in secrets {
        let path = if let Some(tag) = secret.tag.clone() {
            format!("{}{}-{}.bin", config_dir, secret.name, tag)
        } else {
            format!("{}{}.bin", config_dir, secret.name)
        };

        let binary_data = hex::decode(&secret.ciphertext)
            .expect("Failed to decode hexadecimal ciphertext during import");

        std::fs::write(&path, binary_data)?;
    }
    Ok(())
}

/// This function exports the secrets in memory to a JSON file.
pub fn export_secrets(
    secrets: Vec<Secret>,
    path: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(&secrets)?;
    let path = path.as_ref().map_or_else(
        || {
            format!(
                "./export-{}.json",
                chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
            )
        },
        |p| p.clone(),
    );
    std::fs::write(&path, json)?;
    println!("Secrets exported to {}", path);
    Ok(())
}

/// This function imports the secrets from a JSON file and returns a vector of Secret structs.
pub fn import_secrets(path: String) -> Result<Vec<Secret>, Box<dyn std::error::Error>> {
    let json = std::fs::read_to_string(&path)?;
    let secrets: Vec<Secret> = serde_json::from_str(&json)?;
    Ok(secrets)
}
