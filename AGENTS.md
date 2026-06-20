# LeoLock 项目 AI Agent 指南

## 构建与开发命令

```bash
cargo build                    # 构建
cargo test                     # 测试
cargo fmt && cargo clippy      # 格式与 lint
```

## 项目概览

**LeoLock** 是一个 Rust 编写的个人文件加密解密 CLI 工具。

- **语言**: Rust (edition 2021)
- **当前版本**: 1.2.0
- **许可证**: MIT
- **仓库**: https://github.com/lxp731/leolock

### 项目结构

```
leolock/
├── Cargo.toml              # 单一 crate
├── src/                    # CLI + lib
│   ├── main.rs             # CLI 入口 (clap, 9 个子命令)
│   ├── lib.rs              # 库入口 (导出所有模块)
│   ├── crypto.rs           # AES-256-GCM V4 流式加解密
│   ├── fileops.rs          # 文件/目录遍历、进度条
│   ├── keymgmt.rs          # 密钥生成/保存/备份恢复
│   ├── password.rs         # Argon2id 密码管理
│   ├── config.rs           # TOML 配置（[program]/[core]）
│   ├── errors.rs           # ThisError 统一错误类型
│   └── utils.rs            # SHA256 哈希、安全删除
│
├── docs/
│   ├── CHANGELOG.md        # 版本历史
│   ├── INSTALLATION.md     # 安装指南
│   ├── COMMANDS.md         # 完整命令参考
│   ├── CONFIGURATION.md    # 配置文件说明
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
| 序列化 | `serde` + `toml` + `serde_json` | 配置和备份读写 |
| 目录遍历 | `walkdir` 2.4 | 递归处理目录 |
| 密码输入 | `rpassword` 7.0 | 终端无回显密码 |
| 进度条 | `indicatif` 0.17 | 批量操作进度 |
| 系统钥匙串 | `keyring` 2.3 | 跨平台密码存储 |

## 架构设计

### 数据流（加密）

```
用户输入密码 → Argon2id(password, salt, m, t, p) → [u8;32] AES 密钥
  → encrypt_file_v2: V4 文件头 + 加密文件名 + encrypt_stream(1MB chunk)
  → AAD = 文件头字节（含 Argon2id 参数）
  → .tmp 临时文件 + rename 原子替换
```

### 密码来源

`read_password(cli, prompt)` 根据 CLI 参数自动选择：
1. `--env-pass VAR` → 从环境变量读取
2. `--keyring` → 从系统钥匙串读取
3. `--stdin` → 从标准输入读取（管道友好）
4. 默认 → 交互式无回显输入

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
}
```

### CoreConfig 关键字段

```rust
pub struct CoreConfig {
    pub salt: Option<String>,     // None = 未初始化，Base64 编码的 16 字节随机盐值
    pub argon2_m_cost: u32,       // 内存成本 (KB)，默认 19456（约 19MB）
    pub argon2_t_cost: u32,       // 迭代次数，默认 2
    pub argon2_p_cost: u32,       // 并行度，默认 1
}
```

## 命令总览

| 命令 | 说明 |
|------|------|
| `leolock init` | 初始化工具 |
| `leolock encrypt <path>` | 加密文件或目录 |
| `leolock decrypt <path>` | 解密文件或目录 |
| `leolock list <path>` | 列出加密文件信息 |
| `leolock recover --backup <file>` | 从备份文件恢复密钥 |
| `leolock completions <shell>` | 生成 shell 补全脚本 |
| `leolock config show` | 查看当前配置 |
| `leolock config validate` | 验证配置文件 |
| `leolock config set <key> <value>` | 修改配置项 |
| `leolock config add-forbidden <path>` | 添加禁止加密路径 |
| `leolock config remove-forbidden <path>` | 移除禁止加密路径 |

## 扩展点（待实现）

- **密码修改**: `leolock change-password` — 重新派生密钥，批量重加密已有文件 (ROADMAP v2.0)
- **密钥轮换**: `leolock rotate-key` — 重新生成主密钥，可选批量重加密 (ROADMAP v2.0)
- **增量加密**: 只加密变更的文件，大幅提升目录重加密速度 (ROADMAP v2.0)
- **加密文件完整性校验**: 不依赖密码的 `.leo` 文件结构校验 (ROADMAP v2.0)
- **Shell 补全自动安装**: `leolock completions install <shell>` (ROADMAP v2.0)
