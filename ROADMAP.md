# LeoLock 项目路线图 (ROADMAP)

LeoLock 致力于成为 Linux/Unix 环境下最安全、最易用、且高性能的命令行加密工具，并提供 HTTP API 实现远程加解密服务。

## 核心愿景
- **绝对安全**: 内存零残留，算法前沿，元数据防篡改。
- **极致性能**: 超大文件流式处理，多核并行，API 内存直通（零磁盘中转）。
- **无感体验**: 直观的 CLI 交互，完善的 Shell 集成，REST API 远程调用。

---

## 已完成

### 阶段 1: 核心加密 (v1.0.0 → v1.1.0) — ✅

- AES-256-GCM 流式加解密（V3 格式：1MB chunk + AAD 元数据保护）
- Argon2id 密码派生 (m=19456, t=2, p=1)
- 文件名加密 + 原子化文件操作 (.tmp + rename)
- Zeroize 全链路内存擦除，危险路径保护
- 密钥生成/保存/备份/恢复

### 阶段 2: 并行化与交互增强 (v1.2.0) — ✅

- rayon 多线程并行加密目录
- 元数据 padding 防文件名长度泄露
- 密码来源：交互式 / 环境变量 / keyring / stdin
- 密码强度实时评估，indicatif 进度条
- `list` 命令，shell 补全脚本生成

### 阶段 3: API 服务基础 (v1.3.0) — ✅

- axum HTTP 服务（默认 bind 127.0.0.1:3000）
- Lock/Unlock 模式：密钥仅驻留内存，重启自动锁定
- JWT 鉴权（API Key → 短期 Token）
- 内存直通加解密（multipart → 内存 V3 加解密 → 返回，无临时文件）
- API 初始化（`POST /api/v1/init`），Config 缓存到 AppState
- 请求体大小限制 (2GB) + 并发控制 (8)

**已实现端点:**

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/api/v1/health` | 健康检查 |
| GET | `/api/v1/status` | 服务状态（是否初始化/锁定） |
| POST | `/api/v1/auth/login` | API Key → JWT Token |
| POST | `/api/v1/init` | 初始化（首次设置密码、生成密钥） |
| POST | `/api/v1/unlock` | 密码解锁（派生密钥驻内存） |
| POST | `/api/v1/lock` | 锁定（擦除内存密钥） |
| POST | `/api/v1/encrypt` | multipart 上传 → 加密 → 返回 .leo |
| POST | `/api/v1/decrypt` | multipart 上传 → 解密 → 返回原文 |
| GET | `/api/v1/files` | 列出加密文件（分页/排序） |
| GET | `/api/v1/files/get` | 单个文件详情 |
| GET | `/api/v1/files/download` | 原地解密下载 |
| DELETE | `/api/v1/files/delete` | 安全删除加密文件 |

---

## 当前重点：API 完善 (v1.4.0)

### 流式加解密

- [ ] `POST /api/v1/encrypt-stream` — 分块上传 → 流式加密 → 流式返回
- [ ] `POST /api/v1/decrypt-stream` — 分块上传 → 流式解密 → 流式返回

> 当前 encrypt/decrypt 端点将整个文件缓冲在内存中。流式端点使用
> `CryptoManager::encrypt_stream` 逐 chunk 处理，支持 GB 级文件。

### 安全加固

- [ ] 请求日志中间件（不含密码/密钥/文件名等敏感字段）
- [ ] unlock 端点速率限制（防暴力穷举密码）
- [ ] 错误响应脱敏（不泄露内部路径/栈信息）
- [ ] TLS 支持（远程访问场景）

---

## 阶段 4: 配置灵活化 (v1.5.0)

- [ ] **动态 Argon2id 参数**: 允许用户在 `config.toml` 自定义内存/迭代参数，参数随文件头存储确保向后兼容
- [ ] **多格式 list 输出**: `leolock list --format json|csv`，API 直接返回结构化 JSON
- [ ] `GET /api/v1/config` / `PUT /api/v1/config` — 通过 API 读写配置

---

## 阶段 5: 高级特性 (v1.6.0)

- [ ] **临时分享链接**: `POST /api/v1/share` 创建一次性/限时解密链接，支持密码保护和下载次数限制
- [ ] **密钥轮换**: `POST /api/v1/rotate-key` — 重新生成主密钥，可选批量重加密已有文件
- [ ] `POST /api/v1/backup` / `POST /api/v1/recover` — 通过 API 创建/恢复密钥备份
- [ ] `GET /api/v1/stats` — 统计信息（文件数/总大小/最近操作）

---

## 阶段 6: 平台扩展 (v2.0.0)

- [ ] **Web 管理面板**: 纯静态前端，拖拽上传加密、文件列表管理、分享链接管理
- [ ] **FUSE 挂载（实验性）**: 将加密目录挂载为虚拟文件系统，实时解密读取
- [ ] **C FFI 绑定**: 提供 C ABI，支持 Python/Node.js 等语言调用核心加密逻辑
- [ ] **硬件密钥支持**: 集成 PKCS#11，支持 YubiKey 等硬件令牌存储主密钥
- [ ] **云端密钥库**: 支持 HashiCorp Vault / AWS KMS 获取和备份加密盐值与配置

---

## 架构原则

1. **Fail-Safe (故障安全)**: 任何文件操作假设会中途崩溃，数据完整性优先于删除便利性
2. **No Secret Left Behind (零秘密残留)**: 内存敏感信息生命周期尽可能短，使用后立即 zeroize
3. **Explicit over Implicit (显式优于隐式)**: 加密模式转换必须在日志中有清晰提示
4. **Forward Compatibility (向前兼容)**: 文件头设计预留版本扩展空间
5. **Memory over Disk (内存优先)**: API 加解密路径避免磁盘中转，直接操作内存缓冲区

---

## 版本历史

| 版本 | 要点 |
|------|------|
| v1.0.0 | 初始版本，AES-256-GCM + 基础配置 |
| v1.0.3 | 文件名加密 (V2 格式)，list 命令 |
| v1.1.0 | 流式加密 (V3)，AAD，zeroize，原子操作 |
| v1.2.0 | 多线程并行，进度条，keyring/stdin/env 密码 |
| v1.3.0 | HTTP API 服务，lock/unlock 模式，JWT 鉴权 |

---

**最后更新:** 2026-05-24
