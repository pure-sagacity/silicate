const PASSWORD_DIRECTORY: &str = ".silicate/";

use clap::{Parser, Subcommand};
use colored::*;
use rpassword::prompt_password_with_config;
use silicate::*;
use std::process;
use std::string::ToString;
use std::{
    fs,
    io::{self, Read, Write},
};
mod json;
mod tui;
#[derive(Parser)]
#[clap(
    name = "Silicate",
    version,
    about = "A simple command-line password manager."
)]
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

        // Short for tag
        #[clap(long, short = 't')]
        tag: Option<String>,
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

    Search {
        #[clap(long)]
        display: bool,

        #[clap(long, short = 't')]
        tag: Option<String>,
    },

    Generate {
        website: Option<String>,

        #[clap(long, short = 't')]
        tag: Option<String>,

        #[clap(long)]
        length: Option<usize>,

        #[clap(long = "no-symbols")]
        no_symbols: bool,

        #[clap(long)]
        display: bool,
    },

    Edit {
        website: String,
    },

    Rename {
        old_website: String,
        new_website: String,

        #[clap(long, short = 't')]
        tag: Option<String>,
    },

    Import {
        file_path: String,
        #[clap(long = "key", short = 'k')]
        // This is if you are importing a key vs a secrets file
        key: bool,
    },

    Export {
        #[clap(long = "file-path", short = 'f')]
        file_path: Option<String>,

        #[clap(long = "key", short = 'k')]
        // This is if you are exporting a key vs a secrets file
        key: bool,
    },

    Tag {
        #[clap(subcommand)]
        command: TagCommand,
    },

    List {
        #[clap(long, short = 't')]
        tag: Option<String>,
    },

    Stats {},
}

#[derive(Subcommand)]
enum TagCommand {
    List {},
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

    let mut file = match fs::OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            let msg = format!("Failed to open log file at: {}", path)
                .to_string()
                .red();
            println!("{}", msg);
            write_to_logs(&format!("Failed to open log file: {}", e));
            return;
        }
    };
    let msg = msg.to_string() + "\n";

    // Write data
    match file.write_all(msg.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            let msg = format!("Failed to write to log file: {} - Error: {}", path, e)
                .to_string()
                .red();
            println!("{}", msg);
        }
    };
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

fn write_init_timestamp() -> chrono::DateTime<chrono::Utc> {
    let time_init = chrono::Utc::now();

    // Write the init timestamp to ~/.silicate/init_timestamp.txt for stats purposes
    // Non-blocking error, so we will .red().dimmed()

    match fs::write(config_dir() + "init_timestamp.txt", time_init.to_rfc3339()) {
        Ok(_) => (),
        Err(e) => {
            println!("{}", format!("Failed to write initialization timestamp: {}. Check log file (/tmp/silicate.log) for more information.", e).red().dimmed());
            write_to_logs(&format!("Failed to write initialization timestamp: {}", e));
        }
    }

    time_init
}

