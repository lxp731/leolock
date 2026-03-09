
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->


# LeoLock 🔒

A secure file encryption/decryption command-line tool using AES-256-GCM encryption and Argon2id password hashing.

## ✨ Features

- **Military-grade encryption**: AES-256-GCM authenticated encryption
- **Secure password hashing**: Argon2id resistant to GPU/ASIC attacks
- **Interactive operation**: Secure password input (no echo)
- **Recursive processing**: Batch encryption for files and folders
- **Smart error handling**: Single file failure doesn't affect others
- **Secure deletion**: Delete source files by default, optional retention
- **Backup & recovery**: Automatic encrypted backup creation during initialization
- **Tab completion**: Support for Bash, Zsh, Fish

## 📦 Installation

### Compile from source

```bash
# Clone the project
git clone <project-url>
cd leolock

# Build release version
cargo build --release

# Install to system (optional)
sudo cp target/release/leolock /usr/local/bin/
```

### Generate Tab completions

```bash
# Bash
./target/release/leolock --completions bash > ~/.bash_completion.d/leolock

# Zsh
./target/release/leolock --completions zsh > ~/.zsh/completions/_leolock

# Fish
./target/release/leolock --completions fish > ~/.config/fish/completions/leolock.fish
```

## 🚀 Quick Start

### 1. Initialize the tool

```bash
leolock init
```

Initialization will:
- Create config directory `~/.config/leolock/`
- Generate AES-256 key file
- Set initial password
- **Automatically create encrypted backup file** (Keep it safe!)

### 2. Encrypt files

```bash
# Encrypt single file (interactive password input)
leolock encrypt secret.txt
# Output: secret.txt -> secret.txt.leo

# Encrypt folder (recursively process all files)
leolock encrypt documents/

# Encrypt and keep source file
leolock encrypt important.txt --keep-original
```

### 3. Decrypt files

```bash
# Decrypt single file
leolock decrypt secret.txt.leo
# Output: secret.txt.leo -> secret.txt

# Decrypt folder (automatically skip non-encrypted files)
leolock decrypt mixed_folder/

# Decrypt and keep encrypted file
leolock decrypt encrypted.leo --keep-original
```

## 📖 Complete Command Reference

### Initialization & Recovery

| Command | Description |
|---------|-------------|
| `leolock init` | Initialize tool (create config and key) |
| `leolock recover --backup <file>` | Restore key from backup file |

### Password Management

| Command | Description |
|---------|-------------|
| `leolock password update` | Change operation password |

### Key Management

| Command | Description |
|---------|-------------|
| `leolock key update` | Regenerate key (Dangerous operation!) |

### Encryption & Decryption

| Command | Description |
|---------|-------------|
| `leolock encrypt <path>` | Encrypt file or folder |
| `leolock decrypt <path>` | Decrypt file or folder |
| `--keep-original` | Keep source file (do not delete) |

### Help System

| Command | Description |
|---------|-------------|
| `leolock --help` | Show help information |
| `leolock <command> --help` | Show subcommand help |
| `leolock --completions <shell>` | Generate Tab completion scripts |

## 🔐 Security Features

### Cryptographic Algorithms
- **File encryption**: AES-256-GCM (authenticated encryption)
- **Password storage**: Argon2id (random salt, GPU-resistant)
- **Key derivation**: Argon2id (for backup file encryption)

### Security Restrictions
- **Password strength**: At least 8 characters, containing numbers and letters
- **Attempt limit**: Maximum 3 password verification attempts
- **Dangerous path skipping**: Automatically skip system directories like `/bin`, `/usr`, `/lib`
- **Symlink safety**: Encrypt source files, detect circular links

### File Processing
- **Extension preservation**: `a.txt` → `a.txt.leo` → `a.txt`
- **Recursive processing**: Support recursive encryption/decryption of folders
- **Lenient error handling**: Single file failure doesn't affect other files
- **Secure deletion**: Overwrite data before deleting source files by default

## ⚠️ Important Warnings

### 1. Backup Responsibility
- **Automatic backup creation during initialization** - immediately copy backup file to a safe location
- Backup files are encrypted with your password - remember it well
- If you forget the password or lose the backup, **all encrypted data will be permanently lost**

### 2. Key Update Risks
```bash
leolock key update  # ⚠️ Dangerous operation!
```
Regenerating the key will cause:
- All files encrypted with the old key **cannot be decrypted**
- Old backup files **become invalid**
- Must immediately backup the new key

### 3. Password Security
- Use strong passwords (at least 8 characters, containing numbers and letters)
- Change passwords regularly
- Do not share passwords with others

## 📁 File Structure

### User Files
```
~/.config/leolock/
├── leolock.conf      # Configuration file (TOML format)
└── leolock.key       # AES-256 key file

~/leolock_key_backup_YYYYMMDD_HHMMSS.enc  # Encrypted backup file
```

### Configuration File Example
```toml
suffix = ".leo"
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$...$..."
salt = "base64-encoded random salt"
```

## 🧪 Testing

The project includes a complete test suite:

```bash
# Run all tests
cargo test

# Run specific tests
cargo test test_encryption
cargo test test_password
```

Test directory: `/home/knight/test/`

## 🔧 Development

### Project Structure
```
leolock/
├── Cargo.toml                    # Project configuration
├── src/                          # Source code
│   ├── main.rs                   # CLI entry point
│   ├── config.rs                 # Configuration management
│   ├── crypto.rs                 # AES encryption
│   ├── keymgmt.rs                # Key management
│   ├── fileops.rs                # File operations
│   ├── password.rs               # Password handling
│   ├── errors.rs                 # Error handling
│   └── utils.rs                  # Utility functions
└── README.md                     # This documentation
```

### Building
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Code checking
cargo check
cargo clippy
```

## 📄 License

MIT License

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📞 Support

If you have questions, please:
1. Check this documentation
2. Run `leolock --help`
3. Submit an Issue

---

**Security Note**: Regularly backup important data - encryption is not insurance against data loss.