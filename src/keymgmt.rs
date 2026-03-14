use crate::config::Config;
use crate::crypto::CryptoManager;
use crate::errors::{BjtError, Result};

use chrono::Local;
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[allow(dead_code)]
const BACKUP_PREFIX: &str = "leolock_key_backup";
#[allow(dead_code)]
const BACKUP_EXTENSION: &str = "enc";

/// 备份文件元数据（明文部分）
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct BackupMetadata {
    version: u8,
    tool_name: String,
    created_at: String,
    key_size: u8,
}

impl Default for BackupMetadata {
    fn default() -> Self {
        Self {
            version: 1,
            tool_name: "leolock".to_string(),
            created_at: Local::now().to_rfc3339(),
            key_size: 32, // AES-256
        }
    }
}

/// 密钥管理器
pub struct KeyManager;

impl KeyManager {
    /// 生成并保存密钥文件
    pub fn generate_and_save_key() -> Result<[u8; 32]> {
        let key = CryptoManager::generate_key()?;
        Self::save_key(&key)?;
        Ok(key)
    }

    /// 保存密钥到文件
    pub fn save_key(key: &[u8; 32]) -> Result<()> {
        let key_path = Config::default_key_file_path()?;
        fs::write(&key_path, key).map_err(|e| {
            BjtError::KeyError(format!("保存密钥文件失败 {}: {}", key_path.display(), e))
        })?;

        // 设置适当的权限（仅用户可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path)?.permissions();
            perms.set_mode(0o600); // rw-------
            fs::set_permissions(&key_path, perms)?;
        }

