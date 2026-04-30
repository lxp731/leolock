# LeoLock 🔒

一个安全的文件加密解密命令行工具，使用 AES-256-GCM 加密算法和 Argon2id 密码哈希。

## ✨ 特性

- **军用级加密**: AES-256-GCM 认证加密，现已升级 **AAD (附加认证数据)** 保护，防止文件头被篡改。
- **安全密码哈希**: Argon2id 抗 GPU/ASIC 攻击。
- **内存零残留**: 集成 `zeroize` 技术，确保密码和密钥在内存中使用后立即擦除，防止内存嗅探。
- **极速大文件处理**: 采用流式 I/O 重构，3GB 文件加密仅需约 14 秒（实测环境），且内存占用极低。
- **原子化操作**: “先写后换”机制，确保即使在加密中途崩溃或断电，原始数据也不会损坏。
- **双重加密模式**: 支持文件名加密（完全模式）或仅加密内容（快速模式）。
- **递归处理**: 支持文件和文件夹的批量加密。
- **备份恢复**: 初始化时自动创建加密密钥备份。
- **Tab 补全**: 支持 Bash、Zsh、Fish、PowerShell、Elvish。
- **运行时安全检查**: 自动检测配置文件权限（需 600 权限）。

## 🚀 快速开始

### 1. 安装

**从源码编译 (推荐):**
```bash
git clone https://github.com/lxp731/leolock.git
cd leolock
cargo build --release
# 使用 release 版本以获得最高性能
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
# 完全模式（默认）：加密文件内容和文件名
leolock encrypt secret.txt

# 快速模式：仅加密文件内容，不加密文件名
leolock encrypt secret.txt --fast

# 保留原文件
leolock encrypt secret.txt --keep
```
输入密码，文件被加密为：
- 完全模式：`随机哈希.leo`（文件名加密）
- 快速模式：`secret.txt.leo`（保留原文件名）

### 4. 解密文件
```bash
leolock decrypt secret.txt.leo
```
输入密码，恢复原文件（自动识别文件格式）。

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
| `leolock recover --backup <文件>` | 从备份文件恢复密钥 |
| `leolock completions <shell>` | 生成shell补全脚本 |

**常用选项:**
- `-k, --keep`: 保留源文件（不删除）
- `-F, --fast`: 快速模式（不加密文件名）
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

### 版本 1.1.0 (当前)
- **性能质跃**: 引入流式加密重构，大幅提升大文件处理速度（14s/3GB）。
- **内存安全**: 集成 `zeroize` 确保敏感数据在内存中无残留。
- **完整性增强**: 升级 V3 文件格式，引入 AAD (附加认证数据) 保护元数据。
- **鲁棒性增强**: 实现原子化文件写入，防止操作中断导致的数据损坏。

### 版本 1.0.3
- 简化密码管理，移除单独的密码哈希文件
- 添加文件列表功能，支持排序和原文件名显示

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

**最后更新:** 2026-04-30  
**项目状态:** ✅ 安全加固，性能极致，稳定生产可用

**安全提示:** 请定期备份重要数据，加密不是数据丢失的保险措施。