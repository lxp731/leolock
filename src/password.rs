use crate::errors::{BjtError, Result};
use argon2::{
    password_hash::{PasswordHasher, PasswordVerifier, SaltString},
    Argon2, PasswordHash as ArgonPasswordHash,
};
use keyring::Entry;
use rand::rngs::OsRng;
use rpassword::read_password;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "leolock";
const KEYRING_USER: &str = "default";

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

    /// 从环境变量读取密码
    pub fn get_password_from_env(var_name: &str) -> Result<Zeroizing<String>> {
        match env::var(var_name) {
            Ok(p) => Ok(Zeroizing::new(p)),
            Err(_) => Err(BjtError::PasswordError(format!(
                "环境变量 {} 未设置",
                var_name
            ))),
        }
    }

    /// 从标准输入读取密码（非交互式，适用于管道）
    pub fn get_password_from_stdin() -> Result<Zeroizing<String>> {
        let mut p = String::new();
        io::stdin().read_to_string(&mut p).map_err(|e| {
            BjtError::PasswordError(format!("从标准输入读取失败: {}", e))
        })?;
        // 移除末尾换行符
        let trimmed = p.trim_end_matches(['\r', '\n']).to_string();
        Ok(Zeroizing::new(trimmed))
    }

    /// 从系统钥匙串获取密码
    pub fn get_password_from_keyring() -> Result<Zeroizing<String>> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
            BjtError::PasswordError(format!("访问钥匙串失败: {}", e))
        })?;

        match entry.get_password() {
            Ok(p) => Ok(Zeroizing::new(p)),
            Err(e) => Err(BjtError::PasswordError(format!(
                "从钥匙串获取密码失败: {}",
                e
            ))),
        }
    }

    /// 将密码保存到系统钥匙串
    pub fn set_password_to_keyring(password: &str) -> Result<()> {
        let entry = Entry::new(KEYRING_SERVICE, KEYRING_USER).map_err(|e| {
            BjtError::PasswordError(format!("访问钥匙串失败: {}", e))
        })?;

        entry.set_password(password).map_err(|e| {
            BjtError::PasswordError(format!("保存密码到钥匙串失败: {}", e))
        })?;
        Ok(())
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

            if let Err(e) = Self::validate_password_strength(&pwd1) {
                println!("❌ {}", e);
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

    /// 验证密码强度并提供详细反馈
    pub fn validate_password_strength(password: &str) -> Result<()> {
        let mut score = 0;
        let mut feedback = Vec::new();

        if password.len() < 8 {
            return Err(BjtError::ValidationError(
                "密码过短：至少需要8个字符".to_string(),
            ));
        }
        score += 1;

        if password.chars().any(|c| c.is_ascii_digit()) {
            score += 1;
        } else {
            feedback.push("建议包含数字");
        }

        if password.chars().any(|c| c.is_ascii_lowercase()) {
            score += 1;
        } else {
            feedback.push("建议包含小写字母");
        }

        if password.chars().any(|c| c.is_ascii_uppercase()) {
            score += 1;
        } else {
            feedback.push("建议包含大写字母");
        }

        if password.chars().any(|c| !c.is_alphanumeric()) {
            score += 1;
        } else {
            feedback.push("建议包含特殊符号");
        }

        if score < 3 {
            return Err(BjtError::ValidationError(format!(
                "密码太弱 (强度: {}/5)。{}",
                score,
                feedback.join("，")
            )));
        }

        if !feedback.is_empty() {
            println!("💡 密码强度提醒 ({} / 5): {}", score, feedback.join("，"));
        } else {
            println!("✅ 密码强度极佳 (5 / 5)");
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