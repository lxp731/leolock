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
- **Tab 补全**: 支持 Bash、Zsh、Fish、PowerShell、Elvish
- **文件列表功能**: 查看加密文件信息，支持排序和原文件名显示
- **运行时安全检查**: 自动检测配置文件权限问题
- **简化密码管理**: 无需单独的密码哈希文件，密码直接派生密钥

## 🚀 快速开始

### 1. 安装

**从源码编译（推荐）:**
```bash
git clone https://github.com/lxp731/leolock.git
cd leolock
cargo build --release
sudo cp target/release/leolock /usr/local/bin/
```

**或使用包管理器:** 详见 [docs/INSTALLATION.md](docs/INSTALLATION.md)

### 2. 初始化
```bash
leolock init
```
设置密码，生成配置和密钥。

### 3. 加密文件
```bash
leolock encrypt secret.txt
```
输入密码，文件被加密为 `secret.txt.leo`。

### 4. 解密文件
```bash
leolock decrypt secret.txt.leo
```
输入密码，恢复原文件。

### 5. 查看文件
```bash
# 列出加密文件
leolock list .

# 按大小排序
leolock list . --sort-by-size desc

# 显示原文件名（需要密码）
leolock list . --show-original
```

## 📖 基本命令

| 命令 | 说明 |
|------|------|
| `leolock init` | 初始化工具 |
| `leolock encrypt <路径>` | 加密文件或文件夹 |
| `leolock decrypt <路径>` | 解密文件或文件夹 |
| `leolock list <路径>` | 列出加密文件信息 |
| `leolock completions <shell>` | 生成shell补全脚本 |

**常用选项:**
- `-k, --keep`: 保留源文件（不删除）
- `--show-original`: 显示原文件名（需要密码）
- `--sort-by-size <asc/desc>`: 按文件大小排序

**完整命令参考:** 详见 [docs/COMMANDS.md](docs/COMMANDS.md)

## 📦 安装选项

### 从源码编译（推荐）
```bash
cargo build --release
sudo cp target/release/leolock /usr/local/bin/
```

### 生成补全脚本

**Bash:**
```bash
leolock completions bash -o ~/.bash_completion.d/
```

**Zsh:**
```bash
leolock completions zsh -o ~/.zsh/completions/
```

**其他shell:** 详见 [docs/INSTALLATION.md](docs/INSTALLATION.md)

**详细安装指南:** 详见 [docs/INSTALLATION.md](docs/INSTALLATION.md)

## 🔐 安全特性

### 核心安全
- **AES-256-GCM**: 军用级认证加密
- **Argon2id**: 抗GPU/ASIC攻击的密码哈希
- **随机盐值**: 每个实例唯一，防止彩虹表攻击
- **文件权限保护**: 自动设置配置文件权限为 600

### 安全限制
- **危险路径保护**: 默认禁止加密16个系统目录
- **文件大小限制**: 默认10GB，防止意外加密大文件
- **密码强度**: 至少8位，包含数字和字母
- **运行时检查**: 自动检测配置文件权限问题

**详细安全文档:** 详见 [docs/SECURITY.md](docs/SECURITY.md)

## 📁 文档目录

- [docs/INSTALLATION.md](docs/INSTALLATION.md) - 详细安装指南
- [docs/COMMANDS.md](docs/COMMANDS.md) - 完整命令参考
- [docs/SECURITY.md](docs/SECURITY.md) - 安全特性文档
- [docs/CONFIGURATION.md](docs/CONFIGURATION.md) - 配置文件说明
- [docs/WARNINGS.md](docs/WARNINGS.md) - 重要警告
- [docs/STRUCTURE.md](docs/STRUCTURE.md) - 文件结构说明

## ⚠️ 重要提醒

1. **备份至关重要**: 初始化时自动创建备份，请立即转移到安全位置
2. **记住密码**: 忘记密码将导致所有加密数据永久丢失
3. **文件权限**: 配置文件包含敏感信息，保持权限为 600

**完整警告列表:** 详见 [docs/WARNINGS.md](docs/WARNINGS.md)

## 📝 版本历史

### 版本 1.0.3 (当前)
- 简化密码管理，移除单独的密码哈希文件
- 添加文件列表功能，支持排序和原文件名显示
- 增强配置文件安全性，自动权限设置
- 完善shell补全支持（5种shell）

### 版本 1.0.2
- 文件名加密功能，增强隐私保护
- 新文件格式版本2，支持文件名元数据存储
- 向后兼容，支持旧版文件解密

**完整版本历史:** 详见 [docs/CHANGELOG.md](docs/CHANGELOG.md)

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📞 支持

如有问题，请：
1. 查看本文档和 docs/ 目录
2. 运行 `leolock --help`
3. 提交 [GitHub Issue](https://github.com/lxp731/leolock/issues)

---

**最后更新:** 2026-03-15  
**项目状态:** ✅ 功能完整，安全优化，稳定可用

**安全提示:** 请定期备份重要数据，加密不是数据丢失的保险措施。