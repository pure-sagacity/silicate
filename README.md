# Silicate

> Offical Website: [https://silicate.maariz.org](https://silicate.maariz.org)

Silicate is a simple command-line password manager written in Rust.

It stores encrypted secrets locally, supports keyring-based key storage when available, and falls back to password-derived keys when necessary. Silicate also provides tag support, search, stats, import/export, and password generation.

## Features

- AES-256-GCM encryption for stored passwords.
- System keyring support for secure key storage.
- Argon2 fallback key derivation when keyring is unavailable.
- Tagging and filtering for secrets.
- Interactive search with `fzf`.
- Import/export of secrets as JSON.
- Password generation with optional symbols.

## Installation

### Nix

Add the inputs to your flake.nix

```nix
{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";

    silicate = {
      url = "git+https://github.com/pure-sagacity/silicate";

      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

### Cargo

```bash
cargo install silicate
```

### Manual

> Silicate is based on Rust, so you'll need to grab a rust installer. If you'd like, you can also use `devenv shell`, which will give you all the tools needed for compilation.

```bash
git clone https://github.com/pure-sagacity/silicate
cd silicate/
cargo build --release
./target/release/silicate --version
```

## Usage

```bash
silicate init
silicate insert apple
silicate insert github -t work
silicate show github
silicate search
silicate generate github --length 16
silicate list
silicate stats
```

## Commands

- `init` — initialize the password store.
- `insert` — store a new password.
- `show` — display or copy a password.
- `delete` — remove a password.
- `edit` — update a password in an editor.
- `rename` — rename a stored entry.
- `search` — search entries with `fzf`.
- `generate` — generate a password, optionally save it.
- `list` — list stored entries.
- `tag` — list tags.
- `stats` — show password statistics.
- `import` — import secrets or a key.
- `export` — export secrets or a key.

## Storage

Passwords are stored in `.silicate/` as encrypted binary files. If keyring support is unavailable, Silicate stores a salt locally and derives the encryption key from your password.

## Security

Never lose your keyring entry or fallback password, or your stored secrets may be unrecoverable.
