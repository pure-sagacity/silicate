const PASSWORD_DIRECTORY: &str = ".silicate/";

use std::{fs, io::Write};

use clap::{Parser, Subcommand};
use rpassword::prompt_password_with_config;
use silicate::encrypt_passwd;
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
    },

    Init {},
}

fn gen_or_get_key() -> [u8; 32] {
    [0; 32]
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
                "Failed to create password directory: {}/{} - Error: {}",
                std::env::var("HOME").unwrap(),
                PASSWORD_DIRECTORY,
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

fn main() {
    let cli = CLI::parse();
    match &cli.command {
        Some(c) => match c {
            Command::Insert { website, multiline } => {
                let password = loop {
                    match prompt_password_with_config(
                        &format!("Inserting password for {}: ", website),
                        config(),
                    ) {
                        Ok(p) => break p,
                        Err(e) => {
                            println!("Error reading password: {}", e);
                            write_to_logs(&format!(
                                "Error reading password for {}: {}",
                                website, e
                            ));
                        }
                    }
                };
                
                let encrypted_password = encrypt_passwd(key_bytes, plaintext)
            }
            Command::Delete { website } => {
                println!("Deleting password for: {}", website);
            }
            Command::Show { website } => {
                println!("Showing password for: {}", website);
                // Here you would call your show function and handle the result
            }
            Command::Init {} => {
                println!("Initializing password manager...");
                create_dir();
                println!("Password manager initialized.");
                write_to_logs("Password manager initialized.");
            }
        },
        None => {
            // Here we will list all stored passwords
            println!("Listing all stored passwords...");
        }
    }
}
