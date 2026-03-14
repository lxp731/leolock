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
| `-k, --keep` | 保留源文件（不删除） |

### 文件列表

| 命令 | 说明 |
|------|------|
| `leolock list <路径>` | 列出加密文件信息 |
| `--show-original` | 显示原文件名（需要密码验证） |
| `--sort-by-size <asc/desc>` | 按文件大小排序（升序/降序） |

### Shell 补全

| 命令 | 说明 |
|------|------|
| `leolock completions bash` | 生成 Bash 补全脚本 |
| `leolock completions zsh` | 生成 Zsh 补全脚本 |
| `leolock completions fish` | 生成 Fish 补全脚本 |
| `leolock completions powershell` | 生成 PowerShell 补全脚本 |
| `leolock completions elvish` | 生成 Elvish 补全脚本 |
| `-o, --output-dir <目录>` | 指定输出目录（默认：当前目录） |

### 帮助系统

| 命令 | 说明 |
|------|------|
| `leolock --help` | 显示帮助信息 |
| `leolock <命令> --help` | 显示子命令帮助 |

