use crate::errors::{BjtError, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use sha2::{Sha256, Digest};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 工具函数集合
pub struct Utils;

impl Utils {
    /// 交互式确认
    pub fn confirm(prompt: &str) -> Result<bool> {
        print!("{} [y/N]: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y")
    }

    /// 生成随机盐值（base64编码）
    pub fn generate_salt() -> Result<String> {
        let mut salt = [0u8; 16];
        getrandom::getrandom(&mut salt).map_err(|e| {
            BjtError::CryptoError(format!("生成随机盐值失败: {}", e))
        })?;

        Ok(STANDARD.encode(salt))
    }





    /// 生成文件名哈希（用于加密后的显示文件名）
    pub fn generate_filename_hash(filename: &str) -> String {
        let mut hasher = Sha256::new();
        
        // 输入：原文件名 + 时间戳 + 随机数
        hasher.update(filename.as_bytes());
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        hasher.update(timestamp.to_le_bytes());
        
        let random: u64 = rand::random();
        hasher.update(random.to_le_bytes());
        
        // 取前12字节的十六进制表示（24个字符）
        let result = hasher.finalize();
        hex::encode(&result[..12])
    }
    
    /// 获取显示文件名（根据配置决定是否加密文件名）
    pub fn get_display_filename(filename: &str, preserve_original: bool) -> String {
        if preserve_original {
            // 保留原文件名
            format!("{}.leo", filename)
        } else {
            // 生成加密后的显示文件名
            let hash = Self::generate_filename_hash(filename);
            format!("{}.leo", hash)
        }
    }

    /// 安全删除文件（先清空内容）
    pub fn secure_delete_file(path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        // 获取文件大小
        let metadata = fs::metadata(path)?;
        let file_size = metadata.len();

        // 用随机数据覆盖文件内容
        if file_size > 0 {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .truncate(false)
                .open(path)?;

            // 生成随机数据并写入
            let mut random_data = vec![0u8; file_size as usize];
            getrandom::getrandom(&mut random_data).map_err(|e| {
                BjtError::CryptoError(format!("生成随机数据失败: {}", e))
            })?;

            file.write_all(&random_data)?;
            file.sync_all()?;
        }

        // 删除文件
        fs::remove_file(path)?;

        Ok(())
    }






}