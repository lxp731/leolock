use crate::errors::{BjtError, Result};
use crate::utils::Utils;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use chrono::{DateTime, Utc};
use getrandom::getrandom;
use std::fs;
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::SystemTime;

pub const KEY_SIZE: usize = 32; // AES-256密钥大小
const NONCE_SIZE: usize = 12; // GCM nonce大小
const MAGIC_BYTES: [u8; 4] = [0x4C, 0x45, 0x4F, 0x32]; // "LEO2"
const FILE_VERSION: u8 = 2;

/// 文件头部结构
#[derive(Debug)]
struct FileHeader {
    magic: [u8; 4],
    version: u8,
    filename_metadata_len: u32,
}

/// 加密文件信息
#[derive(Debug)]
#[allow(dead_code)]
pub struct FileInfo {
    /// 文件路径
    pub path: PathBuf,
    /// 文件格式版本
    pub version: u8,
    /// 原文件名（如果可解密）
    pub original_filename: Option<String>,
    /// 加密文件大小（字节）
    pub encrypted_size: u64,
    /// 文件创建时间
    pub created: Option<DateTime<Utc>>,
    /// 文件修改时间
    pub modified: Option<DateTime<Utc>>,
    /// 是否可解密（有正确的密钥）
    pub decryptable: bool,
}

impl FileHeader {
    fn new(filename_metadata_len: u32) -> Self {
        Self {
            magic: MAGIC_BYTES,
            version: FILE_VERSION,
            filename_metadata_len,
        }
    }
    
    fn write(&self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&[self.version])?;
        writer.write_all(&self.filename_metadata_len.to_le_bytes())?;
        Ok(())
    }
    
    fn read(reader: &mut impl Read) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if magic != MAGIC_BYTES {
            return Err(BjtError::CryptoError("无效的文件格式".to_string()));
        }
        
        let mut version_bytes = [0u8; 1];
        reader.read_exact(&mut version_bytes)?;
        let version = version_bytes[0];
        
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let filename_metadata_len = u32::from_le_bytes(len_bytes);
        
        Ok(Self {
            magic,
            version,
            filename_metadata_len,
        })
    }
}

/// 加密器
pub struct CryptoManager;

impl CryptoManager {
    /// 生成随机AES-256密钥
    #[allow(dead_code)]
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

    /// 加密文件名
    pub fn encrypt_filename(filename: &str, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        // 使用独立的nonce加密文件名
        let mut filename_nonce = [0u8; NONCE_SIZE];
        getrandom(&mut filename_nonce).map_err(|e| {
            BjtError::CryptoError(format!("生成随机nonce失败: {}", e))
        })?;
        
        let cipher = Self::create_cipher(key)?;
        let nonce = Nonce::from_slice(&filename_nonce);
        
        let ciphertext = cipher.encrypt(nonce, filename.as_bytes()).map_err(|e| {
            BjtError::CryptoError(format!("加密文件名失败: {}", e))
        })?;
        
        // 组合：nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&filename_nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// 解密文件名
    pub fn decrypt_filename(encrypted_filename: &[u8], key: &[u8; KEY_SIZE]) -> Result<String> {
        if encrypted_filename.len() < NONCE_SIZE {
            return Err(BjtError::CryptoError("加密文件名数据太短".to_string()));
        }
        
        let cipher = Self::create_cipher(key)?;
        let nonce = Nonce::from_slice(&encrypted_filename[..NONCE_SIZE]);
        let ciphertext = &encrypted_filename[NONCE_SIZE..];
        
        let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
            BjtError::CryptoError(format!("解密文件名失败: {}", e))
        })?;
        
