
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->


# LeoLock 🔒

一个安全的文件加密解密命令行工具，使用 AES-256-GCM 加密算法和 Argon2id 密码哈希。

## ✨ 特性

- **军用级加密**: AES-256-GCM 认证加密
- **安全密码哈希**: Argon2id 抗 GPU/ASIC 攻击
- **交互式操作**: 安全的密码输入（无回显）
- **递归处理**: 支持文件和文件夹的批量加密
- **智能错误处理**: 单个文件失败不影响其他文件
- **安全删除**: 默认删除源文件，可选保留
- **备份恢复**: 初始化时自动创建加密备份
- **Tab 补全**: 支持 Bash、Zsh、Fish

## 📦 安装

### 从源码编译

```bash
# 克隆项目
git clone <项目地址>
cd leolock

# 编译发布版本
cargo build --release

# 安装到系统（可选）
sudo cp target/release/leolock /usr/local/bin/
```

### 生成 Tab 补全

```bash
# Bash
./target/release/leolock --completions bash > ~/.bash_completion.d/leolock

# Zsh
./target/release/leolock --completions zsh > ~/.zsh/completions/_leolock

# Fish
./target/release/leolock --completions fish > ~/.config/fish/completions/leolock.fish
```

## 🚀 快速开始

### 1. 初始化工具

```bash
leolock init
```

初始化过程会：
- 创建配置目录 `~/.config/leolock/`
- 生成 AES-256 密钥文件
- 设置初始密码
- **自动创建加密备份文件**（请妥善保管！）

### 2. 加密文件

```bash
# 加密单个文件（交互式输入密码）
leolock encrypt secret.txt
# 输出: secret.txt -> secret.txt.leo

# 加密文件夹（递归处理所有文件）
leolock encrypt documents/

# 加密并保留源文件
leolock encrypt important.txt --keep-original
```

### 3. 解密文件

```bash
# 解密单个文件
leolock decrypt secret.txt.leo
# 输出: secret.txt.leo -> secret.txt

# 解密文件夹（自动跳过非加密文件）
leolock decrypt mixed_folder/

# 解密并保留加密文件
leolock decrypt encrypted.leo --keep-original
```

## 📖 完整命令参考

### 初始化与恢复

| 命令 | 说明 |
|------|------|
| `leolock init` | 初始化工具（创建配置和密钥） |
| `leolock recover --backup <文件>` | 从备份文件恢复密钥 |

### 密码管理

| 命令 | 说明 |
|------|------|
| `leolock password update` | 修改操作密码 |

### 密钥管理

| 命令 | 说明 |
|------|------|
| `leolock key update` | 重新生成密钥（危险操作！） |

### 加密解密

| 命令 | 说明 |
|------|------|
| `leolock encrypt <路径>` | 加密文件或文件夹 |
| `leolock decrypt <路径>` | 解密文件或文件夹 |
| `--keep-original` | 保留源文件（不删除） |

### 帮助系统

| 命令 | 说明 |
|------|------|
| `leolock --help` | 显示帮助信息 |
| `leolock <命令> --help` | 显示子命令帮助 |
| `leolock --completions <shell>` | 生成 Tab 补全脚本 |

## 🔐 安全特性

### 密码学算法
- **文件加密**: AES-256-GCM（认证加密）
- **密码存储**: Argon2id（随机盐值，抗 GPU 攻击）
- **密钥派生**: Argon2id（用于备份文件加密）

### 安全限制
- **密码强度**: 至少 8 位，包含数字和字母
- **尝试限制**: 密码验证最多 3 次
- **危险路径跳过**: 自动跳过 `/bin`、`/usr`、`/lib` 等系统目录
- **符号链接安全**: 加密源文件，检测循环链接

### 文件处理
- **扩展名保留**: `a.txt` → `a.txt.leo` → `a.txt`
- **递归处理**: 支持文件夹的递归加密/解密
- **宽松错误处理**: 单个文件失败不影响其他文件
- **安全删除**: 默认覆盖数据后删除源文件

## ⚠️ 重要警告

### 1. 备份责任
- **初始化时自动创建备份**，请立即将备份文件复制到安全位置
- 备份文件使用您的密码加密，请牢记密码
- 如果忘记密码或丢失备份，**所有加密数据将永久丢失**

### 2. 密钥更新风险
```bash
leolock key update  # ⚠️ 危险操作！
```
重新生成密钥将导致：
- 旧密钥加密的所有文件**无法解密**
- 旧的备份文件**失效**
- 必须立即备份新密钥

### 3. 密码安全
- 使用强密码（至少 8 位，包含数字和字母）
- 定期修改密码
- 不要与他人共享密码

## 📁 文件结构

### 用户文件
```
~/.config/leolock/
├── leolock.conf      # 配置文件（TOML 格式）
└── leolock.key       # AES-256 密钥文件

~/leolock_key_backup_YYYYMMDD_HHMMSS.enc  # 加密备份文件
```

### 配置文件示例
```toml
suffix = ".leo"
password_hash = "$argon2id$v=19$m=19456,t=2,p=1$...$..."
salt = "base64编码的随机盐值"
```

## 🧪 测试

项目包含完整的测试套件：

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_encryption
cargo test test_password
```

测试目录：`/home/knight/test/`

## 🔧 开发

### 项目结构
```
leolock/
├── Cargo.toml                    # 项目配置
├── src/                          # 源代码
│   ├── main.rs                   # CLI 入口
│   ├── config.rs                 # 配置管理
│   ├── crypto.rs                 # AES 加密
│   ├── keymgmt.rs                # 密钥管理
│   ├── fileops.rs                # 文件操作
│   ├── password.rs               # 密码处理
│   ├── errors.rs                 # 错误处理
│   └── utils.rs                  # 工具函数
└── README.md                     # 本文档
```

### 构建
```bash
# 开发构建
cargo build

# 发布构建（优化）
cargo build --release

# 检查代码
cargo check
cargo clippy
```

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请：
1. 查看本文档
2. 运行 `leolock --help`
3. 提交 Issue

---

**安全提示**: 请定期备份重要数据，加密不是数据丢失的保险措施。