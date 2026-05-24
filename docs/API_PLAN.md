# LeoLock API 计划书

> 版本: v1.0 | 日期: 2026-05-24 | 作者: Jone Snow 🐺

---

## 一、项目现状分析

### 1.1 LeoLock 是什么

一个 Rust 编写的文件加密解密 CLI 工具，核心能力：

| 模块 | 文件 | 职责 |
|------|------|------|
| 加密引擎 | `crypto.rs` (647行) | AES-256-GCM 流式加解密，1MB 分块，AAD 元数据保护，V3 文件格式 |
| 文件操作 | `fileops.rs` (557行) | 单文件/目录递归处理，原子化替换，进度条，并行加密 |
| 密钥管理 | `keymgmt.rs` (270行) | 密钥生成/保存/加载，备份创建与恢复 |
| 密码管理 | `password.rs` (249行) | Argon2id 密码派生，keyring/env/stdin 多来源支持 |
| 配置管理 | `config.rs` (280行) | TOML 配置读/写，危险路径检查，XDG 目录兼容 |
| CLI 入口 | `main.rs` (560行) | clap 命令解析，7 个子命令 |
| 库入口 | `lib.rs` | 已做好 lib crate 导出，可直接被 API server 引用 |

### 1.2 已有架构优势

- ✅ **lib.rs 已分离** —— 加密逻辑已是独立 crate，API server 可以直接 `use leolock::*`
- ✅ **流式处理** —— 1MB chunk + 2MB buffer，天然适配 HTTP streaming
- ✅ **错误体系完整** —— `errors.rs` 有 ThisError 派生，可映射到 HTTP 状态码
- ✅ **零拷贝路径** —— `Zeroizing` + RAII 擦除，API 场景下安全
- ✅ **已有并行能力** —— rayon 多线程，批处理加密不虚

### 1.3 当前限制

- ❌ 纯 CLI，只能本地终端操作
- ❌ 没有网络接口，无法远程调用
- ❌ 没有 Web 管理面板
- ❌ 没有自动化脚本集成（只能用 expect 模拟交互输密码）

---

## 二、为什么要做 API？

| 场景 | 没有 API | 有了 API |
|------|---------|---------|
| **出差时加密家里文件** | SSH 上去手打命令 | `curl -X POST` 一行搞定 |
| **自动备份脚本** | expect 模拟交互，脆弱 | 脚本直接调 API，可靠 |
| **手机上传前加密** | 不可能 | Android App → API → 加密 → 存云盘 |
| **leochat 集成** | 无法复用 | 聊天室直接调加密接口 |
| **Web 管理面板** | 不存在 | 可视化拖拽加解密 |
| **分享加密文件** | 对方要装 leolock + 密码 | 生成一次性解密链接 |

一句话：**从"工具"变成"服务"**。

---

## 三、技术方案选型

### 3.1 方案对比

| 方案 | 优点 | 缺点 | 适合？ |
|------|------|------|--------|
| **HTTP REST (axum)** | 通用性最强，curl 可调，易做 Web UI | HTTP 开销，需序列化 | ⭐ 推荐 |
| **gRPC (tonic)** | 高性能流式，强类型 | curl 调不了，需 protobuf | 微服务场景 |
| **Unix Socket** | 零网络开销，本地极快 | 仅本机，不跨网络 | 补充方案 |
| **C FFI 绑定** | 原生性能 | 每种语言都要写 wrapper | Roadmap v2.0 |

### 3.2 推荐方案：axum + Unix Socket 双模式

```
              ┌──────────────┐
   curl ───►  │  HTTP:3000   │  ◄── Web Dashboard
   app  ───►  │  (axum)      │  ◄── 远程调用
              │              │
 本地脚本 ──►  │  UDS socket  │  ◄── 本地高性能
              └──────┬───────┘
                     │
              ┌──────▼───────┐
              │  leolock     │  ← lib.rs 直接复用
              │  crypto.rs   │
              │  fileops.rs  │
              └──────────────┘
```

**为什么选 axum？**
- Rust 生态最成熟的 Web 框架（tokio 官方出品）
- 天然支持 streaming body（大文件上传不爆内存）
- Tower 中间件生态（auth、限流、日志开箱即用）
- 配合 `tower-http` 的 `ServeDir` 可以直接托管 Web 前端

