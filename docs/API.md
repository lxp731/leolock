# LeoLock API 参考

> 版本: v1.6.0 | 基础地址: `http://127.0.0.1:3000`

## 认证

除公开端点外，所有端点需要 JWT Token（30 分钟有效）：

```bash
export TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"api_key": "<你的API Key>"}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")

# 后续请求: -H "Authorization: Bearer $TOKEN"
```

---

## 端点速查

| 分类 | 端点 | 方法 | 认证 | 需解锁 |
|------|------|------|------|--------|
| 系统 | `/api/v1/health` | GET | 否 | — |
| 系统 | `/api/v1/status` | GET | 否 | — |
| 系统 | `/api/v1/metrics` | GET | JWT | — |
| 系统 | `/api/v1/stats` | GET | JWT | — |
| 认证 | `/api/v1/init` | POST | 否 | — |
| 认证 | `/api/v1/auth/login` | POST | 否 | — |
| 认证 | `/api/v1/auth/rotate-api-key` | POST | JWT | — |
| 认证 | `/api/v1/auth/rotate-key` | POST | JWT | — |
| 会话 | `/api/v1/unlock` | POST | JWT | — |
| 会话 | `/api/v1/lock` | POST | JWT | — |
| 加解密 | `/api/v1/encrypt` | POST | JWT | 是 |
| 加解密 | `/api/v1/decrypt` | POST | JWT | 是 |
| 加解密 | `/api/v1/encrypt-stream` | POST | JWT | 是 |
| 加解密 | `/api/v1/decrypt-stream` | POST | JWT | 是 |
| 文件 | `/api/v1/files` | GET | JWT | — |
| 文件 | `/api/v1/files/get` | GET | JWT | — |
| 文件 | `/api/v1/files/download` | GET | JWT | 是 |
| 文件 | `/api/v1/files/delete` | DELETE | JWT | 是 |
| 分享 | `/api/v1/share` | POST | JWT | 是 |
| 分享 | `/api/v1/share/download` | GET | 否 | 是 |
| 分享 | `/api/v1/share/delete` | DELETE | JWT | — |
| 配置 | `/api/v1/config` | GET | JWT | — |
| 配置 | `/api/v1/config` | PUT | JWT | — |
| 备份 | `/api/v1/backup` | POST | JWT | 是 |
| 备份 | `/api/v1/recover` | POST | JWT | — |

---

## 系统信息

### GET /api/v1/health
无需认证。返回 `ok`。

```bash
curl http://127.0.0.1:3000/api/v1/health
```

### GET /api/v1/status
无需认证。返回服务初始化/锁定状态和版本号。

```bash
curl -s http://127.0.0.1:3000/api/v1/status | python3 -m json.tool
```
```json
{ "initialized": true, "locked": true, "version": "1.6.0" }
```

### GET /api/v1/metrics
需要认证。Prometheus 格式指标，可用于 Grafana 监控。

```bash
curl -s http://127.0.0.1:3000/api/v1/metrics -H "Authorization: Bearer $TOKEN"
```
```
leolock_uptime_seconds 1234
leolock_service_locked 0
leolock_requests_total{path="/api/v1/encrypt"} 42
```

### GET /api/v1/stats
需要认证。扫描目录下 `.leo` 文件的聚合统计。

```bash
curl -s "http://127.0.0.1:3000/api/v1/stats?path=/data" -H "Authorization: Bearer $TOKEN"
```
```json
{ "total_files": 37, "encrypted_files": 7, "total_encrypted_size": 780,
  "decryptable_count": 5, "versions": { "v4": 3, "v3": 2 } }
```

---

## 认证与会话

### POST /api/v1/init
无需认证。首次使用时设置密码，生成密钥和 API Key。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/init \
  -H 'Content-Type: application/json' -d '{"password": "你的密码"}'
```
```json
{ "status": "initialized", "api_key": "<仅显示一次>" }
```

### POST /api/v1/auth/login
无需认证。API Key 换取 JWT Token（30 分钟有效）。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' -d '{"api_key": "..."}'
```
```json
{ "token": "eyJ...", "expires_in": 1800, "token_type": "Bearer" }
```

