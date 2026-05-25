# LeoLock 项目路线图 (ROADMAP)

LeoLock 致力于成为 Linux/Unix 环境下最安全、最易用、且高性能的命令行加密工具，并提供 HTTP API 实现远程加解密服务。

## 核心愿景
- **绝对安全**: 内存零残留，算法前沿，元数据防篡改。
- **极致性能**: 超大文件流式处理，多核并行，API 内存直通（零磁盘中转）。
- **无感体验**: 直观的 CLI 交互，完善的 Shell 集成，REST API 远程调用。

---

## 已完成

### 阶段 1: 核心加密 — ✅

- AES-256-GCM 流式加解密（V3 格式：1MB chunk + AAD 元数据保护）
- Argon2id 密码派生 (m=19456, t=2, p=1)
- 文件名加密 + 原子化文件操作 (.tmp + rename)
- Zeroize 全链路内存擦除，危险路径保护
- 密钥生成/保存/备份/恢复

### 阶段 2: 并行化与交互增强 — ✅

- rayon 多线程并行加密目录
- 元数据 padding 防文件名长度泄露
- 密码来源：交互式 / 环境变量 / keyring / stdin
- 密码强度实时评估，indicatif 进度条
- `list` 命令，shell 补全脚本生成

### 阶段 3: API 服务基础 — ✅

- axum HTTP 服务（默认 bind 127.0.0.1:3000）
- Lock/Unlock 模式：密钥仅驻留内存，重启自动锁定
- JWT 鉴权（API Key → 短期 Token）
- 内存直通加解密（multipart → 内存 V3 加解密 → 返回，无临时文件）
- API 初始化（`POST /api/v1/init`），Config 缓存到 AppState
- 请求体大小限制 (2GB) + 并发控制 (8)
- 流式加解密：encrypt-stream / decrypt-stream（原始二进制 body）
- unlock 速率限制（每 IP 5次/分钟，429）、请求日志中间件、错误脱敏

---

## 阶段 4: 配置灵活化 — ✅

- [x] **动态 Argon2id 参数**: `config.toml` [core] 段自定义 m/t/p，V4 文件头存储参数确保向后兼容
- [x] **多格式 list 输出**: `leolock list --format json|simple|table`
- [x] `GET /api/v1/config` / `PUT /api/v1/config` — 通过 API 读写配置（敏感字段脱敏）
- [x] `leolock config set <key> <value>` — CLI 修改配置
- [x] `leolock config add-forbidden <path>` / `remove-forbidden <path>`

---

## 阶段 5: 高级特性 — ✅

- [x] **临时分享链接**: `POST /api/v1/share` 创建限时/限次/密码保护解密链接，`GET /api/v1/share/download` 公开下载
- [x] **密钥轮换**: `POST /api/v1/auth/rotate-key` — 重新生成主密钥，可选批量重加密已有文件
- [x] `POST /api/v1/backup` / `POST /api/v1/recover` — 通过 API 创建/恢复密钥备份
- [x] `GET /api/v1/stats` — 统计信息（文件数/总大小/版本分布/可解密数）
- [x] `GET /api/v1/metrics` — Prometheus 格式服务指标

---

## 下一步: 平台扩展 (v2.0.0)

- [ ] **Web 管理面板**: 纯静态前端（React/Vue/原生HTML），直接调用 API
  - 文件列表 + 拖拽上传加密
  - 加密文件 → 点击下载解密
  - 分享链接创建与管理
  - 仪表盘（文件数、总大小、最近活动）
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
| v1.1.2 | 多线程并行，进度条，keyring/stdin/env 密码 |
| v1.2.0 | HTTP API 服务，文件管理，分享链接，动态 Argon2id，V4 格式 |

---

**最后更新:** 2026-05-25
