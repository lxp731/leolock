
<!-- @import "[TOC]" {cmd="toc" depthFrom=1 depthTo=6 orderedList=false} -->

<!-- code_chunk_output -->

- [CREATE.md - LeoLock 最终需求规格](#createmd---leolock-最终需求规格)
  - [📋 项目概述](#-项目概述)
  - [🎯 最终需求](#-最终需求)
    - [1. 项目信息](#1-项目信息)
    - [2. 核心功能需求](#2-核心功能需求)
  - [🔄 命令结构与期望效果](#-命令结构与期望效果)
    - [1. **初始化** (`leolock init`)](#1-初始化-leolock-init)
    - [2. **加密文件** (`leolock encrypt`)](#2-加密文件-leolock-encrypt)
    - [3. **解密文件** (`leolock decrypt`)](#3-解密文件-leolock-decrypt)
    - [4. **修改密码** (`leolock password update`)](#4-修改密码-leolock-password-update)
    - [5. **更新密钥** (`leolock key update`)](#5-更新密钥-leolock-key-update)
    - [6. **恢复密钥** (`leolock recover`)](#6-恢复密钥-leolock-recover)
    - [7. **帮助系统**](#7-帮助系统)
  - [🔧 技术实现规格](#-技术实现规格)
    - [1. 密码学算法](#1-密码学算法)
    - [2. 文件处理](#2-文件处理)
    - [3. 安全特性](#3-安全特性)
    - [4. 配置管理](#4-配置管理)
    - [5. 文件命名规范](#5-文件命名规范)
  - [🏗️ 项目结构](#️-项目结构)
  - [📦 依赖库](#-依赖库)
  - [🧪 测试要求](#-测试要求)
  - [⚠️ 重要注意事项](#️-重要注意事项)
  - [📝 版本历史](#-版本历史)

<!-- /code_chunk_output -->



# CREATE.md - LeoLock 最终需求规格

## 🚀 变更摘要 (2026-03-11)

### 主要架构改进
1. **统一配置系统** - 合并危险路径配置和密码/密钥配置
2. **配置管理命令** - `config show` / `config validate` 子命令
3. **初始化流程优化** - `init` 执行完整初始化（密码+密钥+配置）
4. **补全命令改进** - `complete` 子命令替代 `--completions` 选项
5. **安全设计强化** - 未初始化不能执行加密操作

### 默认配置特性
- **23个危险路径** - 保护系统关键目录
- **10GB文件大小限制** - 防止意外加密大文件
- **可配置性** - 所有安全设置都可自定义
- **环境变量覆盖** - 支持运行时配置覆盖

## 📋 项目概述

**LeoLock** 是一个使用 Rust 开发的 Linux 命令行文件加密工具，提供安全的 AES-256-GCM 加密解密功能，具有现代化的密码学算法和用户友好的交互界面。

## 🎯 最终需求

### 1. 项目信息
- **工具名称**: `leolock` (原名 `bjt372`)
- **项目位置**: `/home/knight/workspace/leolock/`
- **编程语言**: Rust
- **目标平台**: Linux

### 2. 核心功能需求
1. **文件加密/解密**: 支持单个文件和文件夹的递归处理
2. **安全算法**: AES-256-GCM 加密 + Argon2id 密码哈希
3. **配置管理**: 用户级配置文件 (`~/.config/leolock/`)
4. **备份机制**: 初始化时一次性创建加密备份
5. **恢复功能**: 专用恢复命令从备份恢复密钥

## 🔄 命令结构与期望效果

### 1. **初始化** (`leolock init`)
**期望效果**:
```bash
$ leolock init
🚀 开始初始化 leolock 工具...

🔐 设置初始密码
请输入密码（至少8位，包含数字和字母）: 
请确认密码: 

🔑 生成加密密钥...
✅ 密钥生成成功

📁 创建配置文件...
✅ 已生成配置文件: /home/knight/.config/leolock/config.toml

你可以编辑此文件来自定义设置:
  - 危险路径列表 (forbidden_paths)
  - 最大文件大小 (max_file_size)
  - 显示进度 (show_progress)
  - 默认扩展名 (default_extension)
  - 密钥文件路径 (key_file_path)

💾 创建备份文件...
✅ 备份文件已创建: /home/knight/leolock_key_backup_20260311_020000.enc

⚠️  重要提醒：
1. 请妥善保管备份文件！
2. 备份文件已用您设置的密码加密
3. 如果忘记密码或丢失备份，将无法恢复数据
```

### 2. **加密文件** (`leolock encrypt`)
**期望效果**:
```bash
# 基本用法（交互式密码，默认删除源文件）
$ leolock encrypt secret.txt
🔐 加密操作需要验证密码
请输入密码: 
✅ 加密完成: secret.txt -> secret.txt.leo
🗑️  已安全删除源文件: secret.txt

# 保留源文件
$ leolock encrypt important.txt --keep-original
🔐 加密操作需要验证密码
请输入密码: 
✅ 加密完成: important.txt -> important.txt.leo
📁 已保留源文件（使用 --keep-original 选项）

# 加密文件夹
$ leolock encrypt documents/
🔐 加密操作需要验证密码
请输入密码: 
开始加密目录: documents/
----------------------------------------
✅ 加密完成: documents/report.pdf -> documents/report.pdf.leo
✅ 加密完成: documents/notes.txt -> documents/notes.txt.leo
----------------------------------------
加密完成:
  ✅ 成功: 2 个文件
  ❌ 失败: 0 个文件
```

### 3. **解密文件** (`leolock decrypt`)
**期望效果**:
```bash
# 基本用法（交互式密码，默认删除加密文件）
$ leolock decrypt secret.txt.leo
🔐 解密操作需要验证密码
请输入密码: 
✅ 解密完成: secret.txt.leo -> secret.txt
🗑️  已安全删除加密文件: secret.txt.leo

# 保留加密文件
$ leolock decrypt important.txt.leo --keep-original
🔐 解密操作需要验证密码
请输入密码: 
✅ 解密完成: important.txt.leo -> important.txt
📁 已保留加密文件（使用 --keep-original 选项）

# 解密混合目录
$ leolock decrypt mixed_folder/
🔐 解密操作需要验证密码
请输入密码: 
开始解密目录: mixed_folder/
----------------------------------------
✅ 解密完成: mixed_folder/a.txt.leo -> mixed_folder/a.txt
⏭️  跳过: mixed_folder/b.txt (非加密文件)
✅ 解密完成: mixed_folder/c.txt.leo -> mixed_folder/c.txt
----------------------------------------
解密完成:
  ✅ 成功: 2 个文件
  ⏭️  跳过: 1 个非加密文件
  ❌ 失败: 0 个文件
提示：跳过了非加密文件（无.leo后缀）
```

### 4. **修改密码** (`leolock password update`)
**期望效果**:
```bash
$ leolock password update
🔐 修改密码
请输入旧密码: 
✅ 旧密码验证通过

请输入新密码（至少8位，包含数字和字母）: 
请再次输入新密码确认: 
✅ 密码修改成功！
```

### 5. **更新密钥** (`leolock key update`)
**期望效果**:
```bash
$ leolock key update
============================================================
⚠ 警告：重新生成密钥是危险操作！
============================================================

这将导致：
1. 旧密钥加密的所有文件将无法解密！
2. 旧的备份文件将失效！

是否继续？ [y/N]: y

🔑 生成新密钥...
✅ 密钥文件已保存: "/home/knight/.config/leolock/leolock.key"

🔐 为新密钥创建备份
是否创建新密钥的备份文件？ [Y/n]: y
请输入密码以加密备份文件：
密码: 
✅ 新备份已创建: /home/knight/leolock_key_backup_20260309_193712.enc

⚠️  重要提醒：
1. 旧密钥加密的所有文件将无法解密！
2. 旧的备份文件已失效！
3. 请立即备份新密钥文件: "/home/knight/.config/leolock/leolock.key"
```

### 6. **恢复密钥** (`leolock recover`)
**期望效果**:
```bash
$ leolock recover --backup ~/leolock_key_backup_20260311_020000.enc
🔄 从备份文件恢复密钥...
备份文件: /home/knight/leolock_key_backup_20260311_020000.enc

🔐 验证当前操作密码
请输入当前操作密码: 
✅ 当前操作密码验证通过

🔐 输入备份文件密码
提示：如果修改过密码，请输入修改前的密码（备份创建时的密码）
请输入备份密码: 

⚠️  检测到现有配置
这将覆盖现有密钥文件，继续吗？ [y/N]: y

✅ 密钥恢复成功！
密钥文件已保存到: "/home/knight/.config/leolock/keys.toml"
```

### 7. **配置管理** (`leolock config`)
**期望效果**:
```bash
# 显示当前配置
$ leolock config show
当前配置:
==================================================
危险路径列表 (23 个):
  - /bin
  - /sbin
  - /usr/bin
  - /usr/sbin
  - /lib
  - /lib64
  - /usr/lib
  - /usr/lib64
  - /boot
  - /dev
  - /proc
  - /sys
  - /run
  - /etc
  - /root
  - /var
  - /tmp
  - /usr/local/bin
  - /usr/local/sbin
  - /opt
  - /home
  - /mnt
  - /media

最大文件大小: 10737418240 bytes (10.00 GB)
显示进度: true
默认扩展名: .leo
密钥文件路径: ~/.config/leolock/keys.toml
已初始化: true

配置文件搜索路径:
  ✓ /home/knight/.config/leolock/config.toml

# 验证配置文件
$ leolock config validate
验证配置文件...
✅ 配置文件格式正确
✅ 包含 23 个危险路径
✅ 最大文件大小: 10.00 GB

路径安全检查（基于当前配置）:
❌ 危险 /bin/ls
❌ 危险 /etc/passwd
❌ 危险 /tmp/test.txt
✅ 安全 /home/user/document.txt
✅ 安全 ./test.txt
❌ 危险 /usr/bin/bash
❌ 危险 /var/log/syslog
```

### 8. **补全脚本生成** (`leolock complete`)
**期望效果**:
```bash
# 生成 Bash 补全脚本
$ leolock complete bash
_leolock() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""
    # ... 完整的补全脚本
}

# 生成 Zsh 补全脚本
$ leolock complete zsh
#compdef leolock
autoload -U is-at-least
_leolock() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1
    # ... 完整的补全脚本
}

# 生成 Fish 补全脚本
$ leolock complete fish
# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_leolock_global_optspecs
    string join \n h/help V/version
end
# ... 完整的补全脚本
```

### 9. **帮助系统**
**期望效果**:
```bash
$ leolock --help
leolock 1.0.0
文件加密解密工具

Usage: leolock [COMMAND]

Commands:
  init      初始化工具（创建配置和密钥）
  recover   从备份文件恢复密钥
  password  密码管理
  key       密钥管理
  config    配置管理
  encrypt   加密文件或文件夹
  decrypt   解密文件或文件夹
  complete  生成shell补全脚本
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     显示帮助信息
  -V, --version  Print version
```

## 🔧 技术实现规格

### 1. 密码学算法
- **文件加密**: AES-256-GCM (认证加密)
- **密码哈希**: Argon2id (抗GPU攻击，随机盐值)
- **密钥派生**: Argon2id (用于备份文件加密)

### 2. 文件处理
- **扩展名处理**: `文件名` → `文件名.leo` → `文件名`
- **递归处理**: 支持文件夹的递归加密/解密
- **符号链接**: 加密/解密源文件，跳过危险路径
- **错误处理**: 宽松模式，单个文件失败不影响其他文件

### 3. 安全特性
- **密码强度**: 至少8位，包含数字和字母
- **尝试限制**: 密码验证最多3次
- **危险路径跳过**: `/bin`, `/sbin`, `/usr`, `/lib` 等系统目录
- **安全删除**: 覆盖数据后删除源文件

### 4. 配置管理
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

### 5. 文件命名规范
- **配置文件**: `config.toml`
- **密钥文件**: `keys.toml`
- **备份文件**: `leolock_key_backup_YYYYMMDD_HHMMSS.enc`
- **加密文件**: `原文件名.leo`
- **示例配置**: `examples/config.toml`

## 🏗️ 项目结构
```
leolock/
├── Cargo.toml                    # 项目配置
├── CREATE.md                     # 需求规格文档
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
└── README.md                     # 用户文档
```

## 📦 依赖库
```toml
[dependencies]
clap = { version = "4.0", features = ["derive", "env"] }
clap_complete = "4.0"
argon2 = "0.5"
aes-gcm = "0.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
walkdir = "2.4"
rpassword = "7.0"
anyhow = "1.0"
thiserror = "1.0"
chrono = "0.4"
dirs = "5.0"
hex = "0.4"
base64 = "0.21"
rand = "0.8"
getrandom = "0.2"
sha2 = "0.10"
shellexpand = "3.0"  # 路径扩展（支持 ~ 扩展）
```

## 🧪 测试要求
1. **功能测试**: 所有命令的完整流程测试
2. **安全测试**: 密码验证、备份恢复、错误处理
3. **边界测试**: 大文件、特殊字符、权限问题
4. **集成测试**: 使用 `/home/knight/test` 目录进行测试

## ⚠️ 重要注意事项
1. **备份责任**: 用户必须妥善保管备份文件
2. **密钥更新风险**: 重新生成密钥将导致旧文件无法解密
3. **密码安全**: 使用强密码并定期更换
4. **文件扩展名**: 加密后添加 `.leo` 后缀，解密后移除

## 📝 版本历史
- **2026-03-09**: 初始需求 (TASK.md)
- **2026-03-09**: 更新需求 (NEW_TASK.md)
- **2026-03-09**: 最终需求 (CREATE.md) - 初始版本
- **2026-03-11**: 架构重构和功能增强
  - ✅ 统一配置系统：合并危险路径配置和密码/密钥配置
  - ✅ 配置管理命令：`leolock config show` / `leolock config validate`
  - ✅ 初始化流程优化：`leolock init` 执行完整初始化
  - ✅ 补全命令改进：`leolock complete` 替代 `--completions` 选项
  - ✅ 安全设计：未初始化不能执行加密操作
  - ✅ 代码清理：移除所有未使用的函数和导入
  - ✅ 默认配置：包含23个危险路径，10GB文件大小限制
- **状态**: ✅ 已完成所有开发和测试，架构稳定

---

**最后更新**: 2026-03-11  
**作者**: Burgess Leo  
**状态**: ✅ 需求明确，项目已完成，架构稳定