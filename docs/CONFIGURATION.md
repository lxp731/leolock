# ⚙️ 配置文件说明

## 配置文件位置
LeoLock 的配置文件位于 `~/.config/leolock/config.toml`。

## 配置文件示例
```toml
# ~/.config/leolock/config.toml
# 危险路径列表（禁止处理的系统目录）
forbidden_paths = [
    "/bin", "/sbin", "/usr/bin", "/usr/sbin",
    "/lib", "/lib64", "/usr/lib", "/usr/lib64",
    "/boot", "/dev", "/proc", "/sys", "/run",
    "/etc", "/root", "/var", "/tmp",
]

# 最大文件大小（字节），0表示无限制
max_file_size = 10737418240  # 10GB

# 是否启用进度显示
show_progress = true

# 默认加密文件后缀
default_extension = ".leo"

# 密钥文件位置（支持 ~ 扩展）
key_file_path = "~/.config/leolock/keys.toml"

# 盐值（base64编码，用于密钥派生）
salt = "EcwA1CVMrpIx2zbFCWegHw=="

# 是否已初始化
initialized = true

# 是否保留原文件名（false=加密文件名，true=保留文件名）
preserve_original_filename = false

# 加密文件格式版本
file_format_version = 2
```

## 配置说明

### 危险路径 (forbidden_paths)
防止意外加密系统关键目录的路径列表。默认包含16个核心系统目录。

### 最大文件大小 (max_file_size)
防止意外加密大文件。默认10GB（10737418240字节）。

### 显示进度 (show_progress)
是否在加密/解密过程中显示进度信息。

### 默认扩展名 (default_extension)
加密文件的默认扩展名，默认为 `.leo`。

### 密钥文件路径 (key_file_path)
主密钥文件的存储位置。

### 盐值 (salt)
随机生成的盐值，用于密钥派生。每个实例唯一。

### 初始化状态 (initialized)
标识工具是否已完成初始化。

### 保留原文件名 (preserve_original_filename)
控制是否加密文件名。`false`表示加密文件名，`true`表示保留原文件名。

### 文件格式版本 (file_format_version)
加密文件的格式版本，用于向后兼容。

## 环境变量覆盖
可以在运行时通过环境变量覆盖配置：

```bash
# 覆盖危险路径
export LEOLOCK_FORBIDDEN_PATHS="/tmp,/home/test"

# 覆盖最大文件大小
export LEOLOCK_MAX_FILE_SIZE=5368709120  # 5GB

# 运行命令
leolock encrypt file.txt
```

## 配置文件搜索顺序
1. `.leolock.toml`（当前目录）
2. `LEOLOCK_CONFIG` 环境变量指定的路径
3. `~/.config/leolock/config.toml`（XDG配置目录）
4. `~/.leolock.toml`（用户主目录）

## 文件权限
配置文件自动设置权限为 `600`（仅所有者可读写），保护盐值等敏感信息。
