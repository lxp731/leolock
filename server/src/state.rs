use std::sync::RwLock;
use zeroize::Zeroizing;

/// API 服务的全局状态
///
/// 核心设计：
/// - lock/unlock 模式控制加密密钥生命周期
/// - 启动时从配置文件加载并缓存 salt/api_key_hash/initialized
/// - 服务重启自动回到 LOCKED
pub struct AppState {
    /// AES-256 加密密钥（32 字节）
    /// None = 已锁定，不能加解密
    /// Some = 已解锁，可以加解密
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,

    /// JWT 签名密钥（用于签发和验证 Token）
    pub jwt_secret: Option<String>,

    /// 盐值（base64 编码，从配置缓存，用于 unlock 时派生密钥）
    pub salt: Option<String>,

    /// API Key 的 Argon2id 哈希（从配置缓存，用于 login 验证）
    pub api_key_hash: Option<String>,

    /// 是否已初始化
    pub is_initialized: bool,
}

impl AppState {
    /// 创建新的应用状态（从配置加载并缓存关键字段）
    pub fn new(
        jwt_secret: Option<String>,
        salt: Option<String>,
        api_key_hash: Option<String>,
        is_initialized: bool,
    ) -> Self {
        Self {
            encryption_key: RwLock::new(None),
            jwt_secret,
            salt,
            api_key_hash,
            is_initialized,
        }
    }

    /// 检查服务是否已解锁
    pub fn is_unlocked(&self) -> bool {
        self.encryption_key.read().unwrap().is_some()
    }

    /// 设置加密密钥（解锁）
    pub fn unlock(&self, key: [u8; 32]) {
        let mut guard = self.encryption_key.write().unwrap();
        *guard = Some(Zeroizing::new(key));
    }

    /// 清除加密密钥（锁定）
    /// 旧密钥的 Zeroizing wrapper 在 Drop 时自动归零
    pub fn lock(&self) {
        let mut guard = self.encryption_key.write().unwrap();
        *guard = None;
    }

    /// 获取密钥的副本（用于加解密操作）
    pub fn get_key(&self) -> Option<[u8; 32]> {
        let guard = self.encryption_key.read().unwrap();
        guard.as_ref().map(|z| **z)
    }

    /// 验证 API Key
    pub fn verify_api_key(&self, key: &str) -> bool {
        match &self.api_key_hash {
            Some(hash) => leolock::password::PasswordManager::verify_api_key(key, hash),
            None => false,
        }
    }

    /// 检查 API Key 是否已配置
    pub fn has_api_key(&self) -> bool {
        self.api_key_hash.is_some()
    }
}
