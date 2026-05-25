# LeoLock

A secure file encryption tool with CLI and HTTP API interfaces, using AES-256-GCM and Argon2id.

## Features

- **AES-256-GCM**: Authenticated encryption with AAD metadata protection against header tampering.
- **HTTP API**: REST API for remote encrypt/decrypt, file management, and JWT authentication.
- **Argon2id**: GPU/ASIC-resistant password hashing.
- **Zero Memory Residue**: `zeroize` ensures passwords and keys are wiped immediately after use. API service auto-locks on restart.
- **High Performance**: Streaming I/O, ~14s for 3GB files. API uses in-memory processing with no temp files.
- **Atomic Operations**: Write-to-tmp-then-rename prevents data corruption on crash.
- **Dual Mode**: Full mode (encrypts filename + content) or Fast mode (content only).
- **Recursive Processing**: Batch encrypt/decrypt files and directories.
- **Backup & Recovery**: Auto-generates encrypted key backup on initialization.

## Quick Start

### 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/lxp731/leolock/main/install.sh | bash
```

Auto-detects OS and architecture, downloads the latest release from GitHub. Installs both `leolock` (CLI) and `leolock-api` (HTTP service).

**Other methods:** See [docs/INSTALLATION.md](docs/INSTALLATION.md)

### 2. Initialize

```bash
leolock init
```

Sets your password and generates encryption keys.

### 3. Encrypt

```bash
leolock encrypt secret.txt          # Full mode: encrypts filename + content
leolock encrypt secret.txt --fast   # Fast mode: content only, filename visible
leolock encrypt secret.txt --keep   # Keep original file
```

### 4. Decrypt

```bash
leolock decrypt secret.txt.leo
```

### 5. List Files

```bash
leolock list .
leolock list . --sort-by-size desc
leolock list . --show-original
```

## Commands

### CLI

| Command | Description |
|------|------|
| `leolock init` | Initialize tool |
| `leolock encrypt <path>` | Encrypt file or directory |
| `leolock decrypt <path>` | Decrypt file or directory |
| `leolock list <path>` | List encrypted files |
| `leolock recover --backup <file>` | Recover key from backup |
| `leolock completions <shell>` | Generate shell completions |
| `leolock config show` | Show current config |
| `leolock config validate` | Validate config file |
| `leolock config set <key> <value>` | Modify config item |
| `leolock config add-forbidden <path>` | Add forbidden path |
| `leolock config remove-forbidden <path>` | Remove forbidden path |
| `leolock config gen-api-key` | Generate API Key |

**Common options:** `-k, --keep` (keep original), `-F, --fast` (fast mode), `--show-original`, `--sort-by-size`

### HTTP API

```bash
leolock-api    # Start API server (default: 127.0.0.1:3000)

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

Full API reference: [docs/API.md](docs/API.md)

## Security

- **AES-256-GCM**: Military-grade authenticated encryption
- **Argon2id**: GPU/ASIC-resistant password hashing
- **Random Salt**: Unique per instance, prevents rainbow table attacks
- **File Permissions**: Config files automatically set to 600
- **Forbidden Paths**: 17 system directories blocked by default
- **File Size Limit**: 10GB default, prevents accidental encryption of large files
- **Password Strength**: Minimum 8 characters with digits and letters

See [docs/SECURITY.md](docs/SECURITY.md) for details.

## Documentation

- [docs/API.md](docs/API.md) — HTTP API reference
- [docs/INSTALLATION.md](docs/INSTALLATION.md) — Installation guide
- [docs/COMMANDS.md](docs/COMMANDS.md) — Full CLI reference
- [docs/SECURITY.md](docs/SECURITY.md) — Security details
- [docs/CONFIGURATION.md](docs/CONFIGURATION.md) — Config file reference
- [docs/WARNINGS.md](docs/WARNINGS.md) — Important warnings
- [docs/CHANGELOG.md](docs/CHANGELOG.md) — Version history

## License

MIT License — see [LICENSE](LICENSE)
