# LeoLock 项目 AI Agent 指南

## 构建与开发命令

```bash
# 构建 CLI + lib
cargo build

# 构建 CLI + lib (release)
cargo build --release

# 构建 API 服务
cargo build -p leolock-api

# 构建整个 workspace
cargo build --workspace

# 运行 CLI
cargo run -- <命令>

# 运行 API 服务
cargo run -p leolock-api

# 测试
cargo test --workspace

# 格式化
cargo fmt

# Lint
cargo clippy
```

## 项目概览

**LeoLock** 是一个 Rust 编写的文件加密解密工具，提供 CLI 和 HTTP API 两种使用方式。

- **语言**: Rust (edition 2021)
- **当前版本**: 1.3.0
- **许可证**: MIT
- **仓库**: https://github.com/burgessleo/leolock

### 项目结构

```
leolock/
├── Cargo.toml              # workspace (members: ["", "api"])
├── src/                    # CLI + lib crate
│   ├── main.rs             # CLI 入口 (clap, 7 个子命令)
│   ├── lib.rs              # 库入口 (导出所有模块)
│   ├── crypto.rs           # AES-256-GCM 流式加解密 (V3 格式)
│   ├── fileops.rs          # 文件/目录遍历、进度条
│   ├── keymgmt.rs          # 密钥生成/保存/备份恢复
│   ├── password.rs         # Argon2id 密码管理
│   ├── config.rs           # TOML 配置 + API 鉴权数据
│   ├── errors.rs           # ThisError 统一错误类型
│   └── utils.rs            # SHA256 哈希、安全删除
│
├── api/                    # API 服务子 crate
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # axum 启动 + 路由注册
│       ├── state.rs        # AppState (lock/unlock/密钥/配置缓存)
│       ├── routes/mod.rs   # 所有路由处理函数
│       └── middleware/mod.rs  # JWT 鉴权中间件
│
└── docs/
    ├── API.md              # API 接口文档
    ├── API_PLAN.md         # API 设计规划
    ├── CHANGELOG.md        # 版本历史
    └── ...
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
| 目录遍历 | `walkdir` 2.4 | 递归处理目录 |
| 密码输入 | `rpassword` 7.0 | 终端无回显密码 |
| 进度条 | `indicatif` 0.17 | 批量操作进度 |
| 系统钥匙串 | `keyring` 2.3 | 跨平台密码存储 |

## 架构设计

### API 服务数据流

```
POST /api/v1/unlock
  密码 → Argon2id(salt) → [u8;32] 密钥 → RwLock<Zeroizing> 驻内存

POST /api/v1/encrypt (multipart)
  上传数据 → encrypt_data_v3(内存直通) → 返回 .leo 二进制

POST /api/v1/lock
  RwLock 置 None → Drop 触发 Zeroizing → 内存归零
```

```
认证流程:
  API Key (32 字节随机数, URL-safe base64)
    → Argon2id 哈希存 config.toml (api_key_hash)
    → 登录时对比哈希 → 签发 JWT (30 分钟有效)
    → auth_middleware 验证每个请求的 Authorization header
```

### CLI 数据流（加密）

```
用户输入密码 → Argon2id(password, salt) → [u8;32] AES 密钥
  → encrypt_file_v2: 文件头(LEO3) + 加密文件名 + encrypt_stream(1MB chunk)
  → AAD = 文件头字节（防篡改）
  → .tmp 临时文件 + rename 原子替换
```

## 关键数据结构

### 文件格式版本

| 版本 | 特性 | 魔数 | 内容加密 |
|------|------|------|----------|
| V1 | 旧版 | 无 | 一次性加密 |
| V2 | 文件名加密 | LEO2 | 一次性加密 |
| V3 | **当前默认** | LEO3 | 流式 + AAD 保护 |

### 加密常量

- `KEY_SIZE`: 32 字节 (AES-256)
- `NONCE_SIZE`: 12 字节 (GCM)
- `CHUNK_SIZE`: 1 MB (流式分块)
- `TAG_SIZE`: 16 字节 (GCM 认证标签)

### AppState 结构

```rust
pub struct AppState {
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    pub jwt_secret: Option<String>,
    pub salt: Option<String>,
    pub api_key_hash: Option<String>,
    pub is_initialized: bool,
}
```

### Config 关键字段

```rust
pub struct Config {
    pub forbidden_paths: Vec<String>,
    pub max_file_size: u64,
    pub preserve_original_filename: bool,
    pub show_progress: bool,
    pub salt: Option<String>,           // None = 未初始化
    pub api_key_hash: Option<String>,   // API Key 的 Argon2id 哈希
    pub jwt_secret: Option<String>,     // JWT 签名密钥
}
```

## CLI vs API 功能对照

| 功能 | CLI | API |
|------|-----|-----|
| 初始化 | `leolock init` | `POST /api/v1/init` |
| 加密 | `leolock encrypt <path>` | `POST /api/v1/encrypt` |
| 解密 | `leolock decrypt <path>` | `POST /api/v1/decrypt` |
| 列出文件 | `leolock list <path>` | `GET /api/v1/files` |
| 查看文件 | `leolock list --show-original` | `GET /api/v1/files/get` |
| 下载解密 | — | `GET /api/v1/files/download` |
| 删除文件 | — (手动 rm) | `DELETE /api/v1/files/delete` |
| 密钥备份 | `leolock recover` | — (待实现) |

## 扩展点

- **流式加解密 API**: `POST /api/v1/encrypt-stream` — 分块上传/返回 (ROADMAP v1.4.0)
- **动态 Argon2id 参数**: 从 Config 读取替代硬编码 (ROADMAP v1.5.0)
- **临时分享链接**: `POST /api/v1/share` (ROADMAP v1.6.0)
- **FUSE 挂载**: 虚拟文件系统实时解密 (ROADMAP v2.0)
- **C FFI 绑定**: Python/Node.js SDK (ROADMAP v2.0)
- **硬件密钥**: YubiKey PKCS#11 (ROADMAP v2.0)
