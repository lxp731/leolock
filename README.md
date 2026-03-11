
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [LeoLock 🔒](#leolock-)
  - [✨ 特性](#-特性)
  - [📦 安装](#-安装)
    - [Debian/Ubuntu/RHEL/Fedora](#debianubunturhelfedora)
    - [AUR](#aur)
    - [从源码编译](#从源码编译)
    - [生成 Tab 补全](#生成-tab-补全)
  - [🚀 快速开始](#-快速开始)
    - [1. 初始化工具](#1-初始化工具)
    - [2. 加密文件](#2-加密文件)
    - [3. 解密文件](#3-解密文件)
  - [📖 完整命令参考](#-完整命令参考)
    - [初始化与恢复](#初始化与恢复)
    - [密码管理](#密码管理)
    - [密钥管理](#密钥管理)
    - [配置管理](#配置管理)
    - [加密解密](#加密解密)
    - [Shell 补全](#shell-补全)
    - [帮助系统](#帮助系统)
  - [🔐 安全特性](#-安全特性)
    - [密码学算法](#密码学算法)
    - [安全限制](#安全限制)
    - [文件处理](#文件处理)
    - [配置管理](#配置管理-1)
      - [默认配置](#默认配置)
      - [配置特性](#配置特性)
  - [⚠️ 重要警告](#️-重要警告)
    - [1. 备份责任](#1-备份责任)
    - [2. 密钥更新风险](#2-密钥更新风险)
    - [3. 密码安全](#3-密码安全)
    - [4. 初始化要求](#4-初始化要求)
  - [📁 文件结构](#-文件结构)
    - [用户文件](#用户文件)
    - [配置文件示例](#配置文件示例)
    - [项目结构](#项目结构)
  - [📄 许可证](#-许可证)
  - [🤝 贡献](#-贡献)
  - [📞 支持](#-支持)
  - [📝 版本历史](#-版本历史)

<!-- /code_chunk_output -->



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
- **统一配置**: 危险路径、文件大小等安全设置可配置
- **环境变量覆盖**: 运行时可通过环境变量覆盖配置
- **23个危险路径** - 保护系统关键目录
- **10GB文件大小限制** - 防止意外加密大文件
- **可配置性** - 所有安全设置都可自定义
- **版本自动同步** - 只需修改 `Cargo.toml`，`main.rs` 自动读取

## 📦 安装

### Debian/Ubuntu/RHEL/Fedora

```bash
# 在仓库下载 .deb 包或者 .rpm 包
```

### AUR

```bash
yay -S leolock
```

### 从源码编译

```bash
# 克隆项目
git clone https://github.com/lxp731/leolock.git
cd leolock

# 编译发布版本
cargo build --release

# 安装到系统（可选）
sudo cp target/release/leolock /usr/local/bin/
```

### 生成 Tab 补全

```bash
# Bash
leolock complete bash > /usr/share/bash-completion/completions/leolock

# Zsh
leolock complete zsh > /usr/share/zsh/site-functions/_leolock

# Fish
leolock complete fish > /usr/share/fish/vendor_completions.d/leolock.fish

# 使用 AUR 默认会安装了补全脚本
# Bash
/usr/share/bash-completion/completions/leolock

# Zsh
/usr/share/zsh/site-functions/_leolock

# Fish
/usr/share/fish/vendor_completions.d/leolock.fish
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
- 创建默认配置文件（包含23个危险路径）
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

### 配置管理

| 命令 | 说明 |
|------|------|
| `leolock config show` | 显示当前配置 |
| `leolock config validate` | 验证配置文件 |

### 加密解密

| 命令 | 说明 |
|------|------|
| `leolock encrypt <路径>` | 加密文件或文件夹 |
| `leolock decrypt <路径>` | 解密文件或文件夹 |
| `--keep-original` | 保留源文件（不删除） |

### Shell 补全

| 命令 | 说明 |
|------|------|
| `leolock complete bash` | 生成 Bash 补全脚本 |
| `leolock complete zsh` | 生成 Zsh 补全脚本 |
| `leolock complete fish` | 生成 Fish 补全脚本 |
| `leolock complete powershell` | 生成 PowerShell 补全脚本 |
| `leolock complete elvish` | 生成 Elvish 补全脚本 |

### 帮助系统

| 命令 | 说明 |
|------|------|
| `leolock --help` | 显示帮助信息 |
| `leolock <命令> --help` | 显示子命令帮助 |

## 🔐 安全特性

### 密码学算法
- **文件加密**: AES-256-GCM（认证加密）
- **密码存储**: Argon2id（随机盐值，抗 GPU 攻击）
- **密钥派生**: Argon2id（用于备份文件加密）

### 安全限制
- **密码强度**: 至少 8 位，包含数字和字母
- **尝试限制**: 密码验证最多 3 次
- **危险路径保护**: 默认包含23个系统目录，禁止加密
- **最大文件大小**: 默认10GB限制，防止意外加密大文件
- **初始化要求**: 必须先初始化才能执行加密操作

### 文件处理
- **扩展名保留**: `a.txt` → `a.txt.leo` → `a.txt`
- **递归处理**: 支持文件夹的递归加密/解密
- **宽松错误处理**: 单个文件失败不影响其他文件
- **安全删除**: 默认覆盖数据后删除源文件
- **符号链接安全**: 加密源文件，检测循环链接

### 配置管理
#### 默认配置
```toml
# ~/.config/leolock/config.toml
# 危险路径列表（禁止处理的系统目录）
forbidden_paths = [
    "/bin", "/sbin", "/usr/bin", "/usr/sbin",
    "/lib", "/lib64", "/usr/lib", "/usr/lib64",
    "/boot", "/dev", "/proc", "/sys", "/run",
    "/etc", "/root", "/var", "/tmp",
    "/usr/local/bin", "/usr/local/sbin",
    "/opt", "/home", "/mnt", "/media",
]

# 最大文件大小（字节），0表示无限制
max_file_size = 10737418240  # 10GB

# 是否启用进度显示
show_progress = true

# 默认加密文件后缀
default_extension = ".leo"

# 密钥文件位置（支持 ~ 扩展）
key_file_path = "~/.config/leolock/keys.toml"
```

#### 配置特性
1. **安全优先**: 默认包含所有关键系统目录作为危险路径
2. **环境变量覆盖**: 
   - `LEOLOCK_FORBIDDEN_PATHS`: 用逗号分隔的危险路径列表
   - `LEOLOCK_MAX_FILE_SIZE`: 最大文件大小（字节）
3. **配置文件搜索路径**（按优先级）:
   - `.leolock.toml`（当前目录）
   - `LEOLOCK_CONFIG` 环境变量指定的路径
   - `~/.config/leolock/config.toml`（XDG配置目录）
   - `~/.leolock.toml`（用户主目录）
4. **敏感信息保护**: 密码哈希和盐值不保存到配置文件
5. **初始化状态**: 配置加载时自动检测是否已初始化

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

### 4. 初始化要求
- 必须先运行 `leolock init` 完成初始化
- 未初始化的工具无法执行加密/解密操作
- 初始化状态保存在配置中

## 📁 文件结构

### 用户文件
```
~/.config/leolock/
├── config.toml      # 配置文件（危险路径、文件大小等）
└── keys.toml        # 密钥文件（AES-256 密钥）

~/leolock_key_backup_YYYYMMDD_HHMMSS.enc  # 加密备份文件
```

### 配置文件示例
完整配置示例见 `examples/config.toml`。

### 项目结构
```
leolock/
├── Cargo.toml                    # 项目配置
├── CREATE.md                     # 需求规格文档（已合并）
├── examples/                     # 示例文件
│   └── config.toml               # 示例配置文件
├── src/                          # 源代码
│   ├── main.rs                   # CLI入口和命令解析
│   ├── config.rs                 # 统一配置管理（危险路径、文件大小等）
│   ├── crypto.rs                 # AES-256-GCM加密/解密
│   ├── keymgmt.rs                # 密钥管理（生成、备份、恢复）
│   ├── fileops.rs                # 文件操作（递归、危险路径检查）
│   ├── password.rs               # 密码处理（Argon2id、交互式）
│   ├── errors.rs                 # 错误类型定义
│   └── utils.rs                  # 工具函数（确认、盐值生成、安全删除）
└── README.md                     # 本文档
```

## 📄 许可证

<a href="https://github.com/lxp731/leolock/blob/main/LICENSE" alt="MIT LICENSE">
    <p style="color: black">MIT LICENSE</p>
</a>

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请：
1. 查看本文档
2. 运行 `leolock --help`
3. 提交 Issue

## 📝 版本历史

- **2026-03-09**: 初始需求 (TASK.md)
- **2026-03-09**: 更新需求 (NEW_TASK.md)
- **2026-03-09**: 最终需求 (CREATE.md) - 初始版本
- **2026-03-11**: 架构重构和功能增强 (READMD.md)
  - ✅ 统一配置系统：合并危险路径配置和密码/密钥配置
  - ✅ 配置管理命令：`leolock config show` / `leolock config validate`
  - ✅ 初始化流程优化：`leolock init` 执行完整初始化
  - ✅ 补全命令改进：`leolock complete` 替代 `--completions` 选项
  - ✅ 安全设计：未初始化不能执行加密操作
  - ✅ 代码清理：移除所有未使用的函数和导入
  - ✅ 默认配置：包含23个危险路径，10GB文件大小限制
  - ✅ 版本自动同步：只需修改 `Cargo.toml`，`main.rs` 自动读取
- **状态**: ✅ 已完成所有开发和测试，架构稳定

---

**最后更新**: 2026-03-11  
**作者**: Burgess Leo  
**状态**: ✅ 需求明确，项目已完成，架构稳定

**安全提示**: 请定期备份重要数据，加密不是数据丢失的保险措施。