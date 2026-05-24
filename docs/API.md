# LeoLock API 参考

> 版本: v1.3.0 | 基础地址: `http://127.0.0.1:3000`

## 认证

除 `/health`、`/status`、`/auth/login`、`/init` 外，所有端点需要 JWT Token：

```bash
# 获取 Token
curl -s -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"api_key": "<你的API Key>"}'

# 返回: {"token": "eyJ...", "expires_in": 1800, "token_type": "Bearer"}

# 后续请求带上 Token
# -H "Authorization: Bearer $TOKEN"
```

Token 有效期 30 分钟，过期需重新登录。

---

## 端点

### 健康检查

```bash
GET /api/v1/health
```

不需要认证。

```bash
curl http://127.0.0.1:3000/api/v1/health
# → ok
```

---

### 服务状态

```bash
GET /api/v1/status
```

不需要认证。

```bash
curl -s http://127.0.0.1:3000/api/v1/status | python3 -m json.tool
```

响应：
```json
{
    "initialized": true,
    "locked": true,
    "version": "1.3.0"
}
```

| 字段 | 说明 |
|------|------|
| `initialized` | 是否已完成初始化 |
| `locked` | 是否处于锁定状态（需 unlock 才能加解密） |
| `version` | 服务版本号 |

---

### 初始化

```bash
POST /api/v1/init
```

首次使用或重置时调用，不需要认证。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/init \
  -H 'Content-Type: application/json' \
  -d '{"password": "你的密码"}' | python3 -m json.tool
```

响应：
```json
{
    "status": "initialized",
    "message": "✅ 初始化完成，备份已保存至 /home/user/leolock_key_backup_20260524_200000.enc",
    "api_key": "439jgyemZrkIX0VTspNtJO5rt1yPTdKEtqs9l-6L_Mg"
}
```

> **注意**：`api_key` 仅在此时明文返回一次，务必妥善保存。

---

### 解锁

```bash
POST /api/v1/unlock
```

需要认证。输入密码，派生 AES 密钥并驻留内存。密钥不正确不会立即报错（真正的校验在加解密时发生）。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/unlock \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}' | python3 -m json.tool
```

响应：
```json
{
    "status": "unlocked",
    "message": "🔓 服务已解锁，密钥已加载到内存"
}
```

---

### 锁定

```bash
POST /api/v1/lock
```

需要认证。立即从内存中擦除密钥（zeroize），之后加解密返回 423。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/lock \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

响应：
```json
{
    "status": "locked",
    "message": "🔒 服务已锁定，密钥已从内存擦除"
}
```

---

### 加密

```bash
POST /api/v1/encrypt
```

需要认证 + 服务已解锁。multipart 上传文件，返回 V3 格式加密文件。

```bash
# 加密单个文件
curl -s -X POST http://127.0.0.1:3000/api/v1/encrypt \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@document.pdf" \
  -o document.leo
```

加密流程：文件名用 AES-256-GCM 加密存入文件头 → 文件内容 1MB 分块流式加密 → AAD 元数据防篡改 → 输出 `.leo` 文件。

响应头：
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="a3f8e2d1.leo"
```

---

### 解密

```bash
POST /api/v1/decrypt
```

需要认证 + 服务已解锁。multipart 上传 `.leo` 文件，返回解密后的原始文件（文件名从 V3 头中恢复）。

```bash
# 解密文件
curl -s -X POST http://127.0.0.1:3000/api/v1/decrypt \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@document.leo" \
  -o document_decrypted.pdf
```

响应头：
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="document.pdf"
```

解密成功与否取决于 unlock 时输入的密码是否正确。密码错误 → 解密失败返回 400。

---

### 文件列表

```bash
GET /api/v1/files
```

需要认证。列出指定目录下的 `.leo` 加密文件，支持分页和排序。锁定时也可调用（原文件名显示为 `[需要密钥查看]`）。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files?path=/data/encrypted&sort=size_desc&page=1&per_page=50" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

查询参数：

| 参数 | 必填 | 默认值 | 说明 |
|------|------|--------|------|
| `path` | 是 | — | 要扫描的目录 |
| `page` | 否 | 1 | 页码 |
| `per_page` | 否 | 50 | 每页数量（1-200） |
| `sort` | 否 | — | `size_asc` / `size_desc` / `name_asc` / `name_desc` |

