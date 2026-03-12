use crate::errors::{BjtError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 统一的应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // === 安全设置 ===
    
    /// 危险路径列表（禁止处理的系统目录）
    pub forbidden_paths: Vec<String>,
    
    /// 最大文件大小（字节），0表示无限制
    pub max_file_size: u64,
    
    /// 是否启用进度显示
    pub show_progress: bool,
    
    // === 加密设置 ===
    
    /// 默认加密文件后缀
    pub default_extension: String,
    
    /// 密钥文件位置
    pub key_file_path: String,
    
    /// 密码文件位置
    pub password_file_path: String,
    
    /// 是否保留原文件名（false=加密文件名，true=保留文件名）
    pub preserve_original_filename: bool,
    
    /// 加密文件格式版本
    pub file_format_version: u8,
    
    // === 密码和密钥设置（敏感信息，不保存到文件）===
    
    /// 密码哈希（Argon2id）
    #[serde(skip)]
    pub password_hash: Option<String>,
    
    /// 盐值（base64编码）
    #[serde(skip)]
    pub salt: Option<String>,
    
    /// 是否已初始化
    #[serde(skip)]
    pub initialized: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            forbidden_paths: vec![
                // 系统核心目录（绝对不能加密）
                "/bin".to_string(),
                "/sbin".to_string(),
                "/usr/bin".to_string(),
                "/usr/sbin".to_string(),
                "/lib".to_string(),
                "/lib64".to_string(),
                "/usr/lib".to_string(),
                "/usr/lib64".to_string(),
                
                // 系统运行时目录
                "/boot".to_string(),
                "/dev".to_string(),
                "/proc".to_string(),
                "/sys".to_string(),
                "/run".to_string(),
                "/etc".to_string(),
                
                // 特殊目录
                "/root".to_string(),     // root用户家目录
                "/var".to_string(),      // 系统变量文件
                "/tmp".to_string(),      // 临时文件
            ],
            max_file_size: 10 * 1024 * 1024 * 1024, // 10GB
            show_progress: true,
            default_extension: ".leo".to_string(),
            key_file_path: "~/.config/leolock/keys.toml".to_string(),
            password_file_path: "~/.config/leolock/password.bin".to_string(),
            preserve_original_filename: false,  // 默认加密文件名
            file_format_version: 2,             // 新文件格式版本
            password_hash: None,
            salt: None,
            initialized: false,
        }
    }
}

impl Config {
    // === 配置文件管理 ===
    
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        // 1. 尝试从环境变量获取配置文件路径
        let config_paths = Self::get_config_paths();
        
        for path in config_paths {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                let mut config: Config = toml::from_str(&content).map_err(|e| {
                    BjtError::ConfigError(format!("解析配置文件失败 {}: {}", path.display(), e))
                })?;
                
                // 标记为已加载配置文件
                config.initialized = true;
                return Ok(config);
            }
        }
        
        // 2. 使用默认配置（未初始化状态）
        Ok(Config::default())
    }
    
    /// 获取可能的配置文件路径
    pub fn get_config_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        
        // 1. 当前目录的配置文件
        paths.push(PathBuf::from(".leolock.toml"));
        
        // 2. 从环境变量获取
        if let Ok(env_path) = std::env::var("LEOLOCK_CONFIG") {
            paths.push(PathBuf::from(env_path));
        }
        
        // 3. XDG 配置目录
        if let Some(config_dir) = dirs::config_dir() {
            paths.push(config_dir.join("leolock").join("config.toml"));
        }
        
        // 4. 用户主目录
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join(".leolock.toml"));
            paths.push(home_dir.join(".config").join("leolock.toml"));
        }
        
        paths
    }
    
    /// 保存配置文件（只保存非敏感设置）
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::get_default_config_dir()?;
        
        // 确保目录存在
        fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("config.toml");
        
        // 创建只包含非敏感设置的配置
        let safe_config = SafeConfig {
            forbidden_paths: self.forbidden_paths.clone(),
            max_file_size: self.max_file_size,
            show_progress: self.show_progress,
            default_extension: self.default_extension.clone(),
            key_file_path: self.key_file_path.clone(),
            password_file_path: self.password_file_path.clone(),
            preserve_original_filename: self.preserve_original_filename,
            file_format_version: self.file_format_version,
        };
        
        let content = toml::to_string_pretty(&safe_config).map_err(|e| {
            BjtError::ConfigError(format!("序列化配置失败: {}", e))
        })?;
        
        fs::write(config_path, content)?;
        Ok(())
    }
    
    /// 获取默认配置目录
    pub fn get_default_config_dir() -> Result<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join("leolock"))
        } else if let Some(home_dir) = dirs::home_dir() {
            Ok(home_dir.join(".config").join("leolock"))
        } else {
            Err(BjtError::ConfigError("无法确定配置目录".to_string()))
        }
    }
    
    /// 检查工具是否已初始化
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// 获取配置目录路径
    pub fn config_dir() -> Result<PathBuf> {
        Self::get_default_config_dir()
    }
    
    /// 获取密钥文件路径
    pub fn key_file_path(&self) -> Result<PathBuf> {
        let path_str = shellexpand::full(&self.key_file_path).map_err(|e| {
            BjtError::ConfigError(format!("展开路径失败: {}", e))
        })?;
        Ok(PathBuf::from(path_str.to_string()))
    }
    
    /// 获取密码文件路径
    pub fn password_file_path(&self) -> Result<PathBuf> {
        let path_str = shellexpand::full(&self.password_file_path).map_err(|e| {
            BjtError::ConfigError(format!("展开路径失败: {}", e))
        })?;
        Ok(PathBuf::from(path_str.to_string()))
    }
    
    /// 静态方法：获取默认密钥文件路径
    pub fn default_key_file_path() -> Result<PathBuf> {
        let config = Config::load()?;
        config.key_file_path()
    }
    
    /// 创建配置目录
    pub fn create_config_dir() -> Result<()> {
        let config_dir = Self::get_default_config_dir()?;
        fs::create_dir_all(&config_dir)?;
        Ok(())
    }
    
    // === 安全路径检查 ===
    
    /// 检查路径是否安全（不在危险路径中）
    pub fn is_safe_path(&self, path: &Path) -> bool {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false,
        };
        
        for forbidden in &self.forbidden_paths {
            if canonical.starts_with(forbidden) {
                return false;
            }
        }
        
        true
    }
    

}

/// 安全配置（只包含可以保存到文件的非敏感设置）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SafeConfig {
    pub forbidden_paths: Vec<String>,
    pub max_file_size: u64,
    pub show_progress: bool,
    pub default_extension: String,
    pub key_file_path: String,
    pub password_file_path: String,
    pub preserve_original_filename: bool,
    pub file_format_version: u8,
}