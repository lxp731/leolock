# LeoLock 项目 AI Agent 指南

## 构建与开发命令

```bash
cargo build                    # CLI + lib
cargo build -p leolock-api     # API 服务
cargo build --workspace        # 全部
cargo run -- <命令>             # 运行 CLI
cargo run -p leolock-api       # 运行 API 服务
cargo test --workspace
cargo fmt && cargo clippy
```

## 项目概览

**LeoLock** 是一个 Rust 编写的文件加密解密工具，提供 CLI 和 HTTP API 两种使用方式。

- **语言**: Rust (edition 2021)
- **当前版本**: 1.2.0
- **许可证**: MIT
- **仓库**: https://github.com/lxp731/leolock

### 项目结构

```
leolock/
├── Cargo.toml              # workspace (members: ["", "api"])
├── src/                    # CLI + lib crate
│   ├── main.rs             # CLI 入口 (clap, 10+ 子命令)
│   ├── lib.rs              # 库入口 (导出所有模块)
│   ├── crypto.rs           # AES-256-GCM V4 流式加解密
│   ├── fileops.rs          # 文件/目录遍历、进度条
│   ├── keymgmt.rs          # 密钥生成/保存/备份恢复
│   ├── password.rs         # Argon2id 密码管理
│   ├── config.rs           # TOML 配置（[program]/[core]/[auth]/[api]）
│   ├── errors.rs           # ThisError 统一错误类型
│   └── utils.rs            # SHA256 哈希、安全删除
│
├── api/                    # API 服务子 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # axum 启动 + 路由注册（24 端点）
│       ├── state.rs        # AppState (lock/unlock/密钥/配置缓存)
│       ├── routes/mod.rs   # 所有路由处理函数
│       └── middleware/mod.rs  # JWT 鉴权 + 请求日志
│
├── docs/
│   ├── API.md              # API 接口文档（24 端点，6 分类）
│   ├── CHANGELOG.md        # 版本历史
│   ├── INSTALLATION.md     # 安装指南
│   └── ...
│
└── install.sh              # 一键安装脚本
```

### 核心技术栈

| 类别 | 依赖 | 用途 |
|------|------|------|
| CLI 框架 | `clap` 4.0 (derive) | 命令解析、自动补全 |
| 加密算法 | `aes-gcm` 0.10 | AES-256-GCM 认证加密 |
| 密码哈希 | `argon2` 0.5 | Argon2id 密钥派生 |
| 内存安全 | `zeroize` 1.7 | 敏感数据即时擦除 |
| HTTP 框架 | `axum` 0.7 | API 路由、multipart、状态管理 |
| 鉴权 | `jsonwebtoken` 9 | JWT 签发与验证 |
| 序列化 | `serde` + `toml` + `serde_json` | 配置和备份读写 |
| 流式 | `tokio-stream` `http-body-util` | 流式响应 |
| 目录遍历 | `walkdir` 2.4 | 递归处理目录 |
| 密码输入 | `rpassword` 7.0 | 终端无回显密码 |
| 进度条 | `indicatif` 0.17 | 批量操作进度 |
| 系统钥匙串 | `keyring` 2.3 | 跨平台密码存储 |

## 架构设计

### API 服务数据流

```
POST /api/v1/unlock
  密码 → Argon2id(salt, m, t, p) → [u8;32] 密钥 → RwLock<Zeroizing> 驻内存
  速率限制：每 IP 5次/分钟

POST /api/v1/encrypt (multipart)
  上传 → encrypt_data_v3(内存直通) → 返回 .leo 二进制
  stream 端点：原始二进制 body + X-Filename 头

POST /api/v1/lock
  RwLock 置 None → Drop 触发 Zeroizing → 内存归零

认证流程:
  API Key → Argon2id 哈希存 config.toml [auth] 段
    → login 时对比 → 签发 JWT (30 分钟)
    → auth_middleware 验证 Authorization header
    → rotate-api-key 支持运行时轮换
```

### CLI 数据流（加密）

```
用户输入密码 → Argon2id(password, salt, m, t, p) → [u8;32] AES 密钥
  → encrypt_file_v2: V4 文件头 + 加密文件名 + encrypt_stream(1MB chunk)
  → AAD = 文件头字节（含 Argon2id 参数）
  → .tmp 临时文件 + rename 原子替换
```

## 关键数据结构

### 文件格式版本

| 版本 | 魔数 | 特性 |
|------|------|------|
| V1 | 无 | 旧版，一次性加密 |
| V2 | LEO2 | 文件名加密 |
| V3 | LEO3 | 流式 + AAD 保护 |
| V4 | LEO3(ver=4) | V3 + Argon2id 参数存储（当前默认） |

### Config 结构（嵌套）

```rust
pub struct Config {
    pub program: ProgramConfig,  // forbidden_paths, max_file_size, ...
    pub core: CoreConfig,        // salt, argon2_m_cost/t_cost/p_cost
    pub auth: AuthConfig,        // api_key_hash, jwt_secret
    pub api: ApiConfig,          // bind_address, port
}
```

### AppState 关键字段

```rust
pub struct AppState {
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    pub jwt_secret: Option<String>,
    salt: RwLock<Option<String>>,       // 支持运行时更新
    api_key_hash: RwLock<Option<String>>, // 支持运行时轮换
    pub argon2_m/t/p: u32,
    pub start_time: Instant,            // 用于 uptime 指标
    pub request_count: Mutex<HashMap<String, u64>>,  // 请求计数
    pub shares: Mutex<HashMap<String, ShareInfo>>,    // 分享链接存储
    unlock_attempts: Mutex<HashMap<IpAddr, ...>>,     // 速率限制
}
```

## CLI vs API 功能对照

| 功能 | CLI | API |
|------|-----|-----|
| 初始化 | `leolock init` | `POST /api/v1/init` |
| 加密 | `leolock encrypt <path>` | `POST /api/v1/encrypt` |
| 流式加密 | — | `POST /api/v1/encrypt-stream` |
| 解密 | `leolock decrypt <path>` | `POST /api/v1/decrypt` |
| 流式解密 | — | `POST /api/v1/decrypt-stream` |
| 列出文件 | `leolock list <path>` | `GET /api/v1/files` |
| 查看/下载/删除 | — | `GET /DELETE /api/v1/files/*` |
| 配置管理 | `config set/show/validate` | `GET/PUT /api/v1/config` |
| 分享链接 | — | `POST /api/v1/share` |
| 密钥备份 | `leolock recover` | `POST /api/v1/backup` `/recover` |
| 密钥轮换 | — | `POST /api/v1/auth/rotate-key` |
| API Key 轮换 | `config gen-api-key` | `POST /api/v1/auth/rotate-api-key` |
| 统计/指标 | — | `GET /api/v1/stats` `/metrics` |

## 扩展点（待实现）

- **Web 管理面板** — 纯静态前端拖拽上传加密、文件管理 (ROADMAP v2.0)
- **FUSE 挂载** — 虚拟文件系统实时解密 (ROADMAP v2.0)
- **C FFI 绑定** — Python/Node.js SDK (ROADMAP v2.0)
- **硬件密钥** — YubiKey PKCS#11 (ROADMAP v2.0)
- **云端密钥库** — HashiCorp Vault / AWS KMS (ROADMAP v2.0)
