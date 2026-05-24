## 📁 文件结构

### 用户文件
```
~/.config/leolock/
├── config.toml      # 配置文件（危险路径、文件大小、盐值等，权限600）
└── keys.toml        # 密钥文件（AES-256 密钥，权限600）

~/leolock_key_backup_YYYYMMDD_HHMMSS.enc  # 加密备份文件
```

**文件权限说明**:
- `config.toml`: `600`（仅所有者可读写），包含盐值等敏感信息
- `keys.toml`: `600`（仅所有者可读写），包含主密钥
- 目录权限: `700`（仅所有者可访问）
- 运行时自动检查权限安全性

### 项目结构
```
leolock/
├── Cargo.toml                    # 工作空间配置（含 server 子 crate）
├── ROADMAP.md                    # 项目路线图和技术规划
├── src/                          # 核心库 + CLI
│   ├── main.rs                   # CLI入口和命令解析
│   ├── lib.rs                    # 库模式接口
│   ├── config.rs                 # 统一配置管理（[program]/[core]/[auth]/[api]）
│   ├── crypto.rs                 # AES-256-GCM 加解密（支持文件名加密、V3 流式）
│   ├── keymgmt.rs                # 密钥管理（生成、备份、恢复）
│   ├── fileops.rs                # 文件操作（递归、危险路径检查）
│   ├── password.rs               # 密码处理（Argon2id、交互式、keyring）
│   ├── errors.rs                 # 错误类型定义
│   └── utils.rs                  # 工具函数
├── api/                          # HTTP API 服务（axum）
│   └── src/
│       ├── main.rs               # 服务入口
│       ├── state.rs              # 共享状态（AppState）
│       ├── routes/mod.rs         # 所有 API 端点
│       └── middleware/mod.rs     # JWT 验证、日志、速率限制
└── docs/                         # 项目文档
    ├── API.md                    # HTTP API 完整参考
    ├── COMMANDS.md               # CLI 命令参考
    ├── CONFIGURATION.md          # 配置文件说明
    ├── SECURITY.md               # 安全特性文档
    ├── INSTALLATION.md           # 安装指南
    ├── STRUCTURE.md              # 本文档
    ├── CHANGELOG.md              # 版本更新历史
    ├── WARNINGS.md               # 重要警告
    └── README.md                 # 文档索引
```