fn get_key() -> Vec<u8> {
    let init_cmd = "silicate init".to_string().italic();

    match retrieve_key_from_keyring() {
        Ok(k) => k.try_into().unwrap(),
        Err(_) => {
            match fs::exists(config_dir() + "salt.bin") {
                Ok(t) => {
                    if t {
                        let salt = fs::read(config_dir() + "salt.bin").unwrap();
                        let password = get_password("Enter password to derive key: ");
                        let key =
                            match derive_key_from_password(&password, &salt.try_into().unwrap()) {
                                Ok(s) => s,
                                Err(e) => {
                                    let msg =
                                        format!("Failed to read salt for key derivation: {}", e)
                                            .to_string()
                                            .red();
                                    println!("{}", msg);
                                    write_to_logs(&format!(
                                        "Failed to read salt for key derivation: {}",
                                        e
                                    ));
                                    process::exit(1);
                                }
                            };
                        return key.to_vec();
                    } else {
                        let msg = format!(
                            "No key found in keyring or fallback location. Please run `{}` first.",
                            init_cmd
                        )
                        .to_string()
                        .red();
                        println!("{}", msg);
                        write_to_logs(
                            "No key found in keyring or fallback location during get_key.",
                        );
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    let msg = format!(
                        "Failed to check for fallback key: {}. Please run `{}` first.",
                        e, init_cmd
                    )
                    .to_string()
                    .red();
                    println!("{}", msg);
                    write_to_logs(&format!("Failed to check for fallback key: {}", e));
                    std::process::exit(1);
                }
            };
        }
    }
}

fn main() {
    let cli = CLI::parse();
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

    if !fs::exists(config_dir()).unwrap_or(false) {
        println!(
            "{}",
            "No ~/.silicate folder created. You may want to run `silicate init`.".dimmed()
        );
    }

    match &cli.command {
        Some(c) => match c {
            Command::Insert {
                website,
                multiline,
                tag: option_tag,
            } => {
                let password = if *multiline {
                    println!("Enter the password (press Ctrl+D twice to finish):");
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

                if let Some(tag) = option_tag {
                    fs::write(
                        format!("{}{}-{}.bin", config_dir(), website, tag),
                        [nonce_bytes.as_slice(), cipher_bytes.as_slice()].concat(),
                    )
                    .unwrap();
                } else {
                    fs::write(
                        format!("{}{}.bin", config_dir(), website),
                        [nonce_bytes.as_slice(), cipher_bytes.as_slice()].concat(),
                    )
                    .unwrap();
                }
            }
            Command::Delete { website } => {
                let default_letter = "N".to_string().italic().bold();
                print!(
                    "Are you sure you want to delete the password for {website}? (y/{default_letter}): "
                );
                io::stdout().flush().unwrap();
                let mut input = String::new();
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {}
                    Err(_) => {
                        println!("Failed to read input. Deletion implicitly cancelled.");
                        write_to_logs("Failed to read input during deletion confirmation.");
                        return;
                    }
                }

                if input.trim().to_lowercase() == "y" {
                    // 1. Scan the directory using your existing helper to look for a matching name
                    let passwords = silicate::list_passwords(&config_dir());

                    // Find if any entry matches 'website' or starts with 'website-'
                    let target_file = passwords.unwrap().into_iter().find(|filename| {
                        filename == website || filename.starts_with(&format!("{}-", website))
                    });

                    match target_file {
                        Some(filename) => {
                            // 2. Reconstruct the actual filename found on disk
                            let password_path = format!("{}{}.bin", config_dir(), filename);

                            match fs::remove_file(&password_path) {
                                Ok(_) => println!(
                                    "{}",
                                    format!("Password for {website} deleted successfully.").green()
                                ),
                                Err(e) => {
                                    println!(
                                        "{}",
                                        format!("Failed to delete password for {}: {}", website, e)
                                            .red()
                                    );
                                    write_to_logs(&format!(
                                        "Failed to delete password for {website}: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        None => {
                            println!(
                                "{}",
                                format!("Error: No password file found for '{}'.", website).red()
                            );
                        }
                    }
                } else {
                    println!("{}", "Deletion cancelled.".italic().yellow());
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
                    let msg = format!("Password for {}: {}", website, password.bold());
                    println!("{}", msg);
                } else {
                    // Copy to clipboard
                    let mut clipboard = match arboard::Clipboard::new() {
                        Ok(c) => c,
                        Err(e) => {
                            println!("{}", format!("Failed to access clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to access clipboard: {}", e));
                            return;
                        }
                    };
                    match clipboard.set_text(password) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("{}", format!("Failed to copy password to clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to copy password to clipboard: {}", e));
                            return;
                        }
                    }

                    println!("{}", "Password copied to clipboard.".green());

                    let msg = format!(
                        "To show the password, use `silicate show {} --display`",
                        website
                    );

                    println!("{}", msg.dimmed());
                }
            }
            Command::Init {} => {
                let welcome_msg = "Welcome to Silicate.".bold().green();
                let default_letter = "N".to_string().italic().bold();

                // We are going to check if there are any files already existing. After confirming, DELETE
                let config_path = config_dir();

                if fs::exists(&config_path).unwrap_or(false) {
                    let msg = format!(
                        "A configuration directory already exists at {}. Initializing will delete all existing passwords. Do you want to continue? (y/{default_letter}): ",
                        config_path.dimmed()
                    );
                    print!("{}", msg.yellow().bold());
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => {}
                        Err(_) => {
                            println!("Failed to read input. Initialization implicitly cancelled.");
                            write_to_logs(
                                "Failed to read input during initialization confirmation.",
                            );
                            return;
                        }
                    }

                    if input.trim().to_lowercase() != "y" {
                        println!("{}", "Initialization cancelled.".italic().yellow());
                        return;
                    }

                    for entry in fs::read_dir(&config_path).unwrap() {
                        let entry = entry.unwrap();
                        let path = entry.path();
                        if path.is_file() {
                            match fs::remove_file(&path) {
                                Ok(_) => (),
                                Err(e) => {
                                    println!(
                                        "{}",
                                        format!("Failed to delete existing file during initialization: {}", e)
                                            .red()
                                    );
                                    write_to_logs(&format!(
                                        "Failed to delete existing file during initialization: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }
                }
                println!("{}", "Initializing password manager...".dimmed());
                create_dir();
                println!("{}", "Password manager initialized.".dimmed());
                write_to_logs("Password manager initialized.");

                // Generating a new key/password derivation and storing it in the keyring
                if is_keyring_available() {
                    let new_key = generate_key();

                    match store_key_in_keyring(&new_key) {
                        Ok(_) => {
                            write_init_timestamp();
                            println!("Key stored in keyring successfully.\n{}", welcome_msg);
                        }
                        Err(e) => {
                            println!("{}", format!("Failed to store key in keyring: {}", e).red());
                            write_to_logs(&format!("Failed to store key in keyring: {}", e));
                        }
                    }
                } else {
                    print!(
                        "Keyring is not available on this system. Would you like to continue with a password-based key? (y/{default_letter}): "
                    );
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.trim().to_lowercase() != "y" {
                        println!("Exiting...");
                        return;
                    }

                    let password = get_password("Enter a password to derive the encryption key: ");
                    let (_, salt) = match generate_fallback_key(&password) {
                        Ok((_, s)) => ((), s),
                        Err(e) => {
                            println!("{}", format!("Failed to generate fallback key, please check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to generate fallback key: {}", e));
                            return;
                        }
                    };

                    create_dir();
                    fs::write(config_dir() + "salt.bin", &salt).unwrap();

                    write_init_timestamp();

                    println!(
                        "Key derived from password and stored successfully.\n{}",
                        welcome_msg
                    );
                }
            }
            Command::Search {
                display,
                tag: option_tag,
            } => {
                if !check_fzf_installed() {
                    let msg = "fzf is not installed or not found in PATH. Please install fzf to use the search feature.".to_string().red();
                    println!("{}", msg);
                    write_to_logs("fzf not found during search command.");
                    return;
                }

                match silicate::search_password(&config_dir(), option_tag) {
                    Ok(Some(selection)) => {
                        let key = get_key();
                        let data = if let Some(tag) = option_tag {
                            fs::read(format!("{}{}-{}.bin", config_dir(), selection, tag))
                        } else {
                            fs::read(format!("{}{}.bin", config_dir(), selection))
                        }
                        .unwrap();
                        let (nonce_bytes, cipher_bytes) = data.split_at(12);
                        let password = silicate::decrypt_passwd(
                            &key.try_into().unwrap(),
                            cipher_bytes.to_vec(),
                            nonce_bytes.try_into().unwrap(),
                        )
                        .unwrap();

                        if *display {
                            let msg = format!("Password for {}: {}", selection, password.bold());
                            println!("{}", msg);
                        } else {
                            // Copy to clipboard
                            let mut clipboard = match arboard::Clipboard::new() {
                                Ok(c) => c,
                                Err(e) => {
                                    println!("{}", format!("Failed to access clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                    write_to_logs(&format!("Failed to access clipboard: {}", e));
                                    return;
                                }
                            };
                            match clipboard.set_text(password) {
                                Ok(_) => (),
                                Err(e) => {
                                    println!("{}", format!("Failed to copy password to clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                    write_to_logs(&format!(
                                        "Failed to copy password to clipboard: {}",
                                        e
                                    ));
                                    return;
                                }
                            }

                            println!("{}", "Password copied to clipboard.".green());

                            let msg = format!(
                                "To show the password, use `silicate show {} --display`",
                                selection
                            );

                            println!("{}", msg.dimmed());
                        }
                    }
                    Ok(None) => println!("No selection made or selection canceled."),
                    Err(e) => {
                        println!("{}", format!("Error during search: {}", e).red());
                        write_to_logs(&format!("Error during search: {}", e));
                    }
                }
            }
            Command::Generate {
                website,
                tag,
                length,
                no_symbols,
                display,
            } => {
                let symbols = !*no_symbols;

                let length = length.unwrap_or(16); // Default length of 16 if not specified

                let password = silicate::generate_password(length, symbols);

                if let Some(website) = website {
                    let key = get_key();

                    let (cipher_bytes, nonce_bytes) =
                        encrypt_passwd(&key.try_into().unwrap(), password.clone()).unwrap();

                    if let Some(tag) = tag {
                        fs::write(
                            format!("{}{}-{}.bin", config_dir(), website, tag),
                            [nonce_bytes.as_slice(), cipher_bytes.as_slice()].concat(),
                        )
                        .unwrap();
                    } else {
                        fs::write(
                            format!("{}{}.bin", config_dir(), website),
                            [nonce_bytes.as_slice(), cipher_bytes.as_slice()].concat(),
                        )
                        .unwrap();
                    }

                    if *display {
                        let msg =
                            format!("Generated password for {}: {}", website, password.bold());
                        println!("{}", msg);
                    } else {
                        // Copy to clipboard
                        let mut clipboard = match arboard::Clipboard::new() {
                            Ok(c) => c,
                            Err(e) => {
                                println!("{}", format!("Failed to access clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                write_to_logs(&format!("Failed to access clipboard: {}", e));
                                return;
                            }
                        };
                        match clipboard.set_text(password) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("{}", format!("Failed to copy password to clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                write_to_logs(&format!(
                                    "Failed to copy password to clipboard: {}",
                                    e
                                ));
                                return;
                            }
                        }

                        println!("{}", "Generated password copied to clipboard.".green());

                        let msg = format!(
                            "To show the generated password, use `silicate show {} --display`",
                            website
                        );

                        println!("{}", msg.dimmed());
                    }
                } else {
                    if *display {
                        let msg = "Generated password:".dimmed();
                        println!("{msg} {}", password.bold().green());
                    } else {
                        // Copy to clipboard
                        let mut clipboard = match arboard::Clipboard::new() {
                            Ok(c) => c,
                            Err(e) => {
                                println!("{}", format!("Failed to access clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                write_to_logs(&format!("Failed to access clipboard: {}", e));
                                return;
                            }
                        };
                        match clipboard.set_text(password) {
                            Ok(_) => (),
                            Err(e) => {
                                println!("{}", format!("Failed to copy password to clipboard. Check log file (/tmp/silicate.log) for more information.").red());
                                write_to_logs(&format!(
                                    "Failed to copy password to clipboard: {}",
                                    e
                                ));
                                return;
                            }
                        }

                        println!("{}", "Generated password copied to clipboard.".green());

                        println!("{}", "To show the generated password again, use `silicate generate --display`".dimmed());
                    }
                }
            }
            Command::Edit { website } => {
                let file_path_option = match find_password_file(&config_dir(), website) {
                    Ok(path) => path,
                    Err(e) => {
                        println!("{}", format!("Error finding password file for '{}': check log file (/tmp/silicate.log).", website.italic()).red());
                        write_to_logs(&format!(
                            "Error finding password file for '{}': {}",
                            website, e
                        ));
                        return;
                    }
                };

                if let Some(path) = file_path_option {
                    let key_vec = get_key();
                    let key_bytes = key_vec.as_slice();

                    let key: &[u8; 32] = match key_bytes.try_into() {
                        Ok(k) => k,
                        Err(e) => {
                            println!("{}", format!("Error occurred while getting key. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while processing the key: {}",
                                e
                            ));
                            return;
                        }
                    };

                    let new_file =
                        format!("/tmp/{}-{}.tmp", website, chrono::Utc::now().timestamp());

                    // Read the existing password to pre-populate the editor
                    let data = match fs::read(format!("{}{}.bin", config_dir(), path)) {
                        Ok(d) => d,
                        Err(e) => {
                            println!("{}", format!("Error occurred while reading the existing password. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while reading the existing password: {}",
                                e
                            ));
                            return;
                        }
                    };

                    let (nonce_bytes, cipher_bytes) = data.split_at(12);
                    let old_password = match silicate::decrypt_passwd(
                        key,
                        cipher_bytes.to_vec(),
                        nonce_bytes.try_into().unwrap(),
                    ) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("{}", format!("Error occurred while decrypting the existing password. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while decrypting the existing password: {}",
                                e
                            ));
                            return;
                        }
                    };

                    match fs::write(&new_file, old_password) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("{}", format!("Error occurred while writing to temporary file. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while writing to temporary file: {}",
                                e
                            ));
                            return;
                        }
                    }

                    let status = match std::process::Command::new(editor).arg(&new_file).status() {
                        Ok(s) => s,
                        Err(e) => {
                            println!("{}", format!("Error occurred while opening the editor. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while opening the editor: {}",
                                e
                            ));
                            return;
                        }
                    };

                    if !status.success() {
                        println!("{}", format!("Editor exited with an error. Check log file (/tmp/silicate.log) for more information.").red());
                        write_to_logs(&format!(
                            "Editor exited with an error during password update: {}",
                            status
                        ));
                        return;
                    }

                    let new_password = match fs::read_to_string(&new_file) {
                        Ok(p) => p.trim().to_string(),
                        Err(e) => {
                            println!("{}", format!("Error occurred while reading the updated password. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while reading the updated password: {}",
                                e
                            ));
                            return;
                        }
                    };

                    // Delete the temp file
                    match fs::remove_file(&new_file) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("{}", format!("Error occurred while deleting the temporary file. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!(
                                "Error occurred while deleting the temporary file: {}",
                                e
                            ));
                        }
                    }

                    let (new_cipher_bytes, new_nonce_bytes) = match encrypt_passwd(
                        key,
                        new_password,
                    ) {
                        Ok((c, n)) => (c, n),
                        Err(e) => {
                            println!("{}", format!("Error occurred while encrypting the new password. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while encrypting the new password: {}",
                                e
                            ));
                            return;
                        }
                    };

                    let full_path = format!("{}{}.bin", config_dir(), path);

                    match fs::write(
                        full_path,
                        [new_nonce_bytes.as_slice(), new_cipher_bytes.as_slice()].concat(),
                    ) {
                        Ok(_) => (),
                        Err(e) => {
                            println!("{}", format!("Error occurred while writing the updated password to disk. Check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!(
                                "Error occurred while writing the updated password to disk: {}",
                                e
                            ));
                            return;
                        }
                    }

                    println!("{}", "Password updated successfully.".green());
                } else {
                    println!("{}", format!("No password found for '{}'.", website).red());
                }
            }
            Command::Export { file_path, key } => {
                if *key {
                    match export_key(file_path) {
                        Ok(()) => println!("Key exported successfully."),
                        Err(e) => eprintln!("Failed to export key: {}", e),
                    }
                } else {
                    let passwords = match list_passwords(&config_dir()) {
                        Ok(p) => p,
                        Err(e) => {
                            println!("{}", format!("Failed to list passwords for export. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to list passwords for export: {}", e));
                            return;
                        }
                    };

                    let secrets = match json::get_secrets(&config_dir(), passwords) {
                        Ok(s) => s,
                        Err(e) => {
                            println!("{}", format!("Failed to get secrets from passwords. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to get secrets from passwords: {}", e));
                            return;
                        }
                    };

                    match json::export_secrets(secrets, &file_path) {
                        Ok(()) => (),
                        Err(e) => {
                            println!("{}", format!("Failed to export secrets to JSON file. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to export secrets to JSON file: {}", e));
                            return;
                        }
                    }
                }
            }
            Command::Import { file_path, key } => {
                if *key {
                    match import_key(file_path) {
                        Ok(()) => println!("Key imported successfully."),
                        Err(e) => eprintln!("Failed to import key: {}", e),
                    }
                } else {
                    let secrets = match json::import_secrets(file_path.to_string()) {
                        Ok(s) => s,
                        Err(e) => {
                            println!("{}", format!("Failed to import secrets from JSON file. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!(
                                "Failed to import secrets from JSON file: {}",
                                e
                            ));
                            return;
                        }
                    };

                    match json::write_secrets(&config_dir(), secrets) {
                        Ok(()) => println!("Secrets imported successfully."),
                        Err(e) => {
                            println!("{}", format!("Failed to write secrets. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to write secrets: {}", e));
                            return;
                        }
                    }
                }
            }
            Command::Rename {
                old_website,
                new_website,
                tag,
            } => {
                let default_letter = "N".to_string().italic().bold();

                print!(
                    "Are you sure you want to rename the password file for {old_website} to {new_website}? (y/{default_letter}): "
                );
                std::io::stdout().flush().unwrap();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                if input.trim() == "y" || input.trim() == "Y" {
                    match rename_password_file(&config_dir(), old_website, new_website, tag) {
                        Ok(()) => println!(
                            "{}",
                            format!(
                                "Password file renamed from {} to {} successfully.",
                                old_website, new_website
                            )
                            .green()
                        ),
                        Err(e) => {
                            println!("{}", format!("Failed to rename password file, check log file (/tmp/silicate.log).").red());
                            write_to_logs(&format!("Failed to rename password file: {}", e));
                        }
                    };
                }
            }
            Command::Tag { command } => match command {
                TagCommand::List {} => {
                    let tags = match silicate::list_tags(&config_dir()) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("{}", format!("Failed to list tags. Check log file (/tmp/silicate.log) for more information.").red());
                            write_to_logs(&format!("Failed to list tags: {}", e));
                            return;
                        }
                    };

                    if tags.is_empty() {
                        println!("{}", "No tags found.".yellow());
                    } else {
                        println!("{}", "Existing tags:".dimmed());
                        for tag in tags {
                            println!("- {}", tag.bold().italic().bright_green());
                        }
                    }
                }
            },
            Command::List { tag } => {
                let websites =
                    silicate::list_passwords(&config_dir()).expect("Failed to list passwords.");
                if websites.is_empty() {
                    println!("{}", "No passwords stored yet.".yellow());
                } else {
                    println!("Stored passwords for the following websites:");

                    if let Some(tag) = tag {
                        println!(
                            "{}",
                            format!("Filtering: {}", tag.italic().green()).dimmed()
                        );
                    }

                    let websites = if let Some(tag) = tag {
                        websites
                            .into_iter()
                            .filter(|site| site.ends_with(&format!("-{}", tag)))
                            .collect()
                    } else {
                        websites
                    };

                    for site in websites {
                        let (site, tag) = if let Some((s, t)) = site.split_once('-') {
                            (s.to_string(), Some(t.to_string()))
                        } else {
                            (site, None)
                        };
                        if let Some(tag) = tag {
                            let tag = tag.dimmed().italic();
                            println!("{}", format!("- ({}) {}", tag, site.green().bold()));
                        } else {
                            println!("{}", format!("- {}", site.green().bold()));
                        }
                    }

                    println!(
                        "{}",
                        "Use `silicate show <website>` to view a password.".dimmed()
                    );
                }
            }
            Command::Stats {} => {
                let stats = match silicate::get_stats(&config_dir()) {
                    Ok(s) => s,
                    Err(e) => {
                        println!("{}", format!("Failed to get stats. Check log file (/tmp/silicate.log) for more information.").red());
                        write_to_logs(&format!("Failed to get stats: {}", e));
                        return;
                    }
                };

                println!("{}", "Password Statistics:".bold().green());
                let msg = "󰌾 Total Passwords:".dimmed();
                println!(
                    "{} {}",
                    msg,
                    stats.total_passwords.to_string().bold().green()
                );

                let msg = "󰓹 Total Unique Tags".dimmed();
                println!("{} {}", msg, stats.unique_tags.to_string().bold().green());

                let msg = "󱑆 Time Since Initialization".dimmed();
                let duration = chrono::Utc::now() - stats.init_timestamp;
                let duration_str = if duration.num_days() > 0 {
                    format!("{} days", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} minutes", duration.num_minutes())
                } else {
                    format!("{} seconds", duration.num_seconds())
                };
                println!("{} {}", msg, duration_str.bold().green());
            }
        },
        None => {
            let mut terminal = ratatui::init();
            let mut app = tui::App::default();

            match app.run(&mut terminal) {
                Ok(_) => {
                    ratatui::restore();
                }
                Err(e) => {
                    let msg = format!("Failed to open TUI. {}", e).red().bold();
                    println!("{msg}");
                }
            }
        }
    }
}