### 3.3 新增依赖（Cargo.toml）

```toml
axum = { version = "0.7", features = ["multipart"] }
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["cors", "auth", "limit"] }
serde = { version = "1.0", features = ["derive"] }        # 已有
serde_json = "1.0"                                         # 已有
uuid = { version = "1", features = ["v4"] }
jsonwebtoken = "9"           # JWT 鉴权
```

新增依赖很少，因为核心逻辑完全复用现有 crate。

---

## 四、API 端点设计

### 4.1 总览

```
健康检查
  GET  /api/v1/health

初始化与密钥
  POST /api/v1/init                    # 首次初始化
  GET  /api/v1/status                  # 是否已初始化
  POST /api/v1/backup                  # 创建密钥备份
  POST /api/v1/recover                 # 从备份恢复
  POST /api/v1/rotate-key              # 轮换密钥

加密/解密
  POST /api/v1/encrypt                 # 加密文件
  POST /api/v1/decrypt                 # 解密文件
  POST /api/v1/encrypt-stream          # 流式加密（分块上传）
  POST /api/v1/decrypt-stream          # 流式解密（分块下载）

文件管理
  GET  /api/v1/files                   # 列出加密文件
  GET  /api/v1/files/:id               # 获取单个文件信息
  GET  /api/v1/files/:id/download      # 下载解密后的文件
  DELETE /api/v1/files/:id             # 删除加密文件

分享（高级特性）
  POST /api/v1/share                   # 创建分享链接
  GET  /api/v1/share/:token            # 通过分享链接获取文件
  DELETE /api/v1/share/:token          # 撤销分享

配置管理
  GET  /api/v1/config                  # 获取配置
  PUT  /api/v1/config                  # 更新配置

系统管理
  GET  /api/v1/stats                   # 统计信息
  GET  /api/v1/metrics                 # Prometheus 指标
```

### 4.2 核心端点详细设计

#### POST /api/v1/encrypt

```
请求:
  POST /api/v1/encrypt
  Authorization: Bearer <token>
  Content-Type: multipart/form-data

  file: <binary>           # 要加密的文件
  mode: "full" | "fast"    # full=加密文件名，fast=仅内容
  keep_original: bool      # 是否保留原文件
  output_path: string?     # 可选：指定输出目录

响应 200:
  {
    "id": "a1b2c3d4",
    "filename": "a3f8e2.leo",
    "original_name": "secret.docx",
    "size": 1048576,
    "encrypted_size": 1048640,
    "mode": "full",
    "created_at": "2026-05-24T13:00:00Z"
  }

响应 401: 未授权
响应 413: 文件过大（超过 max_file_size）
```

#### POST /api/v1/decrypt

```
请求:
  POST /api/v1/decrypt
  Content-Type: multipart/form-data

  file: <binary>           # .leo 加密文件

响应 200:
  Content-Type: application/octet-stream
  Content-Disposition: attachment; filename="secret.docx"

  <解密后的二进制流>

响应 401: 密码错误或未授权
```

#### GET /api/v1/files

```
请求:
  GET /api/v1/files?path=/data/encrypted&sort=size_desc&page=1&per_page=50

响应 200:
  {
    "items": [
      {
        "id": "a1b2c3d4",
        "path": "/data/encrypted/report.leo",
        "version": 3,
        "encrypted_size": 256000,
        "original_name": "Q2 财报.xlsx",
        "created_at": "2026-05-20T08:00:00Z",
        "decryptable": true
      }
    ],
    "total": 42,
    "page": 1,
    "per_page": 50
  }
```

#### POST /api/v1/share

```
请求:
  POST /api/v1/share
  {
    "file_id": "a1b2c3d4",
    "expires_in": 3600,          # 1小时后过期
    "max_downloads": 3,          # 最多下载3次
    "password": "share-pass-123"  # 分享密码
  }

响应 200:
  {
    "token": "eyJhbGciOi...",
    "url": "http://localhost:3000/api/v1/share/eyJhbGciOi...",
    "expires_at": "2026-05-24T14:00:00Z",
    "max_downloads": 3
  }
```

