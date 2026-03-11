
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [LeoLock 🔒](#leolock-)
  - [🚀 Latest Architecture (2026-03-11)](#-latest-architecture-2026-03-11)
    - [Major Improvements](#major-improvements)
  - [✨ Features](#-features)
  - [📦 Installation](#-installation)
    - [Debian/Ubuntu/RHEL/Fedora](#debianubunturhelfedora)
    - [AUR](#aur)
    - [Compile from source](#compile-from-source)
    - [Generate Tab completions](#generate-tab-completions)
  - [🚀 Quick Start](#-quick-start)
    - [1. Initialize the tool](#1-initialize-the-tool)
    - [2. Encrypt files](#2-encrypt-files)
    - [3. Decrypt files](#3-decrypt-files)
  - [📖 Complete Command Reference](#-complete-command-reference)
    - [Initialization & Recovery](#initialization--recovery)
    - [Password Management](#password-management)
    - [Key Management](#key-management)
    - [Configuration Management](#configuration-management)
    - [Encryption & Decryption](#encryption--decryption)
    - [Shell Completions](#shell-completions)
    - [Help System](#help-system)
  - [🔐 Security Features](#-security-features)
    - [Cryptographic Algorithms](#cryptographic-algorithms)
    - [Security Restrictions](#security-restrictions)
    - [File Processing](#file-processing)
    - [Configuration Management](#configuration-management-1)
      - [Default Configuration](#default-configuration)
      - [Configuration Features](#configuration-features)
  - [⚠️ Important Warnings](#️-important-warnings)
    - [1. Backup Responsibility](#1-backup-responsibility)
    - [2. Key Update Risks](#2-key-update-risks)
    - [3. Password Security](#3-password-security)
    - [4. Initialization Requirement](#4-initialization-requirement)
  - [📁 File Structure](#-file-structure)
    - [User Files](#user-files)
    - [Configuration File Example](#configuration-file-example)
    - [Project Structure](#project-structure)
  - [📄 License](#-license)
  - [🤝 Contributing](#-contributing)
  - [📞 Support](#-support)
  - [📝 Version History](#-version-history)

<!-- /code_chunk_output -->



# LeoLock 🔒

A secure file encryption/decryption command-line tool using AES-256-GCM encryption and Argon2id password hashing.

## 🚀 Latest Architecture (2026-03-11)

### Major Improvements
1. **Unified Configuration System** - Merges dangerous path configuration with password/key configuration
2. **Configuration Management Commands** - `config show` / `config validate` subcommands
3. **Optimized Initialization Flow** - `init` performs complete initialization (password + key + configuration)
4. **Completion Command Improvement** - `complete` subcommand replaces `--completions` option
5. **Enhanced Security Design** - Encryption operations require initialization

## ✨ Features

- **Military-grade encryption**: AES-256-GCM authenticated encryption
- **Secure password hashing**: Argon2id resistant to GPU/ASIC attacks
- **Interactive operation**: Secure password input (no echo)
- **Recursive processing**: Supports batch encryption of files and folders
- **Intelligent error handling**: Single file failure doesn't affect other files
- **Secure deletion**: Deletes source files by default, optional retention
- **Backup recovery**: Automatically creates encrypted backup during initialization
- **Tab completion**: Supports Bash, Zsh, Fish
- **Unified configuration**: Dangerous paths, file size limits, and other security settings are configurable
- **Environment variable override**: Runtime configuration can be overridden via environment variables
- **23 Dangerous Paths** - Protects critical system directories
- **10GB File Size Limit** - Prevents accidental encryption of large files
- **Configurability** - All security settings are customizable
- **Automatic Version Sync** - Only modify `Cargo.toml`, `main.rs` reads automatically

## 📦 Installation

### Debian/Ubuntu/RHEL/Fedora

```bash
# Install .deb or .rpm package from github release.
```

### AUR

```bash
yay -S leolock
```

### Compile from source

```bash
# Clone the project
git clone https://github.com/lxp731/leolock.git
cd leolock

# Compile release version
cargo build --release

# Install to system (optional)
sudo cp target/release/leolock /usr/local/bin/
```

### Generate Tab completions

```bash
# Bash
leolock complete bash > /usr/share/bash-completion/completions/leolock

# Zsh
leolock complete zsh > /usr/share/zsh/site-functions/_leolock

# Fish
leolock complete fish > /usr/share/fish/vendor_completions.d/leolock.fish

# 使用 AUR 默认会安装了补全脚本
# Bash
/usr/share/bash-completion/completions/leolock

# Zsh
/usr/share/zsh/site-functions/_leolock

# Fish
/usr/share/fish/vendor_completions.d/leolock.fish
```

## 🚀 Quick Start

### 1. Initialize the tool

```bash
leolock init
```

Initialization will:
- Create configuration directory `~/.config/leolock/`
- Generate AES-256 key file
- Set initial password
- Create default configuration file (includes 23 dangerous paths)
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

# Decrypt folder (automatically skips non-encrypted files)
leolock decrypt mixed_folder/

# Decrypt and keep encrypted file
leolock decrypt encrypted.leo --keep-original
```

## 📖 Complete Command Reference

### Initialization & Recovery

| Command | Description |
|---------|-------------|
| `leolock init` | Initialize the tool (create configuration and keys) |
| `leolock recover --backup <file>` | Restore keys from backup file |

### Password Management

| Command | Description |
|---------|-------------|
| `leolock password update` | Change operation password |

### Key Management

| Command | Description |
|---------|-------------|
| `leolock key update` | Regenerate keys (Dangerous operation!) |

### Configuration Management

| Command | Description |
|---------|-------------|
| `leolock config show` | Show current configuration |
| `leolock config validate` | Validate configuration file |

### Encryption & Decryption

| Command | Description |
|---------|-------------|
| `leolock encrypt <path>` | Encrypt file or folder |
| `leolock decrypt <path>` | Decrypt file or folder |
| `--keep-original` | Keep source file (do not delete) |

### Shell Completions

| Command | Description |
|---------|-------------|
| `leolock complete bash` | Generate Bash completion script |
| `leolock complete zsh` | Generate Zsh completion script |
| `leolock complete fish` | Generate Fish completion script |
| `leolock complete powershell` | Generate PowerShell completion script |
| `leolock complete elvish` | Generate Elvish completion script |

### Help System

| Command | Description |
|---------|-------------|
| `leolock --help` | Show help information |
| `leolock <command> --help` | Show subcommand help |

## 🔐 Security Features

### Cryptographic Algorithms
- **File encryption**: AES-256-GCM (authenticated encryption)
- **Password storage**: Argon2id (random salt, GPU-resistant)
- **Key derivation**: Argon2id (for backup file encryption)

### Security Restrictions
- **Password strength**: At least 8 characters, containing numbers and letters
- **Attempt limit**: Password verification limited to 3 attempts
- **Dangerous path protection**: Default includes 23 system directories, encryption prohibited
- **Maximum file size**: Default 10GB limit, prevents accidental encryption of large files
- **Initialization requirement**: Must initialize before encryption operations

### File Processing
- **Extension preservation**: `a.txt` → `a.txt.leo` → `a.txt`
- **Recursive processing**: Supports recursive encryption/decryption of folders
- **Lenient error handling**: Single file failure doesn't affect other files
- **Secure deletion**: Overwrites data then deletes source file by default
- **Symbolic link safety**: Encrypts source files, detects circular links

### Configuration Management
#### Default Configuration
```toml
# ~/.config/leolock/config.toml
# Dangerous path list (system directories prohibited from processing)
forbidden_paths = [
    "/bin", "/sbin", "/usr/bin", "/usr/sbin",
    "/lib", "/lib64", "/usr/lib", "/usr/lib64",
    "/boot", "/dev", "/proc", "/sys", "/run",
    "/etc", "/root", "/var", "/tmp",
    "/usr/local/bin", "/usr/local/sbin",
    "/opt", "/home", "/mnt", "/media",
]

# Maximum file size (bytes), 0 means unlimited
max_file_size = 10737418240  # 10GB

# Whether to enable progress display
show_progress = true

# Default encrypted file extension
default_extension = ".leo"

# Key file location (supports ~ expansion)
key_file_path = "~/.config/leolock/keys.toml"
```

#### Configuration Features
1. **Security first**: Default includes all critical system directories as dangerous paths
2. **Environment variable override**: 
   - `LEOLOCK_FORBIDDEN_PATHS`: Comma-separated list of dangerous paths
   - `LEOLOCK_MAX_FILE_SIZE`: Maximum file size in bytes
3. **Configuration file search paths** (in priority order):
   - `.leolock.toml` (current directory)
   - Path specified by `LEOLOCK_CONFIG` environment variable
   - `~/.config/leolock/config.toml` (XDG config directory)
   - `~/.leolock.toml` (user home directory)
4. **Sensitive information protection**: Password hashes and salts are not saved to configuration file
5. **Initialization state**: Automatically detects if initialized when loading configuration

## ⚠️ Important Warnings

### 1. Backup Responsibility
- **Automatically creates backup during initialization**, immediately copy backup file to secure location
- Backup file is encrypted with your password, remember your password
- If you forget password or lose backup, **all encrypted data will be permanently lost**

### 2. Key Update Risks
```bash
leolock key update  # ⚠️ Dangerous operation!
```
Regenerating keys will cause:
- All files encrypted with old key **cannot be decrypted**
- Old backup files **become invalid**
- Must immediately backup new key

### 3. Password Security
- Use strong passwords (at least 8 characters, containing numbers and letters)
- Change password regularly
- Do not share password with others

### 4. Initialization Requirement
- Must first run `leolock init` to complete initialization
- Uninitialized tool cannot perform encryption/decryption operations
- Initialization state is saved in configuration

## 📁 File Structure

### User Files
```
~/.config/leolock/
├── config.toml      # Configuration file (dangerous paths, file size, etc.)
└── keys.toml        # Key file (AES-256 keys)

~/leolock_key_backup_YYYYMMDD_HHMMSS.enc  # Encrypted backup file
```

### Configuration File Example
Complete configuration example in `examples/config.toml`.

### Project Structure
```
leolock/
├── Cargo.toml                    # Project configuration
├── CREATE.md                     # Requirements specification (merged)
├── examples/                     # Example files
│   └── config.toml               # Example configuration file
├── src/                          # Source code
│   ├── main.rs                   # CLI entry and command parsing
│   ├── config.rs                 # Unified configuration management
│   ├── crypto.rs                 # AES-256-GCM encryption/decryption
│   ├── keymgmt.rs                # Key management (generation, backup, recovery)
│   ├── fileops.rs                # File operations (recursive, dangerous path checking)
│   ├── password.rs               # Password handling (Argon2id, interactive)
│   ├── errors.rs                 # Error type definitions
│   └── utils.rs                  # Utility functions (confirmation, salt generation, secure deletion)
└── README.md                     # This documentation
```

## 📄 License

<a href="https://github.com/lxp731/leolock/blob/main/LICENSE" alt="MIT LICENSE">
    <p style="color: black">MIT LICENSE</p>
</a>

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📞 Support

If you have questions, please:
1. Check this documentation
2. Run `leolock --help`
3. Submit an Issue

## 📝 Version History

- **2026-03-09**: Initial requirements (TASK.md)
- **2026-03-09**: Updated requirements (NEW_TASK.md)
- **2026-03-09**: Final requirements (CREATE.md) - Initial version
- **2026-03-11**: Architecture refactoring and feature enhancements (README.md)
  - ✅ Unified configuration system: Merges dangerous path configuration with password/key configuration
  - ✅ Configuration management commands: `leolock config show` / `leolock config validate`
  - ✅ Optimized initialization flow: `leolock init` performs complete initialization
  - ✅ Completion command improvement: `leolock complete` replaces `--completions` option
  - ✅ Enhanced security design: Encryption operations require initialization
  - ✅ Code cleanup: Removed all unused functions and imports
  - ✅ Default configuration: Includes 23 dangerous paths, 10GB file size limit
  - ✅ Automatic version sync: Only modify `Cargo.toml`, `main.rs` reads automatically
- **Status**: ✅ All development and testing completed, architecture stable

---

**Last Updated**: 2026-03-11  
**Author**: Burgess Leo  
**Status**: ✅ Requirements clear, project completed, architecture stable

**Security Note**: Regularly backup important data - encryption is not insurance against data loss.