# LeoLock 项目路线图 (ROADMAP)

LeoLock 是一个个人使用的命令行文件加密工具，聚焦安全、性能和易用性。

## 核心愿景
- **绝对安全**: 内存零残留，算法前沿，元数据防篡改。
- **极致性能**: 超大文件流式处理，多核并行。
- **无感体验**: 直观的 CLI 交互，完善的 Shell 集成。

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

### 阶段 3: 配置灵活化 — ✅

- 动态 Argon2id 参数: `config.toml` [core] 段自定义 m/t/p，V4 文件头存储参数确保向后兼容
- 多格式 list 输出: `leolock list --format json|simple|table`
- `leolock config set <key> <value>` — CLI 修改配置
- `leolock config add-forbidden <path>` / `remove-forbidden <path>`
- Config 结构重组: `[program]/[core]` 两段，旧格式自动迁移

---

## 下一步: 体验增强 (v2.0.0)

- [ ] **密码修改**: `leolock change-password` — 重新派生密钥，批量重加密已有文件
- [ ] **密钥轮换**: `leolock rotate-key` — 重新生成主密钥，可选批量重加密
- [ ] **增量加密**: 只加密变更的文件，大幅提升目录重加密速度
- [ ] **并行解密**: 目录解密也支持 rayon 并行加速
- [ ] **加密文件完整性校验**: 不依赖密码的 `.leo` 文件结构校验
- [ ] **Shell 补全自动安装**: `leolock completions install <shell>` 一步到位

---

## 架构原则

1. **Fail-Safe (故障安全)**: 任何文件操作假设会中途崩溃，数据完整性优先于删除便利性
2. **No Secret Left Behind (零秘密残留)**: 内存敏感信息生命周期尽可能短，使用后立即 zeroize
3. **Explicit over Implicit (显式优于隐式)**: 加密模式转换必须在日志中有清晰提示
4. **Forward Compatibility (向前兼容)**: 文件头设计预留版本扩展空间
5. **Keep It Simple**: 单一二进制，单一职责——文件加密。不引入服务端复杂度。

---

## 版本历史

| 版本 | 要点 |
|------|------|
| v1.0.0 | 初始版本，AES-256-GCM + 基础配置 |
| v1.0.3 | 文件名加密 (V2 格式)，list 命令 |
| v1.1.0 | 流式加密 (V3)，AAD，zeroize，原子操作 |
| v1.1.2 | 多线程并行，进度条，keyring/stdin/env 密码 |
| v1.2.0 | V4 格式，动态 Argon2id，Config 重组，多格式 list，CLI 配置管理 |

---

**最后更新:** 2026-06-20
