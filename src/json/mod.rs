use serde::{Deserialize, Serialize};

use crate::find_password_file;

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    pub name: String,
    pub tag: Option<String>,
    pub ciphertext: String,
}

/// This function reads the secrets from the files in the config directory and returns a vector of Secret structs.
pub fn get_secrets(config_dir: &str, websites: Vec<String>) -> Vec<Secret> {
    let mut secrets = Vec::new();
    for website in websites {
        let path = find_password_file(config_dir, &website.to_string()).unwrap();

        let (name, tag) = path.split_once("-").unwrap();

        let path = format!("{}{}", config_dir, path);

        let ciphertext = std::fs::read_to_string(&path).unwrap();

        secrets.push(Secret {
            name: name.to_string(),
            tag: Some(tag.to_string()),
            ciphertext,
        });
    }

    secrets
}

/// This function writes the secrets in memory to files in the config directory.
pub fn write_secrets(config_dir: &str, secrets: Vec<Secret>) {
    for secret in secrets {
        let path = format!(
            "{}{}-{}",
            config_dir,
            secret.name,
            secret.tag.clone().unwrap_or_default()
        );
        std::fs::write(&path, secret.ciphertext).unwrap();
    }
}

/// This function exports the secrets in memory to a JSON file.
pub fn export_secrets(secrets: Vec<Secret>, path: Option<String>) {
    let json = serde_json::to_string_pretty(&secrets).unwrap();
    let path = path.unwrap_or_else(|| {
        format!(
            "./export-{}.json",
            chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S")
        )
    });
    std::fs::write(&path, json).unwrap();
    println!("Secrets exported to {}", path);
}

/// This function imports the secrets from a JSON file and returns a vector of Secret structs.
pub fn import_secrets(path: String) -> Vec<Secret> {
    let json = std::fs::read_to_string(&path).unwrap();
    let secrets: Vec<Secret> = serde_json::from_str(&json).unwrap();
    secrets
}