---

## 五、安全设计

### 5.1 认证：JWT Token

```rust
// 初始化时生成 API key
POST /api/v1/init → 返回 {"api_key": "ll_xxxxxxxxxxxx"}

// 用 API key 换取短期 JWT (30分钟)
POST /api/v1/auth/login
Body: {"api_key": "ll_xxxxxxxxxxxx"}
→ {"token": "eyJ...", "expires_in": 1800}
```

### 5.2 敏感数据处理

| 数据 | 处理策略 |
|------|---------|
| 密码 | 仅在请求时接收，用完立即 `zeroize`，**不落盘不缓存不记日志** |
| 密钥 | 内存中保持 `Zeroizing<[u8;32]>`，server 重启后需重新 unlock |
| API Key | 存储为 Argon2id hash（同密码安全级别） |
| JWT | 短期有效 + 单次使用限制 |

### 5.3 网络安全

```toml
# 默认配置（安全优先）
bind_address = "127.0.0.1"   # 仅本地监听
port = 3000
enable_remote = false         # 需要手动打开
tls_cert = ""                 # 远程访问建议配 TLS
max_request_size = "10GB"     # 匹配现有 max_file_size
rate_limit = "100/min"        # 防暴力破解
```

### 5.4 服务状态：Unlock 模式

API 服务引入一个「锁定/解锁」状态：

```
          ┌─────────┐   POST /api/v1/unlock   ┌──────────┐
 启动 ──► │ LOCKED  │ ──────────────────────► │ UNLOCKED │
          │ (无密钥) │    {password: "..."}     │ (可加解密) │
          └─────────┘                          └──────────┘
                                                    │
                                          POST /api/v1/lock
                                                    │
                                                    ▼
                                              密钥从内存擦除
```

- 服务重启后自动回到 LOCKED 状态
- 必须调用 `/unlock` 输入密码才能进行加解密操作
- `/lock` 手动锁定，立即擦除内存中的密钥
- 这种设计确保：即使服务器被入侵，重启后攻击者也无法解密文件

---

## 六、项目结构

```
leolock/
├── Cargo.toml              # workspace
├── src/                    # 现有 CLI + lib（不改动）
│   ├── main.rs
│   ├── lib.rs
│   ├── crypto.rs
│   ├── fileops.rs
│   ├── keymgmt.rs
│   ├── password.rs
│   ├── config.rs
│   ├── errors.rs
│   └── utils.rs
│
├── server/                 # ✨ 新增：API 服务
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs         # tokio + axum 启动
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── health.rs
│       │   ├── auth.rs
│       │   ├── encrypt.rs
│       │   ├── decrypt.rs
│       │   ├── files.rs
│       │   ├── share.rs
│       │   └── config.rs
│       ├── middleware/
│       │   ├── mod.rs
│       │   ├── auth.rs     # JWT 验证
│       │   └── logging.rs  # 请求日志
│       ├── state.rs        # AppState (密钥持有者)
│       └── error.rs        # HTTP 错误映射
│
├── web/                    # ✨ 新增：Web 管理面板（可选，后续）
│   ├── index.html
│   ├── app.js
│   └── style.css
│
├── examples/
├── ROADMAP.md
└── README.md

Cargo.toml (workspace):
[workspace]
members = ["", "server"]
```

---

## 七、实施路线图

### Phase 1: 核心 API（2-3 天）

```
[ ] 创建 server/ crate，引入 axum + tokio
[ ] 实现 AppState（锁定/解锁状态管理）
[ ] POST /api/v1/unlock  ← 密码 → 派生密钥 → 驻内存
[ ] POST /api/v1/lock    ← 擦除密钥
[ ] GET  /api/v1/health
[ ] GET  /api/v1/status
[ ] POST /api/v1/encrypt  ← multipart 上传 → 加密 → 返回 .leo
[ ] POST /api/v1/decrypt  ← multipart 上传 → 解密 → 流式返回
```

**Phase 1 结束时**：curl 一行命令加解密，核心价值已交付。

### Phase 2: 认证与安全（1-2 天）

