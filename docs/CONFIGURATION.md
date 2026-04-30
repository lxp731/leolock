## ⚙️ 配置文件说明

LeoLock 采用 TOML 格式管理用户偏好设置。

### 配置文件搜索路径
工具会按以下优先级寻找配置文件：
1.  当前目录下的 `.leolock.toml`
2.  `LEOLOCK_CONFIG` 环境变量指定的路径
3.  `~/.config/leolock/config.toml` (默认位置)
4.  用户主目录下的 `~/.leolock.toml`

### 配置项详解

| 字段 | 类型 | 说明 |
|------|------|------|
| `forbidden_paths` | Array | 禁止加密的系统目录列表 |
| `max_file_size` | Integer | 允许加密的最大单文件大小（字节），`0` 为无限制 |
| `show_progress` | Boolean | 是否在命令行显示进度信息 |
| `default_extension`| String | 加密文件的后缀名，默认为 `.leo` |
| `key_file_path` | String | 存放主密钥的文件路径 |
| `salt` | String | 用于密钥派生的 Base64 编码盐值（初始化时生成） |
| `preserve_original_filename` | Boolean | 加密时是否保留原始文件名 |

### 环境变量参考

可以通过设置以下环境变量来快速调整 LeoLock 的行为：

| 变量名 | 说明 |
|------|------|
| `LEOLOCK_PASSWORD_VAR` | 指定加载密码时默认检查的环境变量名 |
| `LEOLOCK_FORBIDDEN_PATHS` | 逗号分隔的禁止路径，将追加到默认列表 |
| `LEOLOCK_MAX_FILE_SIZE` | 设置最大文件大小限制 |

### 示例配置 (`config.toml`)
```toml
# 禁止加密的关键目录
forbidden_paths = [
    "/etc", "/bin", "/sbin", "/boot", "/dev"
]

# 限制加密文件不超过 5GB
max_file_size = 5368709120

# 默认加密文件名
preserve_original_filename = false

# 密钥存储位置
key_file_path = "~/.config/leolock/keys.toml"

# 内部使用（请勿手动修改）
initialized = true
salt = "..."
file_format_version = 3
```
