use crate::errors::{BjtError, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR_NAME: &str = ".config/leolock";
const CONFIG_FILE_NAME: &str = "leolock.conf";
const KEY_FILE_NAME: &str = "leolock.key";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// 加密文件后缀
    pub suffix: String,

    /// 密码哈希（Argon2id格式）
    pub password_hash: String,

    /// 盐值（base64编码）
    pub salt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            suffix: ".leo".to_string(),
            password_hash: String::new(),
            salt: String::new(),
        }
    }
}

impl Config {
    /// 获取配置目录路径
    pub fn config_dir() -> Result<PathBuf> {
        let home = home_dir().ok_or_else(|| {
            BjtError::ConfigError("无法获取用户家目录".to_string())
        })?;
        Ok(home.join(CONFIG_DIR_NAME))
    }

    /// 获取配置文件路径
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CONFIG_FILE_NAME))
    }

    /// 获取密钥文件路径
    pub fn key_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(KEY_FILE_NAME))
    }

    /// 创建配置目录
    pub fn create_config_dir() -> Result<()> {
        let config_dir = Self::config_dir()?;
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                BjtError::ConfigError(format!("创建配置目录失败: {}", e))
            })?;
            println!("✅ 创建配置目录: {:?}", config_dir);
        }
        Ok(())
    }

    /// 加载配置文件
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        if !config_path.exists() {
            return Err(BjtError::ConfigError(
                "配置文件不存在，请先运行 'leolock init'".to_string(),
            ));
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            BjtError::ConfigError(format!("读取配置文件失败: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            BjtError::ConfigError(format!("解析配置文件失败: {}", e))
        })
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<()> {
        Self::create_config_dir()?;
        
        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self).map_err(|e| {
            BjtError::ConfigError(format!("序列化配置失败: {}", e))
        })?;

        fs::write(&config_path, content).map_err(|e| {
            BjtError::ConfigError(format!("写入配置文件失败: {}", e))
        })?;

        println!("✅ 配置文件已保存: {:?}", config_path);
        Ok(())
    }

    /// 检查是否已初始化
    pub fn is_initialized() -> bool {
        Self::config_file_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }
}