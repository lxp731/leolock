## 📖 完整命令参考

### 初始化与恢复

| 命令 | 说明 |
|------|------|
| `leolock init` | 初始化工具（创建配置、密钥、盐值） |
| `--save-to-keyring` | (Init 选项) 初始化后将密码保存到系统钥匙串 |
| `leolock recover --backup <文件>` | 从备份文件恢复密钥 |

### 加密解密 (通用选项)

以下选项适用于 `encrypt`, `decrypt`, `list`, `recover` 命令：

| 选项 | 说明 |
|------|------|
| `--env-pass <变量名>` | 从指定的环境变量加载密码 |
| `--keyring` | 从系统钥匙串自动加载密码 |
| `--stdin` | 从标准输入加载密码（适用于管道操作） |

### 加密命令

`leolock encrypt <路径>`

| 选项 | 说明 |
|------|------|
| `-k, --keep` | 保留源文件（不删除） |
| `-F, --fast` | 快速模式：不加密文件名，仅加密文件内容 |

### 解密命令

`leolock decrypt <路径>`

| 选项 | 说明 |
|------|------|
| `-k, --keep` | 保留加密文件（不删除） |

### 文件列表

`leolock list <路径>`

| 选项 | 说明 |
|------|------|
| `--show-original` | 显示原文件名（需要密码验证） |
| `--sort-by-size <asc\|desc>` | 按文件大小排序（升序/降序） |
| `--format <table\|json\|simple>` | 输出格式（默认 table） |

### 配置管理

`leolock config <子命令>`

| 子命令 | 说明 |
|------|------|
| `show` | 显示当前配置文件内容 |
| `validate` | 验证配置文件的完整性和权限 |
| `set <key> <value>` | 修改指定配置项 |
| `add-forbidden <path>` | 添加禁止加密的路径 |
| `remove-forbidden <path>` | 移除禁止加密的路径 |

### config set 支持的键

| 键 | 类型 | 示例 |
|------|------|------|
| `program.max_file_size` | 整数 | `leolock config set program.max_file_size 1073741824` |
| `program.default_extension` | 字符串 | `leolock config set program.default_extension .enc` |
| `program.key_file_path` | 字符串 | `leolock config set program.key_file_path ~/.leolock.keys` |
| `program.preserve_original_filename` | 布尔 | `leolock config set program.preserve_original_filename true` |
| `program.show_progress` | 布尔 | `leolock config set program.show_progress false` |
| `program.file_format_version` | 整数 | `leolock config set program.file_format_version 3` |
| `core.argon2_m_cost` | 整数 | `leolock config set core.argon2_m_cost 65536` |
| `core.argon2_t_cost` | 整数 | `leolock config set core.argon2_t_cost 3` |
| `core.argon2_p_cost` | 整数 | `leolock config set core.argon2_p_cost 2` |

布尔值支持: `true`/`false`/`yes`/`no`/`1`/`0`/`on`/`off`

### Shell 补全

`leolock completions <shell>`

支持的 shell: `bash`, `zsh`, `fish`, `powershell`, `elvish`。
使用 `-o, --output-dir <目录>` 指定输出位置。

### 帮助系统

- `leolock --help`: 显示全局帮助。
- `leolock <命令> --help`: 显示特定命令的详细用法。