响应：
```json
{
  "items": [
    {
      "id": "L3RtcC90ZXN0X2dhbW1hLmxlbw",
      "version": 3,
      "encrypted_size": 108,
      "original_name": "test_gamma.txt",
      "decryptable": true
    }
  ],
  "total": 11,
  "page": 1,
  "per_page": 50
}
```

| 字段 | 说明 |
|------|------|
| `id` | 文件唯一标识（用于后续 get/download/delete 操作） |
| `version` | 加密格式版本（1/2/3） |
| `encrypted_size` | 加密文件大小（字节） |
| `original_name` | 原文件名（锁定时显示 `[需要密钥查看]`） |
| `decryptable` | 当前密钥能否解密 |

---

### 文件详情

```bash
GET /api/v1/files/get
```

需要认证。查看单个加密文件的详细信息。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files/get?id=L3RtcC90ZXN0X2dhbW1hLmxlbw" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

响应：
```json
{
  "id": "L3RtcC90ZXN0X2dhbW1hLmxlbw",
  "path": "/tmp/test_gamma.leo",
  "version": 3,
  "encrypted_size": 108,
  "original_name": "test_gamma.txt",
  "decryptable": true,
  "exists": true
}
```

---

### 下载解密

```bash
GET /api/v1/files/download
```

需要认证 + 服务已解锁。原地解密 `.leo` 文件并以原始文件名流式返回。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files/download?id=L3RtcC90ZXN0X2dhbW1hLmxlbw" \
  -H "Authorization: Bearer $TOKEN" \
  -o 还原的文件.txt
```

响应头：
```
Content-Type: application/octet-stream
Content-Disposition: attachment; filename="原文件名.txt"
```

---

### 删除文件

```bash
DELETE /api/v1/files/delete
```

需要认证 + 服务已解锁。安全删除加密文件（覆写随机数据后删除）。

```bash
curl -s -X DELETE "http://127.0.0.1:3000/api/v1/files/delete?id=L3RtcC90ZXN0X2dhbW1hLmxlbw" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
```

响应：
```json
{
  "status": "deleted",
  "message": "已删除: /tmp/test_gamma.leo"
}
```

> 只能删除 `.leo` 后缀的加密文件。服务锁定状态下删除返回 423。

---

## 完整调用流程

```bash
# 0. 初次使用：初始化（已有配置可跳过）
curl -s -X POST http://127.0.0.1:3000/api/v1/init \
  -H 'Content-Type: application/json' \
  -d '{"password": "你的密码"}'
# 记下返回的 api_key！

# 1. 登录获取 Token
export TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"api_key": "<你的API Key>"}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")

# 2. 解锁
curl -s -X POST http://127.0.0.1:3000/api/v1/unlock \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}'

# 3. 加密
curl -s -X POST http://127.0.0.1:3000/api/v1/encrypt \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@机密文档.pdf" \
  -o 机密文档.leo

# 4. 浏览加密文件
curl -s "http://127.0.0.1:3000/api/v1/files?path=." \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool

# 5. 下载解密
curl -s "http://127.0.0.1:3000/api/v1/files/download?id=<从列表获取的id>" \
  -H "Authorization: Bearer $TOKEN" \
  -o 机密文档_还原.pdf

# 6. 用完后锁定
curl -s -X POST http://127.0.0.1:3000/api/v1/lock \
  -H "Authorization: Bearer $TOKEN"
```

---

## 错误码

| 状态码 | 含义 | 触发场景 |
|--------|------|----------|
| 200 | 成功 | — |
| 400 | 请求错误 | 密码太弱、API Key 无效、文件解析失败、解密密钥不正确、路径不存在 |
| 401 | 未认证 | Token 缺失、无效或过期 |
| 404 | 未找到 | 路由不存在（检查 URL 拼写） |
| 412 | 前置条件不满足 | 服务未初始化就调用 unlock |
| 423 | 已锁定 | 服务 locked 状态下调用 encrypt/decrypt/download/delete |
| 500 | 服务内部错误 | JWT 未配置、IO 异常 |

## 安全说明

- **默认仅监听 127.0.0.1**，不暴露公网。如需远程访问，应配置反向代理 + TLS
- **密码不落盘不记日志**：unlock 后密码立即 zeroize，派生出的 32 字节密钥驻留内存
- **重启自动锁定**：服务重启后密钥丢失，必须重新 unlock
- **API Key 存储为 Argon2id 哈希**：与密码同等安全级别
- **加密为 V3 格式**：AES-256-GCM + AAD 元数据保护 + 文件名加密
