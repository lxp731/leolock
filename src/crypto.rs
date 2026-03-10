use crate::errors::{BjtError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use getrandom::getrandom;
use std::fs;
use std::path::PathBuf;

const KEY_SIZE: usize = 32; // AES-256密钥大小
const NONCE_SIZE: usize = 12; // GCM nonce大小
#[allow(dead_code)]
const TAG_SIZE: usize = 16; // GCM认证标签大小

/// 加密器
pub struct CryptoManager;

impl CryptoManager {
    /// 生成随机AES-256密钥
    pub fn generate_key() -> Result<[u8; KEY_SIZE]> {
        let mut key = [0u8; KEY_SIZE];
        getrandom(&mut key).map_err(|e| {
            BjtError::CryptoError(format!("生成随机密钥失败: {}", e))
        })?;
        Ok(key)
    }

    /// 从字节数组创建加密器
    pub fn create_cipher(key: &[u8; KEY_SIZE]) -> Result<Aes256Gcm> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        Ok(Aes256Gcm::new(key))
    }

    /// 加密数据
    pub fn encrypt_data(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        let cipher = Self::create_cipher(key)?;
        
        // 生成随机nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        getrandom(&mut nonce_bytes).map_err(|e| {
            BjtError::CryptoError(format!("生成随机nonce失败: {}", e))
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密数据
        let ciphertext = cipher.encrypt(nonce, data).map_err(|e| {
            BjtError::CryptoError(format!("加密数据失败: {}", e))
        })?;

        // 组合：nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// 解密数据
    pub fn decrypt_data(encrypted_data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        if encrypted_data.len() < NONCE_SIZE {
            return Err(BjtError::CryptoError(
                "加密数据太短，无法提取nonce".to_string(),
            ));
        }

        let cipher = Self::create_cipher(key)?;
        
        // 提取nonce
        let nonce = Nonce::from_slice(&encrypted_data[..NONCE_SIZE]);
        
        // 提取密文
        let ciphertext = &encrypted_data[NONCE_SIZE..];

        // 解密数据
        cipher.decrypt(nonce, ciphertext).map_err(|e| {
            BjtError::CryptoError(format!("解密数据失败: {}", e))
        })
    }

    /// 加密文件
    pub fn encrypt_file(input_path: &std::path::Path, key: &[u8; KEY_SIZE], keep_original: bool) -> Result<()> {
        // 读取文件内容
        let data = fs::read(input_path).map_err(|e| {
            BjtError::FileError(format!("读取文件失败 {}: {}", input_path.display(), e))
        })?;

        // 加密数据
        let encrypted_data = Self::encrypt_data(&data, key)?;

        // 写入加密文件（添加.leo后缀）
        let output_path = PathBuf::from(format!("{}.leo", input_path.display()));
        fs::write(&output_path, &encrypted_data).map_err(|e| {
            BjtError::FileError(format!("写入加密文件失败 {}: {}", output_path.display(), e))
        })?;

        println!("✅ 加密完成: {} -> {}", 
            input_path.display(), 
            output_path.display()
        );

        // 如果不保留原始文件，安全删除源文件
        if !keep_original {
            crate::utils::Utils::secure_delete_file(input_path)?;
            println!("🗑️  已安全删除源文件: {}", input_path.display());
        }

        Ok(())
    }

    /// 解密文件
    pub fn decrypt_file(input_path: &std::path::Path, key: &[u8; KEY_SIZE], keep_original: bool) -> Result<()> {
        // 读取加密文件
        let encrypted_data = fs::read(input_path).map_err(|e| {
            BjtError::FileError(format!("读取加密文件失败 {}: {}", input_path.display(), e))
        })?;

        // 解密数据
        let decrypted_data = Self::decrypt_data(&encrypted_data, key)?;

        // 写入解密文件（移除.leo后缀）
        let input_str = input_path.to_string_lossy();
        let output_path = if input_str.ends_with(".leo") {
            // 移除最后的".leo"后缀
            let output_str = &input_str[..input_str.len() - 4];
            PathBuf::from(output_str)
        } else {
            // 如果不是.leo文件，保持原样
            input_path.to_path_buf()
        };
        fs::write(&output_path, &decrypted_data).map_err(|e| {
            BjtError::FileError(format!("写入解密文件失败 {}: {}", output_path.display(), e))
        })?;

        println!("✅ 解密完成: {} -> {}", 
            input_path.display(), 
            output_path.display()
        );

        // 如果不保留加密文件，安全删除
        if !keep_original {
            crate::utils::Utils::secure_delete_file(input_path)?;
            println!("🗑️  已安全删除加密文件: {}", input_path.display());
        }

        Ok(())
    }

    /// 使用密码派生密钥（用于备份加密）
    pub fn derive_key_from_password(password: &str, salt: &[u8]) -> Result<[u8; KEY_SIZE]> {
        use argon2::{Algorithm, Argon2, Params, Version};

        let mut key = [0u8; KEY_SIZE];
        
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(19456, 2, 1, Some(KEY_SIZE)).map_err(|e| {
                BjtError::CryptoError(format!("创建Argon2参数失败: {}", e))
            })?,
        );

        argon2.hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| BjtError::CryptoError(format!("密码派生密钥失败: {}", e)))?;

        Ok(key)
    }
}