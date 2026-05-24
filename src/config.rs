use crate::errors::{BjtError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ─── 程序基础设置 ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    /// 危险路径列表（禁止处理的系统目录）
    #[serde(default = "default_forbidden_paths")]
    pub forbidden_paths: Vec<String>,

    /// 最大文件大小（字节），0表示无限制
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// 默认加密文件后缀
    #[serde(default = "default_extension")]
    pub default_extension: String,

    /// 密钥文件位置
    #[serde(default = "default_key_file_path")]
    pub key_file_path: String,

    /// 是否保留原文件名（false=加密文件名，true=保留文件名）
    #[serde(default = "default_false")]
    pub preserve_original_filename: bool,

    /// 是否显示进度条
    #[serde(default = "default_true")]
    pub show_progress: bool,

    /// 加密文件格式版本
    #[serde(default = "default_file_version")]
    pub file_format_version: u8,
}

impl Default for ProgramConfig {
    fn default() -> Self {
        Self {
            forbidden_paths: default_forbidden_paths(),
            max_file_size: default_max_file_size(),
            default_extension: default_extension(),
            key_file_path: default_key_file_path(),
            preserve_original_filename: false,
            show_progress: true,
            file_format_version: 2,
        }
    }
}

// ─── 加密核心数据 ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// 盐值（base64编码，用于从密码派生密钥）
    /// None = 未初始化
    #[serde(default)]
    pub salt: Option<String>,

    /// Argon2id 内存成本 (KB)
    #[serde(default = "default_argon2_m")]
    pub argon2_m_cost: u32,

    /// Argon2id 迭代次数
    #[serde(default = "default_argon2_t")]
    pub argon2_t_cost: u32,

    /// Argon2id 并行度
    #[serde(default = "default_argon2_p")]
    pub argon2_p_cost: u32,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            salt: None,
            argon2_m_cost: default_argon2_m(),
            argon2_t_cost: default_argon2_t(),
            argon2_p_cost: default_argon2_p(),
        }
    }
}

fn default_argon2_m() -> u32 {
    19456
}
fn default_argon2_t() -> u32 {
    2
}
fn default_argon2_p() -> u32 {
    1
}

// ─── API 鉴权 ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// API Key 的 Argon2id 哈希
    #[serde(default)]
    pub api_key_hash: Option<String>,

    /// JWT 签名密钥（256位随机数 base64 编码）
    #[serde(default)]
    pub jwt_secret: Option<String>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            api_key_hash: None,
            jwt_secret: None,
        }
    }
}

// ─── API 服务 ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// API 服务监听地址
    #[serde(default = "default_bind_address")]
    pub bind_address: String,

    /// API 服务监听端口
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: default_bind_address(),
            port: default_port(),
        }
    }
}

// ─── 顶层配置 ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub program: ProgramConfig,

    #[serde(default)]
    pub core: CoreConfig,

    #[serde(default)]
    pub auth: AuthConfig,

    #[serde(default)]
    pub server: ServerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            program: ProgramConfig::default(),
            core: CoreConfig::default(),
            auth: AuthConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

// ─── 旧版扁平格式（用于自动迁移）──────────────────────────────

#[derive(Debug, Deserialize)]
struct FlatConfig {
    #[serde(default)]
    forbidden_paths: Option<Vec<String>>,
    #[serde(default)]
    max_file_size: Option<u64>,
    #[serde(default)]
    default_extension: Option<String>,
    #[serde(default)]
    key_file_path: Option<String>,
    #[serde(default)]
    preserve_original_filename: Option<bool>,
    #[serde(default)]
    show_progress: Option<bool>,
    #[serde(default)]
    file_format_version: Option<u8>,
    #[serde(default)]
    salt: Option<String>,
    #[serde(default)]
    argon2_m_cost: Option<u32>,
    #[serde(default)]
    argon2_t_cost: Option<u32>,
    #[serde(default)]
    argon2_p_cost: Option<u32>,
    #[serde(default)]
    api_key_hash: Option<String>,
    #[serde(default)]
    jwt_secret: Option<String>,
    #[serde(default)]
    bind_address: Option<String>,
    #[serde(default)]
    port: Option<u16>,
}

impl From<FlatConfig> for Config {
    fn from(f: FlatConfig) -> Self {
        let program = ProgramConfig::default();
        Config {
            program: ProgramConfig {
                forbidden_paths: f.forbidden_paths.unwrap_or(program.forbidden_paths),
                max_file_size: f.max_file_size.unwrap_or(program.max_file_size),
                default_extension: f.default_extension.unwrap_or(program.default_extension),
                key_file_path: f.key_file_path.unwrap_or(program.key_file_path),
                preserve_original_filename: f
                    .preserve_original_filename
                    .unwrap_or(program.preserve_original_filename),
                show_progress: f.show_progress.unwrap_or(program.show_progress),
                file_format_version: f.file_format_version.unwrap_or(program.file_format_version),
            },
            core: CoreConfig {
                salt: f.salt,
                argon2_m_cost: f.argon2_m_cost.unwrap_or_else(default_argon2_m),
                argon2_t_cost: f.argon2_t_cost.unwrap_or_else(default_argon2_t),
                argon2_p_cost: f.argon2_p_cost.unwrap_or_else(default_argon2_p),
            },
            auth: AuthConfig {
                api_key_hash: f.api_key_hash,
                jwt_secret: f.jwt_secret,
            },
            server: ServerConfig {
                bind_address: f.bind_address.unwrap_or_else(default_bind_address),
                port: f.port.unwrap_or_else(default_port),
            },
        }
    }
}

// ─── serde 默认值函数 ─────────────────────────────────────────

