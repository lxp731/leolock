use crate::errors::{BjtError, Result};
use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash as ArgonPasswordHash,
};
use rand::rngs::OsRng;
use rpassword::read_password;
use zeroize::Zeroizing;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

/// 密码验证器
pub struct PasswordManager;

impl PasswordManager {
    /// 哈希密码（使用Argon2id）
    #[allow(dead_code)]
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| BjtError::PasswordError(format!("密码哈希失败: {}", e)))?
            .to_string();

        Ok(password_hash)
    }

    /// 验证密码
    pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool> {
        let parsed_hash = ArgonPasswordHash::new(stored_hash)
            .map_err(|e| BjtError::PasswordError(format!("解析密码哈希失败: {}", e)))?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// 交互式输入密码（无回显，且自动清理内存）
    pub fn read_password_interactive(prompt: &str) -> Result<Zeroizing<String>> {
        print!("{}: ", prompt);
        io::stdout().flush().map_err(|e| {
            BjtError::PasswordError(format!("刷新输出失败: {}", e))
        })?;

        // 尝试使用rpassword，如果失败则回退到标准输入
        let password = match read_password() {
            Ok(p) => p,
            Err(_e) => {
                // rpassword失败，尝试从标准输入读取（有回显，用于非交互式环境）
                println!("[注意: 使用标准输入读取密码]");
                let mut p = String::new();
                io::stdin().read_line(&mut p).map_err(|e| {
                    BjtError::PasswordError(format!("从标准输入读取密码失败: {}", e))
                })?;
                // 移除换行符
                if p.ends_with('\n') {
                    p.pop();
                    if p.ends_with('\r') {
                        p.pop();
                    }
                }
                p
            }
        };
        
        Ok(Zeroizing::new(password))
    }

    /// 交互式修改密码
    #[allow(dead_code)]
    pub fn change_password_interactive() -> Result<(Zeroizing<String>, Zeroizing<String>)> {
        println!("🔄 修改密码");

        // 输入旧密码
        let old_password = Self::read_password_interactive("请输入旧密码")?;

        // 输入新密码
        let new_password = loop {
            let pwd1 = Self::read_password_interactive("请输入新密码（至少8位）")?;
            
            if pwd1.len() < 8 {
                println!("❌ 密码长度不足8位，请重新输入");
                continue;
            }

            let pwd2 = Self::read_password_interactive("请确认新密码")?;
            
            if *pwd1 != *pwd2 {
                println!("❌ 两次输入的密码不一致，请重新输入");
                continue;
            }

            break pwd1;
        };

        Ok((old_password, new_password))
    }



    /// 验证密码强度
    #[allow(dead_code)]
    pub fn validate_password_strength(password: &str) -> Result<()> {
        if password.len() < 8 {
            return Err(BjtError::ValidationError(
                "密码长度必须至少8位".to_string(),
            ));
        }

        // 检查是否包含数字
        if !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(BjtError::ValidationError(
                "密码必须包含至少一个数字".to_string(),
            ));
        }

        // 检查是否包含字母
        if !password.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(BjtError::ValidationError(
                "密码必须包含至少一个字母".to_string(),
            ));
        }

        Ok(())
    }

    /// 保存密码哈希到文件
    #[allow(dead_code)]
    pub fn save_password_hash(password_hash: &str, password_file_path: &Path) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = password_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 保存密码哈希
        fs::write(password_file_path, password_hash)?;
        Ok(())
    }

    /// 从文件加载密码哈希
    pub fn load_password_hash(password_file_path: &Path) -> Result<String> {
        if !password_file_path.exists() {
            return Err(BjtError::PasswordError(
                "密码文件不存在，请先运行 'leolock init'".to_string(),
            ));
        }
        
        let password_hash = fs::read_to_string(password_file_path)?;
        Ok(password_hash)
    }

    /// 验证密码并返回密码哈希
    #[allow(dead_code)]
    pub fn verify_and_get_password_hash(
        password: &str,
        password_file_path: &Path,
    ) -> Result<String> {
        let stored_hash = Self::load_password_hash(password_file_path)?;
        
        if Self::verify_password(password, &stored_hash)? {
            Ok(stored_hash)
        } else {
            Err(BjtError::PasswordError("密码错误".to_string()))
        }
    }
}