### POST /api/v1/auth/rotate-api-key
需要认证。API Key 泄漏时生成新 Key，旧 Key 立即失效。需 JWT + 密码双重验证。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/auth/rotate-api-key \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}'
```
```json
{ "status": "rotated", "api_key": "<新的API Key>" }
```

### POST /api/v1/auth/rotate-key
需要认证。生成新盐值 + 主密钥。可选 `re_encrypt_path` 批量重加密已有文件。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/auth/rotate-key \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码", "re_encrypt_path": "/data"}'
```
```json
{ "status": "rotated", "re_encrypted": 3, "re_encrypt_errors": 0 }
```

### POST /api/v1/unlock
需要认证。密码派生 AES 密钥驻留内存。每 IP 每分钟最多 5 次，超限返回 429。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/unlock \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}'
```
```json
{ "status": "unlocked", "message": "🔓 服务已解锁" }
```

### POST /api/v1/lock
需要认证。立即擦除内存中的密钥。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/lock -H "Authorization: Bearer $TOKEN"
```
```json
{ "status": "locked", "message": "🔒 服务已锁定" }
```

---

## 加解密

### POST /api/v1/encrypt
multipart 上传 → V4 加密 → 返回 `.leo` 文件。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/encrypt \
  -H "Authorization: Bearer $TOKEN" -F "file=@document.pdf" -o document.leo
```

### POST /api/v1/decrypt
multipart 上传 `.leo` → 返回解密文件。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/decrypt \
  -H "Authorization: Bearer $TOKEN" -F "file=@document.leo" -o document.pdf
```

### POST /api/v1/encrypt-stream
原始二进制 body + `X-Filename` 头，无 MIME 解析开销，适合脚本。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/encrypt-stream \
  -H "Authorization: Bearer $TOKEN" -H "X-Filename: doc.pdf" \
  --data-binary @doc.pdf -o doc.leo
```

### POST /api/v1/decrypt-stream
原始 V4 加密二进制 body → 解密返回。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/decrypt-stream \
  -H "Authorization: Bearer $TOKEN" --data-binary @doc.leo -o doc.pdf
```

---

## 文件管理

### GET /api/v1/files
列出加密文件（分页/排序）。锁定时原文件名显示 `[需要密钥查看]`。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files?path=/data&sort=size_desc&page=1&per_page=50" \
  -H "Authorization: Bearer $TOKEN"
```
| 参数 | 必填 | 默认 | 说明 |
|------|------|------|------|
| `path` | 是 | — | 扫描目录 |
| `page` / `per_page` | 否 | 1/50 | 分页 |
| `sort` | 否 | — | `size_asc\|size_desc\|name_asc\|name_desc` |

```json
{ "items": [{ "id": "L3RtcC9h...", "version": 4, "encrypted_size": 108,
  "original_name": "report.pdf", "decryptable": true }], "total": 11, "page": 1 }
```

### GET /api/v1/files/get
单个文件详情。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files/get?id=L3RtcC9h..." \
  -H "Authorization: Bearer $TOKEN"
```

### GET /api/v1/files/download
原地解密下载。

```bash
curl -s "http://127.0.0.1:3000/api/v1/files/download?id=L3RtcC9h..." \
  -H "Authorization: Bearer $TOKEN" -o restored.pdf
```

### DELETE /api/v1/files/delete
安全删除加密文件。

```bash
curl -s -X DELETE "http://127.0.0.1:3000/api/v1/files/delete?id=L3RtcC9h..." \
  -H "Authorization: Bearer $TOKEN"
```

---

## 分享

### POST /api/v1/share
创建限时/限次/密码保护解密链接。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/share \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"file_id": "L3RtcC9h...", "max_downloads": 3, "expires_in": 3600, "password": "s3cret"}'
```
| 参数 | 必填 | 默认 | 说明 |
|------|------|------|------|
| `file_id` | 是 | — | 文件 ID |
| `expires_in` | 否 | 3600 | 过期秒数 |
| `max_downloads` | 否 | 1 | 最大下载次数 |
| `password` | 否 | — | 分享密码 |

```json
{ "token": "BCtc...", "url": "http://127.0.0.1:3000/api/v1/share/download?token=BCtc...",
  "expires_at": "2026-05-25T12:00:00Z", "max_downloads": 3 }
