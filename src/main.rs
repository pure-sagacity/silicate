const PASSWORD_DIRECTORY: &str = ".silicate/";

use std::{
    fs,
    io::{self, Read, Write},
};

use clap::{Parser, Subcommand};
use rpassword::prompt_password_with_config;
use silicate::{
    derive_key_from_password, encrypt_passwd, generate_fallback_key, generate_key,
    is_keyring_available, retrieve_key_from_keyring, store_key_in_keyring,
};
#[derive(Parser)]
struct CLI {
    #[clap(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    Insert {
        website: String,

        #[clap(long)]
        multiline: bool,
    },

    Delete {
        website: String,
    },

    Show {
        website: String,

        #[clap(long)]
        display: bool,
    },

    Init {},
}

fn get_password(prompt: &str) -> String {
    loop {
        match prompt_password_with_config(prompt, config()) {
            Ok(p) => break p,
            Err(e) => {
                println!("Error reading password: {}", e);
                write_to_logs(&format!("Error reading password for key derivation: {}", e));
            }
        }
    }
}

fn create_dir() {
    match fs::create_dir_all(format!(
        "{}/{}",
        std::env::var("HOME").unwrap(),
        PASSWORD_DIRECTORY
    )) {
        Ok(_) => return,
        Err(e) => {
            println!(
                "Failed to create password directory: {}/{}",
                std::env::var("HOME").unwrap(),
                PASSWORD_DIRECTORY
            );

            write_to_logs(&format!(
                "Failed to create password directory: {} - Error: {}",
                config_dir(),
                e
            ));
        }
    }
}

fn write_to_logs(msg: &str) {
    let path = "/tmp/silicate.log";

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("Failed to open file");

    let msg = msg.to_string() + "\n";

    // Write data
    file.write_all(msg.as_bytes())
        .expect("Failed to write to file");
}

fn config() -> rpassword::Config {
    let config = rpassword::ConfigBuilder::new()
        .password_feedback_mask('*')
        .build();
    config
}

fn config_dir() -> String {
    format!("{}/{}/", std::env::var("HOME").unwrap(), PASSWORD_DIRECTORY)
}

fn get_key() -> Vec<u8> {
    match retrieve_key_from_keyring() {
        Ok(k) => k.try_into().unwrap(),
        Err(_) => {
            match fs::exists(config_dir() + "salt.bin") {
                Ok(t) => {
                    if t {
                        let salt = fs::read(config_dir() + "salt.bin").unwrap();
                        let password = get_password("Enter password to derive key: ");
                        let key = derive_key_from_password(&password, &salt.try_into().unwrap());
                        return key.to_vec();
                    } else {
                        println!(
                            "No key found in keyring or fallback location. Please run `silicate init` first."
                        );
                        write_to_logs(
                            "No key found in keyring or fallback location during get_key.",
                        );
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    println!(
                        "Failed to check for fallback key: {}. Please run `silicate init` first.",
                        e
                    );
                    write_to_logs(&format!("Failed to check for fallback key: {}", e));
                    std::process::exit(1);
                }
            };
        }
    }
}

fn main() {
    let cli = CLI::parse();
    match &cli.command {
        Some(c) => match c {
            Command::Insert { website, multiline } => {
                let password = if *multiline {
                    println!("Enter the password (press Ctrl+D to finish):");
                    let mut password = String::new();
                    io::stdin().read_to_string(&mut password).unwrap();
                    password = password.trim().to_string();
                    password
                } else {
                    let password = get_password("Enter the password: ");
                    password
                };

                let key = get_key();

                let (cipher_bytes, nonce_bytes) =
                    encrypt_passwd(&key.try_into().unwrap(), password).unwrap();

                fs::write(
                    format!("{}{}.bin", config_dir(), website),
                    [nonce_bytes.as_slice(), cipher_bytes.as_slice()].concat(),
                )
                .unwrap();
            }
            Command::Delete { website } => {
                print!("Are you sure you want to delete the password for {website}? (y/N): ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    match fs::remove_file(format!("{}{}.bin", config_dir(), website)) {
                        Ok(_) => println!("Password for {website} deleted successfully."),
                        Err(e) => {
                            println!("Failed to delete password for {website}: {}", e);
                            write_to_logs(&format!(
                                "Failed to delete password for {website}: {}",
                                e
                            ));
                        }
                    };
                } else {
                    println!("Deletion cancelled.");
                }
            }
            Command::Show { website, display } => {
                println!("Retrieving password for: {}", website);
                let key = get_key();

                let data = fs::read(format!("{}{}.bin", config_dir(), website)).unwrap();
                let (nonce_bytes, cipher_bytes) = data.split_at(12);
                let password = silicate::decrypt_passwd(
                    &key.try_into().unwrap(),
                    cipher_bytes.to_vec(),
                    nonce_bytes.try_into().unwrap(),
                )
                .unwrap();
                if *display {
                    println!("Password for {}: {}", website, password);
                } else {
                    // Copy to clipboard
                    let mut clipboard =
                        arboard::Clipboard::new().expect("Failed to copy password.");
                    clipboard
                        .set_text(password)
                        .expect("Failed to copy password.");

                    println!(
                        "To show the password, use `silicate show {} --display`",
                        website
                    );
                }
            }
            Command::Init {} => {
                println!("Initializing password manager...");
                create_dir();
                println!("Password manager initialized.");
                write_to_logs("Password manager initialized.");

                // Generating a new key/password derivation and storing it in the keyring
                if is_keyring_available() {
                    let new_key = generate_key();

                    match store_key_in_keyring(&new_key) {
                        Ok(_) => {
                            println!("Key stored in keyring successfully.\nWelcome to Silicate.")
                        }
                        Err(e) => {
                            println!("Failed to store key in keyring: {}", e);
                            write_to_logs(&format!("Failed to store key in keyring: {}", e));
                        }
                    }
                } else {
                    print!(
                        "Keyring is not available on this system. Would you like to continue with a password-based key? (y/N: "
                    );
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        println!("Exiting...");
                        return;
                    }

                    let password = get_password("Enter a password to derive the encryption key: ");
                    let (_, salt) = generate_fallback_key(&password);

                    create_dir();
                    fs::write(config_dir() + "salt.bin", &salt).unwrap();

                    println!(
                        "Key derived from password and stored successfully.\nWelcome to Silicate."
                    );
                }
            }
        },
        None => {
            println!("Listing all stored passwords...");

            let websites = silicate::list_passwords(&config_dir());
            if websites.is_empty() {
                println!("No passwords stored yet.");
            } else {
                println!("Stored passwords for the following websites:");
                for site in websites {
                    println!("- {}", site);
                }
            }
        }
    }
}
