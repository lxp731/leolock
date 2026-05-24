use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::RwLock;
use std::time::Instant;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

const UNLOCK_RATE_LIMIT: u32 = 5;
const UNLOCK_RATE_WINDOW: u64 = 60;

/// API 服务的全局状态
pub struct AppState {
    pub encryption_key: RwLock<Option<Zeroizing<[u8; 32]>>>,
    pub jwt_secret: Option<String>,
    pub salt: Option<String>,
    api_key_hash: RwLock<Option<String>>,
    pub is_initialized: bool,
    pub argon2_m: u32,
    pub argon2_t: u32,
    pub argon2_p: u32,

    unlock_attempts: Mutex<HashMap<IpAddr, (u32, Instant)>>,
}

impl AppState {
    pub fn new(
        jwt_secret: Option<String>,
        salt: Option<String>,
        api_key_hash: Option<String>,
        is_initialized: bool,
        argon2_m: u32,
        argon2_t: u32,
        argon2_p: u32,
    ) -> Self {
        Self {
            encryption_key: RwLock::new(None),
            jwt_secret,
            salt,
            api_key_hash: RwLock::new(api_key_hash),
            is_initialized,
            argon2_m,
            argon2_t,
            argon2_p,
            unlock_attempts: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.encryption_key.read().unwrap().is_some()
    }

    pub fn unlock(&self, key: [u8; 32]) {
        let mut guard = self.encryption_key.write().unwrap();
        *guard = Some(Zeroizing::new(key));
    }

    pub fn lock(&self) {
        let mut guard = self.encryption_key.write().unwrap();
        *guard = None;
    }

    pub fn get_key(&self) -> Option<[u8; 32]> {
        let guard = self.encryption_key.read().unwrap();
        guard.as_ref().map(|z| **z)
    }

    pub fn verify_api_key(&self, key: &str) -> bool {
        let guard = self.api_key_hash.read().unwrap();
        match guard.as_ref() {
            Some(hash) => leolock::password::PasswordManager::verify_api_key(key, hash),
            None => false,
        }
    }

    pub fn has_api_key(&self) -> bool {
        self.api_key_hash.read().unwrap().is_some()
    }

    pub fn update_api_key_hash(&self, hash: String) {
        let mut guard = self.api_key_hash.write().unwrap();
        *guard = Some(hash);
    }

    pub async fn check_unlock_rate(&self, ip: IpAddr) -> bool {
        let mut guard = self.unlock_attempts.lock().await;
        let now = Instant::now();
        let entry = guard.entry(ip).or_insert_with(|| (0, now));
        if now.duration_since(entry.1).as_secs() >= UNLOCK_RATE_WINDOW {
            *entry = (1, now);
            return true;
        }
        if entry.0 < UNLOCK_RATE_LIMIT {
            entry.0 += 1;
            true
        } else {
            false
        }
    }
}
