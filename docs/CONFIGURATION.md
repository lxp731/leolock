## ⚙️ 配置文件说明

LeoLock 采用 TOML 格式管理用户偏好设置，配置文件分为四个段：`[program]`、`[core]`、`[auth]`、`[api]`。

### 配置文件搜索路径
工具会按以下优先级寻找配置文件：
1.  当前目录下的 `.leolock.toml`
2.  `LEOLOCK_CONFIG` 环境变量指定的路径
3.  `~/.config/leolock/config.toml` (默认位置)
4.  用户主目录下的 `~/.leolock.toml`

---

### `[program]` — 程序行为

| 字段 | 类型 | 默认值 | 说明 |
|------|------|------|------|
| `forbidden_paths` | Array | 17个系统目录 | 禁止加密的系统目录列表 |
| `max_file_size` | Integer | 10737418240 (10GB) | 允许加密的最大单文件大小（字节），`0` 为无限制 |
| `default_extension` | String | `".leo"` | 加密文件的后缀名 |
| `key_file_path` | String | `"~/.config/leolock/keys.toml"` | 存放主密钥的文件路径 |
| `preserve_original_filename` | Boolean | `false` | 加密时是否保留原始文件名 |
| `show_progress` | Boolean | `true` | 是否在命令行显示进度条 |
| `file_format_version` | Integer | `2` | 加密文件格式版本 |

### `[core]` — 加密核心参数

| 字段 | 类型 | 默认值 | 说明 |
|------|------|------|------|
| `salt` | String | 初始化时生成 | Base64 编码的 16 字节随机盐值，用于 Argon2id 密钥派生。缺失即表示未初始化。**丢失盐值将导致所有加密文件永久无法解密。** |
| `argon2_m_cost` | Integer | `19456` | Argon2id 内存成本 (KB)，约 19MB。值越大暴力破解越难，但内存占用越高。 |
| `argon2_t_cost` | Integer | `2` | Argon2id 迭代次数。值越大暴力破解越慢，但加解密/unlock 等候时间越长。 |
| `argon2_p_cost` | Integer | `1` | Argon2id 并行度（lane 数）。保守设 1，避免给攻击者 GPU 并行可乘之机。 |

### `[auth]` — API 鉴权

| 字段 | 类型 | 说明 |
|------|------|------|
| `api_key_hash` | String | API Key 的 Argon2id 哈希值（初始化时自动生成） |
| `jwt_secret` | String | JWT 签名密钥，256 位随机数 base64 编码（初始化时自动生成） |

### `[api]` — API 服务

| 字段 | 类型 | 默认值 | 说明 |
|------|------|------|------|
| `bind_address` | String | `"127.0.0.1"` | API 服务监听地址 |
| `port` | Integer | `3000` | API 服务监听端口 |

---

### 通过 CLI 修改配置

除 `salt`、`api_key_hash`、`jwt_secret` 外，其余配置项均可用 `leolock config set` 动态修改，无需手动编辑文件：

```bash
leolock config set server.port 3300
leolock config set core.argon2_m_cost 65536
leolock config set program.show_progress false
```

禁止加密路径用专用子命令管理：

```bash
leolock config add-forbidden /mnt/external
leolock config remove-forbidden /mnt/external
```

### 环境变量

| 变量名 | 说明 |
|------|------|
| `LEOLOCK_PASSWORD_VAR` | 指定 `--env-pass` 选项默认检查的环境变量名 |
| `LEOLOCK_CONFIG` | 指定额外的配置文件路径 |

### 初始化后生成的示例配置

```toml
[program]
forbidden_paths = ["/bin", "/sbin", "/usr/bin", "/usr/sbin", "/lib", "/lib64", "/usr/lib", "/usr/lib64", "/boot", "/dev", "/proc", "/sys", "/run", "/etc", "/root", "/var", "/tmp"]
max_file_size = 10737418240
default_extension = ".leo"
key_file_path = "~/.config/leolock/keys.toml"
preserve_original_filename = false
show_progress = true
file_format_version = 2

[core]
salt = "<base64编码的16字节随机盐值>"
argon2_m_cost = 19456
argon2_t_cost = 2
argon2_p_cost = 1

[auth]
api_key_hash = "<API Key 的 Argon2id 哈希>"
jwt_secret = "<256位随机数 base64>"

[api]
bind_address = "127.0.0.1"
port = 3000
```
