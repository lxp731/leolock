use crate::errors::{BjtError, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

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

        Ok(STANDARD.encode(&salt))
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