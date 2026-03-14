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

### 配置文件示例
完整配置示例见 `examples/config.toml`。

### 项目结构
```
leolock/
├── Cargo.toml                    # 项目配置
├── CREATE.md                     # 需求规格文档（已合并）
├── ROADMAP.md                    # 项目路线图和技术规划
├── examples/                     # 示例文件
│   └── config.toml               # 示例配置文件
├── src/                          # 源代码
│   ├── main.rs                   # CLI入口和命令解析
│   ├── config.rs                 # 统一配置管理（危险路径、文件大小等）
│   ├── crypto.rs                 # AES-256-GCM加密/解密（支持文件名加密）
│   ├── keymgmt.rs                # 密钥管理（生成、备份、恢复）
│   ├── fileops.rs                # 文件操作（递归、危险路径检查）
│   ├── password.rs               # 密码处理（Argon2id、交互式）
│   ├── errors.rs                 # 错误类型定义
│   ├── utils.rs                  # 工具函数（确认、盐值生成、安全删除、文件名哈希）
│   └── lib.rs                    # 库模式接口
└── README.md                     # 本文档
```