fn default_forbidden_paths() -> Vec<String> {
    vec![
        "/bin".into(),
        "/sbin".into(),
        "/usr/bin".into(),
        "/usr/sbin".into(),
        "/lib".into(),
        "/lib64".into(),
        "/usr/lib".into(),
        "/usr/lib64".into(),
        "/boot".into(),
        "/dev".into(),
        "/proc".into(),
        "/sys".into(),
        "/run".into(),
        "/etc".into(),
        "/root".into(),
        "/var".into(),
        "/tmp".into(),
    ]
}
fn default_max_file_size() -> u64 {
    10 * 1024 * 1024 * 1024
}
fn default_extension() -> String {
    ".leo".into()
}
fn default_key_file_path() -> String {
    "~/.config/leolock/keys.toml".into()
}
fn default_file_version() -> u8 {
    2
}
fn default_bind_address() -> String {
    "127.0.0.1".into()
}
fn default_port() -> u16 {
    3000
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

// ─── Config 方法 ──────────────────────────────────────────────

impl Config {
    /// 加载配置文件（自动迁移旧格式）
    pub fn load() -> Result<Self> {
        Self::load_with_path().map(|(config, _)| config)
    }

    /// 加载配置文件并返回实际路径
    pub fn load_with_path() -> Result<(Self, Option<PathBuf>)> {
        let config_paths = Self::get_config_paths();

        for path in config_paths {
            if path.exists() {
                let content = fs::read_to_string(&path)?;

                // 检测旧格式（无 [section] 头），自动迁移
                if !content.contains("[program]")
                    && !content.contains("[core]")
                    && !content.contains("[auth]")
                    && !content.contains("[server]")
                {
                    if let Ok(flat) = toml::from_str::<FlatConfig>(&content) {
                        let config: Config = flat.into();
                        config.save_to(&path)?;
                        return Ok((config, Some(path)));
                    }
                }

                // 新格式
                let config: Config = toml::from_str(&content).map_err(|e| {
                    BjtError::ConfigError(format!("解析配置文件失败 {}: {}", path.display(), e))
                })?;
                return Ok((config, Some(path)));
            }
        }

        Ok((Config::default(), None))
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_default_config_dir()?;
        fs::create_dir_all(&config_dir)?;
        let config_path = config_dir.join("config.toml");
        self.save_to(&config_path)
    }

    fn save_to(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| BjtError::ConfigError(format!("序列化配置失败: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }

    // ─── 路径相关 ──────────────────────────────────────────

    pub fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = vec![PathBuf::from(".leolock.toml")];
        if let Ok(env_path) = std::env::var("LEOLOCK_CONFIG") {
            paths.push(PathBuf::from(env_path));
        }
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("leolock").join("config.toml"));
        }
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".leolock.toml"));
            paths.push(home_dir.join(".config").join("leolock.toml"));
        }
        paths
    }

    pub fn get_default_config_dir() -> Result<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join("leolock"))
        } else if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir.join(".config").join("leolock"))
        } else {
            Err(BjtError::ConfigError("无法确定配置目录".to_string()))
        }
    }

    pub fn config_dir() -> Result<PathBuf> {
        Self::get_default_config_dir()
    }

    pub fn config_file_path() -> Result<PathBuf> {
        let config_dir = Self::get_default_config_dir()?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn key_file_path(&self) -> Result<PathBuf> {
        let path_str = shellexpand::full(&self.program.key_file_path)
            .map_err(|e| BjtError::ConfigError(format!("展开路径失败: {}", e)))?;
        Ok(PathBuf::from(path_str.to_string()))
    }

    #[allow(dead_code)]
    pub fn default_key_file_path() -> Result<PathBuf> {
        let config = Config::load()?;
        config.key_file_path()
    }

    #[allow(dead_code)]
    pub fn create_config_dir() -> Result<()> {
        let config_dir = Self::get_default_config_dir()?;
        fs::create_dir_all(&config_dir)?;
        Ok(())
    }

    // ─── 状态查询 ──────────────────────────────────────────

    pub fn is_initialized(&self) -> bool {
        self.core.salt.is_some()
    }

    #[allow(dead_code)]
    pub fn has_api_key(&self) -> bool {
        self.auth.api_key_hash.is_some()
    }

    // ─── API 鉴权 ──────────────────────────────────────────

    pub fn generate_api_key(&mut self) -> Result<String> {
        use base64::Engine;
        let mut raw = [0u8; 32];
        getrandom::getrandom(&mut raw)
            .map_err(|e| BjtError::CryptoError(format!("生成 API Key 失败: {}", e)))?;
        let api_key = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);
        let hash = crate::password::PasswordManager::hash_api_key(&api_key)?;
        self.auth.api_key_hash = Some(hash);
        Ok(api_key)
    }

    pub fn generate_jwt_secret(&mut self) -> Result<()> {
        use base64::Engine;
        let mut raw = [0u8; 32];
        getrandom::getrandom(&mut raw)
            .map_err(|e| BjtError::CryptoError(format!("生成 JWT 密钥失败: {}", e)))?;
        self.auth.jwt_secret = Some(base64::engine::general_purpose::STANDARD.encode(raw));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn verify_api_key(&self, key: &str) -> bool {
        match &self.auth.api_key_hash {
            Some(hash) => crate::password::PasswordManager::verify_api_key(key, hash),
            None => false,
        }
    }

    // ─── 安全检查 ──────────────────────────────────────────

    pub fn is_safe_path(&self, path: &Path) -> bool {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false,
        };
        for forbidden in &self.program.forbidden_paths {
            if canonical.starts_with(forbidden) {
                return false;
            }
        }
        true
    }
}
