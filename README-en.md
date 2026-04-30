# LeoLock 🔒

A secure file encryption/decryption command-line tool using AES-256-GCM encryption and Argon2id password hashing.

## ✨ Features

- **Military-grade encryption**: AES-256-GCM authenticated encryption, now with **AAD (Additional Authenticated Data)** to protect headers from tampering.
- **Secure password hashing**: Argon2id resistant to GPU/ASIC attacks.
- **Zero Secrets in Memory**: Integrated `zeroize` technology ensures passwords and keys are cleared from memory immediately after use.
- **Extreme Performance**: Refactored with streaming I/O; encrypts a 3GB file in ~14 seconds (benchmarked) with minimal memory footprint.
- **Atomic Operations**: "Write-then-swap" mechanism ensures original data remains intact even if a crash or power failure occurs during encryption.
- **Dual Encryption Modes**: Supports filename encryption (Full mode) or content-only encryption (Fast mode).
- **Recursive Processing**: Supports batch encryption of files and folders.
- **Backup Recovery**: Automatically creates an encrypted key backup during initialization.
- **Tab Completion**: Supports Bash, Zsh, Fish, PowerShell, Elvish.
- **Runtime Security Checks**: Automatically detects config file permissions (600 required).

## 🚀 Quick Start

### 1. Installation

**Compile from source (recommended):**
```bash
git clone https://github.com/lxp731/leolock.git
cd leolock
cargo build --release
# Use the release version for maximum performance
sudo cp target/release/leolock /usr/local/bin/
```

**Or use package manager:** See [docs/INSTALLATION.md](docs/INSTALLATION.md)

### 2. Initialization
```bash
leolock init
```
Set password, generate configuration and keys.

### 3. Encrypt files
```bash
# Full mode (default): Encrypts both file content and filename
leolock encrypt secret.txt

# Fast mode: Encrypts only file content, preserves filename
leolock encrypt secret.txt --fast

# Keep original file
leolock encrypt secret.txt --keep
```
Enter password, file is encrypted as:
- Full mode: `random_hash.leo` (filename encrypted)
- Fast mode: `secret.txt.leo` (original filename preserved)

### 4. Decrypt files
```bash
leolock decrypt secret.txt.leo
```
Enter password, restore original file (automatically detects file format).

### 5. View files
```bash
# List encrypted files
leolock list .

# Sort by size
leolock list . --sort-by-size desc

# Show original filename (requires password)
leolock list . --show-original
```

## 📖 Basic Commands

| Command | Description |
|------|------|
| `leolock init` | Initialize the tool |
| `leolock encrypt <path>` | Encrypt file or folder |
| `leolock decrypt <path>` | Decrypt file or folder |
| `leolock list <path>` | List encrypted file information |
| `leolock recover --backup <file>` | Restore key from backup file |
| `leolock completions <shell>` | Generate shell completion scripts |

**Common options:**
- `-k, --keep`: Keep source file (do not delete)
- `-F, --fast`: Fast mode (do not encrypt filename)
- `--show-original`: Show original filename (requires password)
- `--sort-by-size <asc/desc>`: Sort by file size

**Complete command reference:** See [docs/COMMANDS.md](docs/COMMANDS.md)

## 📦 Installation Options

### Compile from source (recommended)
```bash
cargo build --release
sudo cp target/release/leolock /usr/local/bin/
```

### Generate completion scripts

**Bash:**
```bash
leolock completions bash -o ~/.bash_completion.d/
```

**Zsh:**
```bash
leolock completions zsh -o ~/.zsh/completions/
```

**Other shells:** See [docs/INSTALLATION.md](docs/INSTALLATION.md)

**Detailed installation guide:** See [docs/INSTALLATION.md](docs/INSTALLATION.md)

## 🔐 Security Features

### Core Security
- **AES-256-GCM**: Military-grade authenticated encryption
- **Argon2id**: GPU/ASIC-resistant password hashing
- **Random salt**: Unique per instance, prevents rainbow table attacks
- **File permission protection**: Automatically sets configuration file permissions to 600

### Security Restrictions
- **Dangerous path protection**: Default prohibits encryption of 16 system directories
- **File size limit**: Default 10GB, prevents accidental encryption of large files
- **Password strength**: Minimum 8 characters, containing numbers and letters
- **Runtime checks**: Automatically detects configuration file permission issues

**Detailed security documentation:** See [docs/SECURITY.md](docs/SECURITY.md)

## 📁 Documentation Directory

- [docs/INSTALLATION.md](docs/INSTALLATION.md) - Detailed installation guide
- [docs/COMMANDS.md](docs/COMMANDS.md) - Complete command reference
- [docs/SECURITY.md](docs/SECURITY.md) - Security features documentation
- [docs/CONFIGURATION.md](docs/CONFIGURATION.md) - Configuration file documentation
- [docs/WARNINGS.md](docs/WARNINGS.md) - Important warnings
- [docs/STRUCTURE.md](docs/STRUCTURE.md) - File structure documentation

## ⚠️ Important Reminders

1. **Backup is crucial**: Automatic backup created during initialization, immediately transfer to secure location
2. **Remember password**: Forgetting password will cause permanent loss of all encrypted data
3. **File permissions**: Configuration files contain sensitive information, keep permissions at 600

**Complete warning list:** See [docs/WARNINGS.md](docs/WARNINGS.md)

## 📝 Version History

### Version 1.1.0 (Current)
- **Performance Breakthrough**: Refactored with streaming encryption, significantly boosting speed (14s/3GB).
- **Memory Security**: Integrated `zeroize` to ensure no sensitive data remains in memory.
- **Integrity Boost**: Upgraded to V3 file format with AAD (Additional Authenticated Data) for header protection.
- **Robustness**: Implemented atomic file writing to prevent data corruption from interrupted operations.

### Version 1.0.3
- Simplified password management, removed separate password hash file
- Added file listing with sorting and original filename display

**Complete version history:** See [docs/CHANGELOG.md](docs/CHANGELOG.md)

## 📄 License

MIT License - See [LICENSE](LICENSE)

## 🤝 Contributing

Issues and Pull Requests are welcome!

## 📞 Support

If you have questions, please:
1. Check this documentation and docs/ directory
2. Run `leolock --help`
3. Submit [GitHub Issue](https://github.com/lxp731/leolock/issues)

---

**Last Updated:** 2026-04-30  
**Project Status:** ✅ Hardened, High Performance, Production Ready

**Security Note:** Regularly backup important data, encryption is not insurance against data loss.