```

### GET /api/v1/share/download
**公开端点。** 无需 JWT，通过分享链接下载解密文件。

```bash
curl "http://127.0.0.1:3000/api/v1/share/download?token=BCtc...&password=s3cret" -o doc.pdf
```

### DELETE /api/v1/share/delete
撤销分享链接。

```bash
curl -s -X DELETE "http://127.0.0.1:3000/api/v1/share/delete?token=BCtc..." \
  -H "Authorization: Bearer $TOKEN"
```

---

## 配置管理

### GET /api/v1/config
需要认证。返回当前配置，敏感字段（salt、api_key_hash、jwt_secret）脱敏显示为 `***`。

```bash
curl -s http://127.0.0.1:3000/api/v1/config -H "Authorization: Bearer $TOKEN"
```
```json
{ "program": { "forbidden_paths": [...], "max_file_size": 10737418240, ... },
  "core": { "salt": "***", "argon2_m_cost": 19456, ... },
  "auth": { "api_key_hash": "***", "jwt_secret": "***" },
  "api": { "bind_address": "127.0.0.1", "port": 3000 } }
```

### PUT /api/v1/config
需要认证。只能更新 `program` 和 `api` 段。部分修改需重启服务。

```bash
curl -s -X PUT http://127.0.0.1:3000/api/v1/config \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"program": {"preserve_original_filename": true}, "api": {"port": 8443}}'
```
```json
{ "status": "updated", "message": "✅ 配置已更新" }
```

---

## 备份恢复

### POST /api/v1/backup
需要认证 + 已解锁。生成加密密钥备份文件。可反复调用。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/backup \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}' -o leolock_backup.enc
```

### POST /api/v1/recover
需要认证。上传备份文件 + 密码，恢复主密钥并即时生效。

```bash
curl -s -X POST http://127.0.0.1:3000/api/v1/recover \
  -H "Authorization: Bearer $TOKEN" \
  -F "backup=@leolock_backup.enc" -F "password=创建备份时的密码"
```
```json
{ "status": "recovered", "message": "✅ 密钥已从备份恢复，服务已解锁" }
```

---

## 完整调用流程

```bash
# 1. 登录
export TOKEN=$(curl -s -X POST http://127.0.0.1:3000/api/v1/auth/login \
  -H 'Content-Type: application/json' -d '{"api_key": "..."}' \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")

# 2. 解锁
curl -s -X POST http://127.0.0.1:3000/api/v1/unlock \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"password": "你的密码"}'

# 3. 加密
curl -s -X POST http://127.0.0.1:3000/api/v1/encrypt-stream \
  -H "Authorization: Bearer $TOKEN" -H "X-Filename: secret.pdf" \
  --data-binary @secret.pdf -o secret.leo

# 4. 查看文件列表
curl -s "http://127.0.0.1:3000/api/v1/files?path=." \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool

# 5. 创建分享链接
curl -s -X POST http://127.0.0.1:3000/api/v1/share \
  -H 'Content-Type: application/json' -H "Authorization: Bearer $TOKEN" \
  -d '{"file_id": "...", "max_downloads": 5, "expires_in": 7200}'

# 6. 锁定
curl -s -X POST http://127.0.0.1:3000/api/v1/lock -H "Authorization: Bearer $TOKEN"
```

---

## 错误码

| 状态码 | 含义 | 触发场景 |
|--------|------|----------|
| 200 | 成功 | — |
| 400 | 请求错误 | 密码太弱、文件解析失败、解密密钥不正确、路径不存在 |
| 401 | 未认证 | Token 缺失、无效或过期 |
| 404 | 未找到 | 路由不存在 |
| 412 | 前置条件不满足 | 服务未初始化就调用 unlock |
| 423 | 已锁定 | locked 状态下调用加解密/下载/删除 |
| 429 | 请求过多 | unlock 速率限制（每 IP 5次/分钟） |
| 500 | 内部错误 | 异常（详情输出 stderr，客户端只看到通用消息） |

## 安全说明

- **默认仅监听 127.0.0.1**，远程访问需配置反向代理 + TLS
- **密码不落盘**：unlock 后密码立即 zeroize，密钥驻留内存
- **重启自动锁定**：服务重启后密钥丢失
- **API Key 存储为 Argon2id 哈希**：与密码同等安全级别
- **加密格式**：AES-256-GCM V4 + AAD 元数据保护 + 文件名加密 + Argon2id 参数存储
