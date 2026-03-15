# LeoLock 🔒

A secure file encryption/decryption command-line tool using AES-256-GCM encryption and Argon2id password hashing.

## ✨ Features

- **Military-grade encryption**: AES-256-GCM authenticated encryption
- **Secure password hashing**: Argon2id resistant to GPU/ASIC attacks
- **Interactive operation**: Secure password input (no echo)
- **Recursive processing**: Supports batch encryption of files and folders
- **Intelligent error handling**: Single file failure doesn't affect other files
- **Secure deletion**: Deletes source files by default, optional retention
- **Backup recovery**: Automatically creates encrypted backup during initialization
- **Tab completion**: Supports Bash, Zsh, Fish, PowerShell, Elvish
- **File listing**: View encrypted file information with sorting and original filename display
- **Runtime security checks**: Automatically detects configuration file permission issues
- **Simplified password management**: No separate password hash file, password directly derives key
- **Fast mode**: Optional filename preservation for faster small file processing
- **I/O optimization**: Single write operation reduces system call overhead

## 🚀 Quick Start

### 1. Installation

**Compile from source (recommended):**
```bash
git clone https://github.com/lxp731/leolock.git
cd leolock
cargo build --release
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

### Version 1.0.3 (Current)
- Simplified password management, removed separate password hash file
- Added file listing functionality with sorting and original filename display
- Enhanced configuration file security with automatic permission settings
- Improved shell completion support (5 shells)

### Version 1.0.2
- Filename encryption feature for enhanced privacy protection
- New file format version 2 supporting filename metadata storage
- Backward compatible, supports decryption of older format files

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

**Last Updated:** 2026-03-15  
**Project Status:** ✅ Feature complete, security optimized, stable and usable

**Security Note:** Regularly backup important data, encryption is not insurance against data loss.