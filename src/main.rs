const PASSWORD_DIRECTORY: &str = ".silicate/";

use clap::{Parser, Subcommand};
use colored::*;
use rpassword::prompt_password_with_config;
use silicate::*;
use std::string::ToString;
use std::{
    fs,
    io::{self, Read, Write},
};
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
        website: String,

        #[clap(long, short = 't')]
        tag: Option<String>,

        #[clap(long)]
        length: Option<usize>,

        #[clap(long = "no-symbols")]
        no_symbols: bool,

        #[clap(long)]
        display: bool,
    },
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
    match file.write_all(msg.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            let msg = format!("Failed to write to log file: {} - Error: {}", path, e)
                .to_string()
                .red();
            println!("{}", msg);
        }
    }
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
    let init_cmd = "silicate init".to_string().italic();

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
                io::stdin().read_line(&mut input).unwrap();
                if input.trim().to_lowercase() == "y" {
                    match fs::remove_file(format!("{}{}.bin", config_dir(), website)) {
                        Ok(_) => println!(
                            "{}",
                            format!("Password for {website} deleted successfully.").green()
                        ),
                        Err(e) => {
                            println!(
                                "{}",
                                format!("Failed to delete password for {}: {}", website, e).red()
                            );
                            write_to_logs(&format!(
                                "Failed to delete password for {website}: {}",
                                e
                            ));
                        }
                    };
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
                    let mut clipboard =
                        arboard::Clipboard::new().expect("Failed to copy password.");
                    clipboard
                        .set_text(password)
                        .expect("Failed to copy password.");

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

                println!("{}", "Initializing password manager...".dimmed());
                create_dir();
                println!("{}", "Password manager initialized.".dimmed());
                write_to_logs("Password manager initialized.");

                // Generating a new key/password derivation and storing it in the keyring
                if is_keyring_available() {
                    let new_key = generate_key();

                    match store_key_in_keyring(&new_key) {
                        Ok(_) => {
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
                    let (_, salt) = generate_fallback_key(&password);

                    create_dir();
                    fs::write(config_dir() + "salt.bin", &salt).unwrap();

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
                            let mut clipboard =
                                arboard::Clipboard::new().expect("Failed to copy password.");
                            clipboard
                                .set_text(password)
                                .expect("Failed to copy password.");

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
                    let msg = format!("Generated password for {}: {}", website, password.bold());
                    println!("{}", msg);
                } else {
                    // Copy to clipboard
                    let mut clipboard =
                        arboard::Clipboard::new().expect("Failed to copy password.");
                    clipboard
                        .set_text(password)
                        .expect("Failed to copy password.");

                    println!("{}", "Generated password copied to clipboard.".green());

                    let msg = format!(
                        "To show the generated password, use `silicate show {} --display`",
                        website
                    );

                    println!("{}", msg.dimmed());
                }
            }
        },
        None => {
            let websites = silicate::list_passwords(&config_dir());
            if websites.is_empty() {
                println!("{}", "No passwords stored yet.".yellow());
            } else {
                println!("Stored passwords for the following websites:");
                for site in websites {
                    let (site, tag) = if let Some((s, t)) = site.split_once('-') {
                        (s.to_string(), Some(t.to_string()))
                    } else {
                        (site, None)
                    };
                    if let Some(tag) = tag {
                        let tag = tag.dimmed().italic();
                        println!("{}", format!("- ({}) {}", tag, site));
                    } else {
                        println!("{}", format!("- {}", site));
                    }
                }

                println!(
                    "{}",
                    "Use `silicate show <website>` to view a password.".dimmed()
                );
            }
        }
    }
}
