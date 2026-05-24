# LeoLock 🔒

A secure file encryption/decryption tool with CLI and HTTP API, using AES-256-GCM authenticated encryption and Argon2id password hashing.

## ✨ Features

- **Military-grade encryption**: AES-256-GCM authenticated encryption + AAD metadata protection to prevent header tampering.
- **HTTP API service**: REST API for remote encryption/decryption, file management, and JWT authentication.
- **Secure password hashing**: Argon2id resistant to GPU/ASIC attacks.
- **Zero Secrets in Memory**: `zeroize` technology ensures passwords and keys are erased immediately after use. API service auto-locks on restart.
- **Extreme Performance**: Streaming I/O, ~14s for 3GB files; API endpoints use in-memory processing with zero disk I/O.
- **Atomic Operations**: "Write-then-swap" mechanism prevents data corruption from crashes.
- **Dual Encryption Modes**: Filename encryption (Full mode) or content-only (Fast mode).
- **Recursive Processing**: Batch encryption of files and directories.
- **Backup Recovery**: Automatically creates an encrypted key backup during initialization.

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
| `leolock encrypt <path>` | Encrypt file or directory |
| `leolock decrypt <path>` | Decrypt file or directory |
| `leolock list <path>` | List encrypted file information |
| `leolock recover --backup <file>` | Restore key from backup file |
| `leolock completions <shell>` | Generate shell completion scripts |
| `leolock config show` | Show current configuration |
| `leolock config validate` | Validate configuration file |
| `leolock config set <key> <value>` | Modify a configuration item |
| `leolock config gen-api-key` | Generate API Key |
| `leolock config add-forbidden <path>` | Add forbidden path |
| `leolock config remove-forbidden <path>` | Remove forbidden path |

**Common options:**
- `-k, --keep`: Keep source file
- `-F, --fast`: Fast mode (skip filename encryption)
- `--show-original`: Show original filename (requires password)
- `--sort-by-size <asc/desc>`: Sort by file size
- `--format <table/json/simple>`: Output format for list command
- `--env-pass <var>`: Load password from environment variable
- `--keyring`: Load password from system keyring
- `--stdin`: Load password from stdin

### HTTP API

```bash
# Start the server
leolock-api

# Login
curl -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' -d '{"api_key": "..."}'

# Encrypt
curl -X POST http://127.0.0.1:3000/api/v1/encrypt \
  -H "Authorization: Bearer $TOKEN" -F "file=@doc.pdf" -o doc.leo

# Decrypt
curl -X POST http://127.0.0.1:3000/api/v1/decrypt \
  -H "Authorization: Bearer $TOKEN" -F "file=@doc.leo" -o doc.pdf
```

**Complete command reference:** See [docs/COMMANDS.md](docs/COMMANDS.md)
**Complete API reference:** See [docs/API.md](docs/API.md)

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
- **Dangerous path protection**: Default prohibits encryption of 17 system directories
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

### Version 1.5.0 (Current)
- **Dynamic Argon2id Parameters**: Custom m_cost/t_cost/p_cost in `[core]` section, V4 file header stores parameters.
- **Multi-format list output**: `leolock list --format json|simple|table`.
- **Config API**: `GET /api/v1/config` (sensitive fields masked), `PUT /api/v1/config` for runtime updates.
- **CLI config management**: `leolock config set <key> <value>`, `add-forbidden` / `remove-forbidden`.

### Version 1.4.0
- **Stream encryption endpoints**: `encrypt-stream` / `decrypt-stream` for raw binary body.
- **File management API**: List/view/download/delete encrypted files with pagination.
- **API Key rotation**: `POST /api/v1/auth/rotate-api-key` with password verification.
- **Unlock rate limiting**: Max 5 attempts per IP per minute (HTTP 429).
- **Request logging middleware**: Method/path/status/duration, no sensitive data.
- **Error response sanitization**: Internal errors return generic messages, details to stderr.

### Version 1.3.0
- **HTTP API service**: New `leolock-api` sub-crate providing REST API.
- **Lock/Unlock security mode**: Key only in memory, auto-lock on restart.
- **JWT authentication**: API Key (Argon2id hash) → short-lived JWT (30 min).
- **In-memory passthrough**: API encryption/decryption with zero temp files.

### Version 1.2.0
- **Multithreading**: `rayon` parallel processing for directory recursion.
- **Enhanced UX**: Progress bars (`indicatif`) + real-time password strength evaluation.
- **Advanced password policies**: Environment variable / keyring / stdin support.

### Version 1.1.0
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

**Last Updated:** 2026-05-25  
**Project Status:** ✅ CLI v1.5.0 + API Service, Stable

**Security Note:** Regularly backup important data, encryption is not insurance against data loss.