        println!("✅ 密钥文件已保存: {:?}", key_path);
        Ok(())
    }

    /// 加载密钥文件
    pub fn load_key() -> Result<[u8; 32]> {
        let key_path = Config::default_key_file_path()?;
        
        if !key_path.exists() {
            return Err(BjtError::KeyError(
                "密钥文件不存在，请先运行 'leolock init'".to_string(),
            ));
        }

        let key_data = fs::read(&key_path).map_err(|e| {
            BjtError::KeyError(format!("读取密钥文件失败 {}: {}", key_path.display(), e))
        })?;

        if key_data.len() != 32 {
            return Err(BjtError::KeyError(format!(
                "密钥文件大小不正确: 期望32字节，实际{}字节",
                key_data.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&key_data);
        Ok(key)
    }

    /// 创建加密备份文件（初始化时调用）
    #[allow(dead_code)]
    pub fn create_backup(key: &[u8; 32], password: &str) -> Result<PathBuf> {
        // 生成备份文件名（时间戳精确到秒）
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("{}_{}.{}", BACKUP_PREFIX, timestamp, BACKUP_EXTENSION);
        
        let home = home_dir().ok_or_else(|| {
            BjtError::BackupError("无法获取用户家目录".to_string())
        })?;
        
        let backup_path = home.join(&backup_name);

        // 检查是否已存在同名文件
        if backup_path.exists() {
            return Err(BjtError::BackupError(format!(
                "备份文件已存在: {}",
                backup_path.display()
            )));
        }

        // 创建备份文件
        Self::encrypt_and_save_backup(&backup_path, key, password)?;

        println!("✅ 备份文件已创建: {}", backup_path.display());
        Ok(backup_path)
    }

    /// 加密并保存备份文件
    fn encrypt_and_save_backup(
        backup_path: &Path,
        key: &[u8; 32],
        password: &str,
    ) -> Result<()> {
        // 生成随机盐值用于密钥派生
        let mut salt = [0u8; 16];
        getrandom::getrandom(&mut salt).map_err(|e| {
            BjtError::CryptoError(format!("生成随机盐值失败: {}", e))
        })?;

        // 从密码派生加密密钥
        let encryption_key = CryptoManager::derive_key_from_password(password, &salt)?;

        // 加密密钥数据
        let encrypted_key = CryptoManager::encrypt_data(key, &encryption_key)?;

        // 创建备份数据结构
        let backup_data = BackupData {
            metadata: BackupMetadata::default(),
            salt: salt.to_vec(),
            encrypted_key,
        };

        // 序列化为JSON
        let json_data = serde_json::to_vec(&backup_data).map_err(|e| {
            BjtError::BackupError(format!("序列化备份数据失败: {}", e))
        })?;

        // 写入文件
        fs::write(backup_path, &json_data).map_err(|e| {
            BjtError::BackupError(format!("写入备份文件失败 {}: {}", backup_path.display(), e))
        })?;

        Ok(())
    }

    /// 从备份文件恢复密钥
    #[allow(dead_code)]
    pub fn recover_from_backup(backup_path: &Path, password: &str) -> Result<[u8; 32]> {
        if !backup_path.exists() {
            return Err(BjtError::BackupError(
                "备份文件不存在".to_string(),
            ));
        }

        // 读取备份文件
        let json_data = fs::read(backup_path).map_err(|e| {
            BjtError::BackupError(format!("读取备份文件失败 {}: {}", backup_path.display(), e))
        })?;

        // 解析备份数据
        let backup_data: BackupData = serde_json::from_slice(&json_data).map_err(|e| {
            BjtError::BackupError(format!("解析备份文件失败: {}", e))
        })?;

        // 验证版本
        if backup_data.metadata.version != 1 {
            return Err(BjtError::BackupError(format!(
                "不支持的备份版本: {}",
                backup_data.metadata.version
            )));
        }

        // 从密码派生加密密钥
        let encryption_key = CryptoManager::derive_key_from_password(
            password,
            &backup_data.salt,
        )?;

        // 解密密钥数据
        let decrypted_key = CryptoManager::decrypt_data(
            &backup_data.encrypted_key,
            &encryption_key,
        )?;

        if decrypted_key.len() != 32 {
            return Err(BjtError::BackupError(format!(
                "解密后的密钥大小不正确: 期望32字节，实际{}字节",
                decrypted_key.len()
            )));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted_key);
        Ok(key)
    }

    /// 确认危险操作（密钥更新）
    #[allow(dead_code)]
    pub fn confirm_dangerous_operation() -> Result<()> {
        println!("{}", "=".repeat(60));
        println!("⚠️  警告：重新生成密钥是危险操作！");
        println!("{}", "=".repeat(60));
        println!();
        println!("这将导致：");
        println!("1. 旧密钥加密的所有文件将无法解密！");
        println!("2. 旧的备份文件将失效！");
        println!("3. 不会创建新的备份文件！");
        println!();
        println!("建议：");
        println!("1. 先解密所有重要文件");
        println!("2. 再运行此命令");
        println!("3. 立即备份新密钥（手动复制leolock.key文件）");
        println!();

        print!("是否继续？ [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" {
            println!("操作已取消");
            return Err(BjtError::KeyError("用户取消操作".to_string()));
        }

        println!();
        Ok(())
    }

    /// 显示备份警告信息
    #[allow(dead_code)]
    pub fn show_backup_warning(backup_path: &Path) {
        println!("{}", "=".repeat(60));
        println!("⚠️  重要警告：请立即备份密钥文件！");
        println!("{}", "=".repeat(60));
        println!();
        println!("备份文件已创建于: {}", backup_path.display());
        println!();
        println!("请立即将此备份文件转移到安全位置：");
        println!("• 外部存储设备（U盘、移动硬盘）");
        println!("• 加密的云存储");
        println!("• 其他安全的离线位置");
        println!();
        println!("如果没有此备份，密钥文件丢失将导致：");
        println!("• 所有加密数据永久无法恢复！");
        println!("• 无法解密任何已加密的文件！");
        println!();
        println!("记住：备份文件同样需要密码保护！");
        println!("{}", "=".repeat(60));
    }
}

/// 备份文件数据结构
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct BackupData {
    metadata: BackupMetadata,
    salt: Vec<u8>,
    encrypted_key: Vec<u8>,
}