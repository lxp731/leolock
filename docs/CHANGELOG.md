## 📝 版本历史

### 版本 1.5.0 (当前)
- **动态 Argon2id 参数**: `[core]` 段可自定义 m_cost/t_cost/p_cost，V4 文件头存储参数。
- **V4 文件格式**: 文件头追加 12 字节 Argon2id 参数，AAD 同步保护。V1-V3 文件自动使用默认参数。
- **多格式 list 输出**: `leolock list --format json|simple|table` 三种格式。
- **Config API**: `GET /api/v1/config` 查看配置（敏感字段脱敏），`PUT /api/v1/config` 更新。
- **CLI 配置管理**: `leolock config set <key> <value>` 修改配置项，`add-forbidden` / `remove-forbidden` 管理危险路径列表。

### 版本 1.4.0
- **流式加解密端点**: `encrypt-stream` / `decrypt-stream`，接收原始二进制 body，无 MIME 解析开销。
- **文件管理 API**: 列出/查看/下载/删除加密文件，支持分页和排序。
- **API Key 轮换**: `POST /api/v1/auth/rotate-api-key`，密码验证后即时更换 Key，无需重启服务。
- **解锁速率限制**: 每 IP 每分钟最多 5 次尝试，超限返回 429。
- **请求日志中间件**: 记录方法/路径/状态码/耗时，不含敏感数据。
- **错误响应脱敏**: 内部错误只返回通用消息，详情输出至 stderr。
- **Config 结构重组**: 拆分为 `[program]/[core]/[auth]/[server]` 四个 TOML 段，旧格式自动迁移。
- **服务端口可配**: 监听地址和端口写入 `[server]` 段，无需修改代码。

### 版本 1.3.0
- **HTTP API 服务**: 新增 `leolock-server` 子 crate，提供 REST API。
- **Lock/Unlock 安全模式**: 密钥仅驻留内存，服务重启自动锁定，手动 lock 立即 zeroize 擦除。
- **JWT 鉴权**: API Key (Argon2id 哈希存储) → 短期 JWT Token (30 分钟)。
- **API 端点**: health / status / init / login / unlock / lock / encrypt / decrypt。
- **文件管理 API**: 列出加密文件 (分页/排序)、查看详情、原地解密下载、安全删除。
- **内存直通优化**: API 加解密不写临时文件，直接内存 Cursor → encrypt/decrypt_stream。
- **Config 缓存**: AppState 启动时一次性加载配置，消除每个请求的文件 I/O。
- **代码精简**: fileops.rs 去除 6 个重复方法 (557→237 行)，main.rs 提取公共密码读取逻辑。
- **安全加固**: 请求体大小限制 (2GB)、并发控制 (8)、unlock 密码立即 zeroize。

### 版本 1.2.0
- **多线程并行加密**: 使用 `rayon` 库实现目录递归时的并行处理，大幅提升批量加密效率。
- **交互增强**: 引入 `indicatif` 进度条与密码强度实时评估功能。
- **元数据填充**: 加密文件名对齐填充，消除文件名长度泄露信息的潜在风险。
- **高级密码策略**: 支持环境变量 / 系统钥匙串 / 标准输入加载密码。

### 版本 1.1.0
- **性能质跃**: 引入流式加密重构，引入 **1MB 分块** 和 **原地加密 (In-place)** 技术。
- **极致速度**: 实测 3GB 文件加密速度从 440s 提升至 14s (~214 MB/s)。
- **内存安全**: 全面集成 `zeroize` 确保敏感数据在内存中无残留，引入 `Zeroizing<T>` 保护。
- **完整性增强**: 升级 **V3** 文件格式，引入 **AAD (附加认证数据)** 保护头部元数据。
- **鲁棒性增强**: 实现原子化文件写入（先写 .tmp 后 rename），防止断电损坏。

### 版本 1.0.3
- **简化密码管理**: 移除单独的密码哈希文件，密码直接派生密钥。
- **文件列表功能**: 新增 `leolock list` 命令，支持排序和原文件名显示。
- **运行时安全检查**: 自动检测配置文件权限并警告。

### 版本 1.0.0
- **基础功能**: 实现 AES-256-GCM 核心加密逻辑。
- **配置系统**: 支持 TOML 配置文件。
- **备份恢复**: 支持密钥备份。