```
[ ] POST /api/v1/auth/login    ← API Key → JWT
[ ] JWT 中间件（自动续期？）
[ ] 密码 zeroize 全链路审计
[ ] rate limiting
[ ] 错误不泄露内部细节
[ ] 请求日志（不含敏感数据）
```

### Phase 3: 文件管理（1-2 天）

```
[ ] GET  /api/v1/files          ← 列出加密文件，排序/分页/过滤
[ ] GET  /api/v1/files/:id      ← 单个文件详情
[ ] GET  /api/v1/files/:id/download  ← 原地解密流式下载
[ ] DELETE /api/v1/files/:id    ← 删除
[ ] POST /api/v1/encrypt 增强   ← 支持 output_path 参数
```

### Phase 4: 高级特性（2-3 天）

```
[ ] POST /api/v1/init           ← 通过 API 初始化（生成 salt + 密钥）
[ ] POST /api/v1/backup
[ ] POST /api/v1/recover
[ ] POST /api/v1/rotate-key
[ ] POST /api/v1/share          ← 临时分享链接
[ ] GET  /api/v1/share/:token
[ ] GET  /api/v1/stats
[ ] 流式加密 API（chunked upload → chunked encrypt → stream back）
```

### Phase 5: Web 控制台（3-5 天，可选）

```
[ ] 纯静态前端（React/Vue/原生HTML）
[ ] 文件列表 + 拖拽上传 → 加密
[ ] 加密文件 → 点击下载解密
[ ] 分享链接管理
[ ] 仪表盘（文件数、总大小、最近活动）
```

---

## 八、达到的效果

### 8.1 直接效果

```
# 原来（CLI 手工操作）
$ leolock encrypt ~/Documents/report.xlsx
请输入密码: ********
🔒 开始加密...

# 有了 API 后
$ curl -X POST http://localhost:3000/api/v1/unlock \
    -d '{"password":"my-secret"}'

$ curl -X POST http://localhost:3000/api/v1/encrypt \
    -H "Authorization: Bearer $TOKEN" \
    -F "file=@report.xlsx" \
    -F "mode=full" \
    -o report.leo

# 一行密码都不用手打，脚本全自动
```

### 8.2 集成效果

| 集成对象 | 效果 |
|---------|------|
| **leochat** | 聊天室发送文件前自动加密，对方收到后调用解密 API |
| **cron 备份** | `tar czf - /data \| curl -F "file=@-" .../encrypt > backup.leo` |
| **Android app** | 拍照 → 调 API 加密 → 上传云盘，云盘只存密文 |
| **aliyundrive-fuse** | 挂载目录配合 API，自动加密同步到云盘 |
| **Web 管理** | 浏览器拖文件进去 → 加密 → 下载，零学习成本 |
| **CI/CD** | GitHub Actions 加密环境变量后存入仓库 |

### 8.3 安全提升

- **密钥不落盘**：密码只存活在 API 服务内存中，重启自动擦除
- **审计日志**：每次加解密操作留痕
- **分享控制**：精确到次数 + 时间的访问控制
- **远程无需密钥文件**：远程机器不需要存 `keys.toml`，调 API 即可

---

## 九、风险与注意事项

| 风险 | 缓解措施 |
|------|---------|
| **API 暴露公网** | 默认 bind 127.0.0.1，远程需要显式配置 + TLS |
| **大文件上传耗尽内存** | axum 的 Multipart + Stream 逐 chunk 处理 |
| **并发写同一文件** | 文件锁 or 拒绝重复操作 |
| **JWT 泄露** | 短期有效（30min）+ 单设备限制 |
| **密码在 HTTP body 中** | 生产环境必须 HTTPS / Unix Socket |
| **依赖膨胀** | axum + tokio 增加约 3MB 二进制，可接受 |

---

## 十、总结

**一句话**：LeoLock 的核心加密逻辑已经足够成熟，API 化本质上就是给现有的 `lib.rs` 套一层 HTTP 壳。

- 改动量小（不动现有代码，新增 `server/` crate）
- 安全设计先行（unlock/lock 模式 + JWT + zeroize）
- 渐进交付（Phase 1 就能用 curl 加解密）
- 为 leochat / 手机端 / cron 自动化铺平道路

下一步如果你认可这个方案，我可以直接从 Phase 1 开始写代码。
