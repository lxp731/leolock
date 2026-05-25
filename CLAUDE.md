# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此仓库中工作时提供指引。

## 构建与测试

```bash
cargo build                    # CLI + lib
cargo build -p leolock-api  # API 服务
cargo build --workspace        # 全部
cargo test --workspace
cargo fmt && cargo clippy
```

## 架构

LeoLock 是一个 Rust workspace，包含两个 crate：

**`leolock`（根目录）** — lib crate，通过 `src/lib.rs` 导出所有模块。CLI 入口为 `src/main.rs`（clap derive）。核心模块：
- `crypto.rs` — AES-256-GCM V4 流式加解密（1MB 分块，AAD 保护）。`CryptoManager` 同时提供基于文件的（`encrypt_file_v2`）和内存直通的（`encrypt_data_v3`）方法。
- `fileops.rs` — `FileOps` 对外暴露两个入口：`encrypt_path_with_config` / `decrypt_path_with_config`。目录遍历使用 `walkdir`。
- `config.rs` — `Config` 包含 4 个 TOML 段：`[program]`、`[core]`、`[auth]`、`[api]`。旧的扁平格式在加载时自动迁移。
- `password.rs` — `PasswordManager` 处理 Argon2id 哈希、keyring/env/stdin 密码来源、API Key 哈希。
- `keymgmt.rs` — 密钥生成、保存/加载（32 字节，600 权限）、备份/恢复（JSON 格式）。
- `errors.rs` — `BjtError` 枚举（ThisError），所有模块使用 `Result<T> = Result<T, BjtError>`。

**`leolock-api`（api/）** — axum 0.7 HTTP API。依赖 `leolock` lib。
- `state.rs` — `AppState` 持有加密密钥（RwLock）、JWT 密钥、盐值、API Key 哈希（RwLock，支持运行时轮换）。
- `routes/mod.rs` — 所有路由处理函数。公开：health、status、login、init。受保护（JWT）：unlock、lock、encrypt、decrypt、files/*、auth/rotate-api-key。
- `middleware/mod.rs` — JWT 验证中间件（30 分钟过期）。

## 关键模式

**Config 访问**：`Config::load()` 一次性读取 TOML。服务启动时将字段缓存到 `AppState`——路由处理函数**不得**再次调用 `Config::load()`。应通过 `state.salt`、`state.is_initialized` 等访问。

**错误处理**：路由返回 `Result<T, AppError>`。`AppError` 映射到 HTTP 状态码：`Locked` → 423，`NotInitialized` → 412，`BadRequest` → 400，`CryptoError` → 400，`RateLimited` → 429，`Internal` → 500。`BjtError` 和 `std::io::Error` 通过 `From` 自动转换。

**API 加解密**：使用 `CryptoManager::encrypt_data_v3(data, filename, key)` 和 `decrypt_data_v3(data, key)` 进行内存操作——禁止写临时文件。

**密码处理**：请求结构体使用 `password: String`（兼容 serde），通过 `into_password() -> Zeroizing<String>` 在使用后立即擦除。

**文件 ID**：文件绝对路径的 URL-safe base64 编码（无填充）。`encode_id` / `decode_id` 辅助函数在 routes/mod.rs 中。

## Config 文件分段

```toml
[program]   # CLI 设置：forbidden_paths, max_file_size, preserve_original_filename 等
[core]      # salt（None = 未初始化）、argon2_m_cost/t_cost/p_cost
[auth]      # api_key_hash（Argon2id）、jwt_secret
[api]       # bind_address, port
```

`Config::is_initialized()` 检查 `self.core.salt.is_some()`。

## 服务 Lock/Unlock 机制

服务启动时处于 LOCKED 状态。`POST /unlock` 通过 Argon2id 从密码 + 盐值派生 AES 密钥，存入 `RwLock<Option<Zeroizing<[u8;32]>>>`。`POST /lock` 置为 `None`，drop 触发 zeroize。encrypt/decrypt/download/delete 在锁定时返回 423。list/get 不受影响。

## 文件格式

| 版本 | 魔数 | 特性 |
|------|------|------|
| V1 | 无 | 旧版，一次性加密 |
| V2 | LEO2 | 文件名加密 |
| V3 | LEO3 | 流式 + AAD 保护 |
| V4 | LEO3(ver=4) | V3 + Argon2id 参数存储（12 字节） |

V4 是当前默认格式。加密时参数从 `[core]` 段读取并写入文件头；解密时从文件头恢复参数。V1-V3 自动使用默认参数 (19456/2/1)。

## 应该做的事

- 如果被要求提交代码，在提交之前请务必检查以下内容：
    1. 确保所有的代码都被格式化过。
    2. 确保所有的代码编译无报错，功能测试全部通过（可以在 /tmp 目录下进行构建测试文件）。
    3. 保证代码无冗余，文档与代码功能相契合，包括并不限于 README.md、CHANGELOG.md。
    4. **文档交叉同步验证**：新增或修改 API 端点后，检查 `docs/API.md` 是否已同步更新对应的调用示例和参数说明；ROADMAP.md 中的阶段状态是否与实际一致。避免"代码已实现，文档未更新"。
    5. 保证项目中应该被忽略的文件被忽略，比如 `.git`、`.idea`、`.vscode`、`.DS_Store` 等。
    6. 保证项目中的敏感信息被忽略，比如密码、密钥、API Key 等，`.env` 记录在 `.gitignore` 中。如果未被忽略，中断提交，提醒用户对敏感信息进行处理。

- 如果被要求提交代码，提交 commit 信息使用标准格式，比如 `feat: add new feature`，对比实际的完成功能进行描述。

## 不应做的事

- 不要在 `fileops.rs` 中新增公开方法而不删除已弃用的——已经清理过 6 个重复方法（557→237 行）。
- 不要在路由处理函数中调用 `Config::load()`——使用缓存的 `AppState` 字段。
- 不要让 API 加解密走临时文件——使用内存直通的 `_data_v3` 方法。
- 不要修改配置文件格式而不在 `Config::load_with_path()` 中添加向后兼容的迁移逻辑。

详见 `AGENTS.md` 获取更完整的模块文档、数据流图和扩展点说明。
