# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此仓库中工作时提供指引。

## 构建与测试

```bash
cargo build                  # 构建
cargo test                   # 测试
cargo fmt && cargo clippy    # 格式与 lint
```

## 架构

LeoLock 是一个单一 Rust crate，包含 CLI 入口和库。

**`leolock`** — lib + CLI binary。CLI 入口为 `src/main.rs`（clap derive）。核心模块：
- `crypto.rs` — AES-256-GCM V4 流式加解密（1MB 分块，AAD 保护）。`CryptoManager` 提供 `encrypt_file_v2`、`decrypt_file_v2`、`get_file_info` 和内存直通的 `encrypt_data_v3`/`decrypt_data_v3` 方法。
- `fileops.rs` — `FileOps` 对外暴露两个入口：`encrypt_path_with_config` / `decrypt_path_with_config`。目录遍历使用 `walkdir`。
- `config.rs` — `Config` 包含 2 个 TOML 段：`[program]`、`[core]`。旧的扁平格式在加载时自动迁移。
- `password.rs` — `PasswordManager` 处理 Argon2id 哈希、keyring/env/stdin 密码来源。
- `keymgmt.rs` — 密钥生成、保存/加载（32 字节，600 权限）、备份/恢复（JSON 格式）。
- `errors.rs` — `BjtError` 枚举（ThisError），所有模块使用 `Result<T> = Result<T, BjtError>`。

## 关键模式

**Config 访问**：`Config::load()` 一次性读取 TOML。

**密码处理**：`read_password(cli, prompt)` 根据 CLI 参数自动选择密码来源（env/keyring/stdin/interactive），返回 `Zeroizing<String>`。

**加密流程**：
```
用户输入密码 → Argon2id(password, salt, m, t, p) → [u8;32] AES 密钥
  → encrypt_file_v2: V4 文件头 + 加密文件名 + encrypt_stream(1MB chunk)
  → AAD = 文件头字节（含 Argon2id 参数）
  → .tmp 临时文件 + rename 原子替换
```

## Config 文件分段

```toml
[program]   # CLI 设置：forbidden_paths, max_file_size, preserve_original_filename 等
[core]      # salt（None = 未初始化）、argon2_m_cost/t_cost/p_cost
```

`Config::is_initialized()` 检查 `self.core.salt.is_some()`。

## 文件格式

| 版本 | 魔数 | 特性 |
|------|------|------|
| V1 | 无 | 旧版，一次性加密 |
| V2 | LEO2 | 文件名加密 |
| V3 | LEO3 | 流式 + AAD 保护 |
| V4 | LEO3(ver=4) | V3 + Argon2id 参数存储（12 字节） |

V4 是当前默认格式。加密时参数从 `[core]` 段读取并写入文件头；解密时从文件头恢复参数。V1-V3 自动使用默认参数 (19456/2/1)。

## 应该做的事

- **不要自动提交代码**：完成阶段性任务后不要主动 commit，只有用户明确要求"提交代码"时才执行 git commit 和 git push。
- 如果被要求提交代码，在提交之前请务必检查以下内容：
    1. 确保所有的代码都被格式化过。
    2. 确保所有的代码编译无报错，功能测试全部通过（可以在 /tmp 目录下进行构建测试文件）。
    3. 保证代码无冗余，文档与代码功能相契合，包括并不限于 README.md、CHANGELOG.md。
    4. **文档交叉同步验证**：修改功能后检查 `docs/` 下相关文档是否已同步更新；ROADMAP.md 中的阶段状态是否与实际一致。避免"代码已实现，文档未更新"。
    5. 保证项目中应该被忽略的文件被忽略，比如 `.git`、`.idea`、`.vscode`、`.DS_Store` 等。
    6. 保证项目中的敏感信息被忽略，比如密码、密钥、API Key 等，`.env` 记录在 `.gitignore` 中。如果未被忽略，中断提交，提醒用户对敏感信息进行处理。

- 如果被要求提交代码，提交 commit 信息使用标准格式，比如 `feat: add new feature`，对比实际的完成功能进行描述。

## 不应做的事

- 不要在 `fileops.rs` 中新增公开方法而不删除已弃用的——已经清理过 6 个重复方法（557→237 行）。
- 不要修改配置文件格式而不在 `Config::load_with_path()` 中添加向后兼容的迁移逻辑。

详见 `AGENTS.md` 获取更完整的模块文档、数据流图和扩展点说明。