        String::from_utf8(plaintext).map_err(|e| {
            BjtError::CryptoError(format!("文件名解码失败: {}", e))
        })
    }
    
    /// 检测文件版本
    pub fn detect_file_version(file_path: &std::path::Path) -> Result<u8> {
        let mut file = fs::File::open(file_path)?;
        let mut magic = [0u8; 4];
        
        match file.read_exact(&mut magic) {
            Ok(_) => {
                if magic == MAGIC_BYTES {
                    // 读取版本号
                    let mut version = [0u8; 1];
                    file.read_exact(&mut version)?;
                    Ok(version[0])
                } else {
                    // 旧版文件（无魔术字节）
                    Ok(1)
                }
            }
            Err(_) => {
                // 文件太小或读取失败，假设是旧版
                Ok(1)
            }
        }
    }

    /// 加密文件（新版，支持文件名加密）
    pub fn encrypt_file_v2(
        input_path: &std::path::Path, 
        key: &[u8; KEY_SIZE], 
        preserve_filename: bool,
        keep_original: bool
    ) -> Result<PathBuf> {
        // 获取原文件名
        let original_filename = input_path.file_name()
            .ok_or_else(|| BjtError::FileError("无法获取文件名".to_string()))?
            .to_string_lossy()
            .to_string();
        
        // 读取文件内容
        let data = fs::read(input_path).map_err(|e| {
            BjtError::FileError(format!("读取文件失败 {}: {}", input_path.display(), e))
        })?;

        // 加密文件内容
        let encrypted_data = Self::encrypt_data(&data, key)?;
        
        // 创建输出文件
        let display_filename = Utils::get_display_filename(&original_filename, preserve_filename);
        let output_path = if let Some(parent) = input_path.parent() {
            parent.join(&display_filename)
        } else {
            PathBuf::from(&display_filename)
        };
        
        let mut output_file = fs::File::create(&output_path)?;
        
        if preserve_filename {
            // 保留文件名：使用旧版格式（向后兼容）
            output_file.write_all(&encrypted_data)?;
        } else {
            // 加密文件名：使用新版格式
            // 1. 加密文件名
            let encrypted_filename = Self::encrypt_filename(&original_filename, key)?;
            
            // 2. 写入文件头部
            let header = FileHeader::new(encrypted_filename.len() as u32);
            header.write(&mut output_file)?;
            
            // 3. 写入加密的文件名
            output_file.write_all(&encrypted_filename)?;
            
            // 4. 写入文件内容
            output_file.write_all(&encrypted_data)?;
        }
        
        println!("✅ 加密完成: {} -> {}", 
            input_path.display(), 
            output_path.display()
        );
        
        if !preserve_filename {
            println!("  原文件名已加密存储在文件头部");
        }

        // 如果不保留原始文件，安全删除源文件
        if !keep_original {
            Utils::secure_delete_file(input_path)?;
            println!("🗑️  已安全删除源文件: {}", input_path.display());
        }

        Ok(output_path)
    }
    
    /// 解密文件（新版，支持文件名恢复）
    pub fn decrypt_file_v2(
        input_path: &std::path::Path, 
        key: &[u8; KEY_SIZE], 
        keep_original: bool
    ) -> Result<PathBuf> {
        // 检测文件版本
        let version = Self::detect_file_version(input_path)?;
        
        let mut input_file = fs::File::open(input_path)?;
        
        match version {
            1 => {
                // 旧版文件：直接解密内容
                let mut encrypted_data = Vec::new();
                input_file.read_to_end(&mut encrypted_data)?;
                
                let decrypted_data = Self::decrypt_data(&encrypted_data, key)?;
                
                // 从文件名推断原文件名（移除.leo后缀）
                let input_str = input_path.to_string_lossy();
                let output_filename = if let Some(stripped) = input_str.strip_suffix(".leo") {
                    stripped.to_string()
                } else {
                    format!("{}_decrypted", input_str)
                };
                
                let output_path = if let Some(parent) = input_path.parent() {
                    parent.join(&output_filename)
                } else {
                    PathBuf::from(&output_filename)
                };
                
                fs::write(&output_path, &decrypted_data)?;
                
                println!("✅ 解密完成 (旧版格式): {} -> {}", 
                    input_path.display(), 
                    output_path.display()
                );
                
                // 如果不保留原始文件，则安全删除它
                if !keep_original {
                    Utils::secure_delete_file(input_path)?;
                    println!("🗑️  已安全删除加密文件: {}", input_path.display());
                }
                
                Ok(output_path)
            }
            2 => {
                // 新版文件：读取头部和加密的文件名
                let header = FileHeader::read(&mut input_file)?;
                
                // 读取加密的文件名
                let mut encrypted_filename = vec![0u8; header.filename_metadata_len as usize];
                input_file.read_exact(&mut encrypted_filename)?;
                
                // 解密文件名
                let original_filename = Self::decrypt_filename(&encrypted_filename, key)?;
                
                // 读取并解密文件内容
                let mut encrypted_data = Vec::new();
                input_file.read_to_end(&mut encrypted_data)?;
                
                let decrypted_data = Self::decrypt_data(&encrypted_data, key)?;
                
                // 创建输出文件
                let output_path = if let Some(parent) = input_path.parent() {
                    parent.join(&original_filename)
                } else {
                    PathBuf::from(&original_filename)
                };
                
                fs::write(&output_path, &decrypted_data)?;
                
                println!("✅ 解密完成: {} -> {}", 
                    input_path.display(), 
                    output_path.display()
                );
                println!("  原文件名已恢复: {}", original_filename);
                
                // 如果不保留原始文件，则安全删除它
                if !keep_original {
                    Utils::secure_delete_file(input_path)?;
                    println!("🗑️  已安全删除加密文件: {}", input_path.display());
                }
                
                Ok(output_path)
            }
            _ => Err(BjtError::CryptoError(
                format!("不支持的文件版本: {}", version)
            )),
        }
    }

    /// 加密文件（兼容接口）
    pub fn encrypt_file(
        input_path: &std::path::Path, 
        key: &[u8; KEY_SIZE], 
        keep_original: bool
    ) -> Result<()> {
        // 加载配置获取 preserve_original_filename 设置
        let config = crate::config::Config::load()
            .unwrap_or_else(|_| crate::config::Config::default());
        
        Self::encrypt_file_v2(input_path, key, config.preserve_original_filename, keep_original)?;
        Ok(())
    }
    
    /// 加密文件（带文件名加密选项）
    #[allow(dead_code)]
    pub fn encrypt_file_with_options(
        input_path: &std::path::Path, 
        key: &[u8; KEY_SIZE], 
        preserve_filename: bool,
        keep_original: bool
    ) -> Result<()> {
        Self::encrypt_file_v2(input_path, key, preserve_filename, keep_original)?;
        Ok(())
    }

    /// 解密文件（兼容旧接口）
    pub fn decrypt_file(input_path: &std::path::Path, key: &[u8; KEY_SIZE], keep_original: bool) -> Result<()> {
        Self::decrypt_file_v2(input_path, key, keep_original)?;
        Ok(())
    }

    /// 使用密码派生密钥（用于备份加密）
    #[allow(dead_code)]
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

    /// 获取加密文件信息
    pub fn get_file_info(file_path: &std::path::Path, key: Option<&[u8; KEY_SIZE]>) -> Result<FileInfo> {
        // 检查文件是否存在
        if !file_path.exists() {
            return Err(BjtError::FileError(format!("文件不存在: {}", file_path.display())));
        }

        // 获取文件元数据
        let metadata = fs::metadata(file_path)?;
        let encrypted_size = metadata.len();
        
        // 获取文件时间
        let created = metadata.created()
            .ok()
            .and_then(|t| SystemTime::UNIX_EPOCH.checked_add(t.duration_since(SystemTime::UNIX_EPOCH).ok()?))
            .map(|st| DateTime::<Utc>::from(st));
            
        let modified = metadata.modified()
            .ok()
            .and_then(|t| SystemTime::UNIX_EPOCH.checked_add(t.duration_since(SystemTime::UNIX_EPOCH).ok()?))
            .map(|st| DateTime::<Utc>::from(st));

        // 检测文件版本
        let version = Self::detect_file_version(file_path)?;
        
        let mut original_filename = None;
        let decryptable;

        match version {
            1 => {
                // 旧版文件：无法获取原文件名（除非从文件名推断）
                decryptable = key.is_some(); // 有密钥就可以解密
                
                // 尝试从文件名推断原文件名
                // 只对明显是"原文件名+.leo"格式的文件进行推断
                let filename = file_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                    
                if filename.ends_with(".leo") {
                    let stripped = &filename[..filename.len() - 4]; // 移除.leo
                    // 只有当移除后缀后文件名仍然有效时才显示
                    if !stripped.is_empty() && stripped != filename {
                        // 检查是否看起来像原文件名（包含点表示有扩展名，或者不是简单单词）
                        if stripped.contains('.') || stripped.len() > 8 {
                            original_filename = Some(stripped.to_string());
                        } else {
                            // 简单单词如"small"、"test"等，不显示为原文件名
                            original_filename = Some("[文件名已加密]".to_string());
                        }
                    }
                }
            }
            2 => {
                // 新版文件：尝试读取并解密文件名
                let mut file = fs::File::open(file_path)?;
                
                // 跳过魔术字节和版本号
                file.seek(SeekFrom::Start(5))?;
                
                // 读取文件名元数据长度
                let mut len_bytes = [0u8; 4];
                file.read_exact(&mut len_bytes)?;
                let metadata_len = u32::from_le_bytes(len_bytes) as usize;
                
                if let Some(key) = key {
                    // 尝试解密文件名
                    let mut encrypted_filename = vec![0u8; metadata_len];
                    file.read_exact(&mut encrypted_filename)?;
                    
                    match Self::decrypt_filename(&encrypted_filename, key) {
                        Ok(filename) => {
                            original_filename = Some(filename);
                            decryptable = true;
                        }
                        Err(_) => {
                            // 解密失败，可能密钥错误
                            original_filename = Some("[需要正确密钥]".to_string());
                            decryptable = false;
                        }
                    }
                } else {
                    // 没有提供密钥
                    original_filename = Some("[需要密钥查看]".to_string());
                    decryptable = false;
                }
            }
            _ => {
                return Err(BjtError::CryptoError(
                    format!("不支持的文件版本: {}", version)
                ));
            }
        }

        Ok(FileInfo {
            path: file_path.to_path_buf(),
            version,
            original_filename,
            encrypted_size,
            created,
            modified,
            decryptable,
        })
    }
}