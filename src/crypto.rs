use crate::errors::{BjtError, Result};
use crate::utils::Utils;
use aes_gcm::{
    aead::{Aead, AeadInPlace, KeyInit, Payload},
    Aes256Gcm, Key, Nonce,
};
use chrono::{DateTime, Utc};
use getrandom::getrandom;
use zeroize::Zeroize;
use std::fs;
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::PathBuf;
use std::time::SystemTime;

pub const KEY_SIZE: usize = 32; // AES-256密钥大小
const NONCE_SIZE: usize = 12; // GCM nonce大小
const CHUNK_SIZE: usize = 1024 * 1024; // 提升至 1MB 分块大小
const IO_BUFFER_SIZE: usize = 2 * 1024 * 1024; // 2MB IO 缓存
const MAGIC_BYTES: [u8; 4] = [0x4C, 0x45, 0x4F, 0x33]; 
const FILE_VERSION: u8 = 3;
const TAG_SIZE: usize = 16; // GCM 认证标签大小


/// 文件头部结构
#[derive(Debug, Zeroize)]
#[zeroize(drop)]
struct FileHeader {
    magic: [u8; 4],
    version: u8,
    filename_metadata_len: u32,
    #[zeroize(skip)]
    is_streaming: bool,
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
    fn new(filename_metadata_len: u32, is_streaming: bool) -> Self {
        Self {
            magic: MAGIC_BYTES,
            version: FILE_VERSION,
            filename_metadata_len,
            is_streaming,
        }
    }
    
    /// 获取用于 AAD 的字节 (仅限 V3+)
    fn to_aad(&self) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(&self.magic);
        aad.push(self.version);
        aad.extend_from_slice(&self.filename_metadata_len.to_le_bytes());
        if self.version >= 3 {
            aad.push(if self.is_streaming { 1 } else { 0 });
        }
        aad
    }
    
    /// 写入头部到缓冲区
    fn write_to_buffer(&self, buffer: &mut Vec<u8>) -> Result<()> {
        buffer.extend_from_slice(&self.magic);
        buffer.push(self.version);
        buffer.extend_from_slice(&self.filename_metadata_len.to_le_bytes());
        if self.version >= 3 {
            buffer.push(if self.is_streaming { 1 } else { 0 });
        }
        Ok(())
    }
    
    fn read(reader: &mut impl Read) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        // 支持 LEO2 (旧版) 和 LEO3 (新版)
        if magic != MAGIC_BYTES && magic != [0x4C, 0x45, 0x4F, 0x32] {
            return Err(BjtError::CryptoError("无效的文件格式或版本不支持".to_string()));
        }
        
        let mut version_bytes = [0u8; 1];
        reader.read_exact(&mut version_bytes)?;
        let version = version_bytes[0];
        
        let mut len_bytes = [0u8; 4];
        reader.read_exact(&mut len_bytes)?;
        let filename_metadata_len = u32::from_le_bytes(len_bytes);
        
        let is_streaming = if version >= 3 {
            let mut stream_byte = [0u8; 1];
            reader.read_exact(&mut stream_byte)?;
            stream_byte[0] == 1
        } else {
            false
        };
        
        Ok(Self {
            magic,
            version,
            filename_metadata_len,
            is_streaming,
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

    /// 加密数据 (带可选 AAD)
    pub fn encrypt_data_with_aad(data: &[u8], key: &[u8; KEY_SIZE], aad: &[u8]) -> Result<Vec<u8>> {
        let cipher = Self::create_cipher(key)?;
        
        // 生成随机nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        getrandom(&mut nonce_bytes).map_err(|e| {
            BjtError::CryptoError(format!("生成随机nonce失败: {}", e))
        })?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密数据，附带 AAD
        let payload = Payload {
            msg: data,
            aad,
        };
        
        let ciphertext = cipher.encrypt(nonce, payload).map_err(|e| {
            BjtError::CryptoError(format!("加密数据失败: {}", e))
        })?;

        // 组合：nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// 加密数据 (兼容原接口)
    pub fn encrypt_data(data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        Self::encrypt_data_with_aad(data, key, &[])
    }

    /// 解密数据 (带可选 AAD)
    pub fn decrypt_data_with_aad(encrypted_data: &[u8], key: &[u8; KEY_SIZE], aad: &[u8]) -> Result<Vec<u8>> {
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

        // 解密数据，附带 AAD
        let payload = Payload {
            msg: ciphertext,
            aad,
        };
        
        cipher.decrypt(nonce, payload).map_err(|e| {
            BjtError::CryptoError(format!("解密数据失败: {}", e))
        })
    }

    /// 解密数据 (兼容原接口)
    pub fn decrypt_data(encrypted_data: &[u8], key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        Self::decrypt_data_with_aad(encrypted_data, key, &[])
    }

    /// 流式加密 (优化版：原地加密，零内存分配)
    pub fn encrypt_stream(
        reader: &mut impl Read,
        writer: &mut impl Write,
        key: &[u8; KEY_SIZE],
        aad: &[u8],
    ) -> Result<()> {
        let cipher = Self::create_cipher(key)?;
        
        let mut base_nonce = [0u8; NONCE_SIZE];
        getrandom(&mut base_nonce).map_err(|e| {
            BjtError::CryptoError(format!("生成随机nonce失败: {}", e))
        })?;
        writer.write_all(&base_nonce)?;
        
        // 预分配缓冲区，复用内存
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut counter: u64 = 0;
        
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            
            let mut chunk_nonce_bytes = base_nonce;
            let counter_bytes = counter.to_le_bytes();
            for i in 0..8 {
                chunk_nonce_bytes[i] ^= counter_bytes[i];
            }
            let nonce = Nonce::from_slice(&chunk_nonce_bytes);
            
            // 原地加密：不产生新的 Vec
            let tag = cipher
                .encrypt_in_place_detached(nonce, aad, &mut buffer[..n])
                .map_err(|e| BjtError::CryptoError(format!("分块加密失败: {}", e)))?;
            
            // 写入格式：[分块长度(u32)][加密数据][16字节标签]
            writer.write_all(&(n as u32).to_le_bytes())?;
            writer.write_all(&buffer[..n])?;
            writer.write_all(tag.as_slice())?;
            
            counter += 1;
        }
        
        Ok(())
    }

    /// 流式解密 (优化版：原地解密)
    pub fn decrypt_stream(
        reader: &mut impl Read,
        writer: &mut impl Write,
        key: &[u8; KEY_SIZE],
        aad: &[u8],
    ) -> Result<()> {
        let cipher = Self::create_cipher(key)?;
        
        let mut base_nonce = [0u8; NONCE_SIZE];
        reader.read_exact(&mut base_nonce)?;
        
        let mut buffer = vec![0u8; CHUNK_SIZE];
        let mut counter: u64 = 0;
        
        loop {
            let mut len_bytes = [0u8; 4];
            match reader.read_exact(&mut len_bytes) {
                Ok(_) => (),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            let chunk_len = u32::from_le_bytes(len_bytes) as usize;
            
            // 读取加密数据和标签
            reader.read_exact(&mut buffer[..chunk_len])?;
            let mut tag_bytes = [0u8; TAG_SIZE];
            reader.read_exact(&mut tag_bytes)?;
            let tag = aes_gcm::Tag::from_slice(&tag_bytes);
            
            let mut chunk_nonce_bytes = base_nonce;
            let counter_bytes = counter.to_le_bytes();
            for i in 0..8 {
                chunk_nonce_bytes[i] ^= counter_bytes[i];
            }
            let nonce = Nonce::from_slice(&chunk_nonce_bytes);
            
            // 原地解密
            cipher
                .decrypt_in_place_detached(nonce, aad, &mut buffer[..chunk_len], tag)
                .map_err(|e| BjtError::CryptoError(format!("分块解密失败: {}", e)))?;
            
            writer.write_all(&buffer[..chunk_len])?;
            
            counter += 1;
        }
        
        Ok(())
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

    /// 加密文件（支持文件名加密、流式加密和 AAD）
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
        
        // 生成输出文件名
        let display_filename = Utils::get_display_filename(&original_filename, preserve_filename);
        let output_path = if let Some(parent) = input_path.parent() {
            parent.join(&display_filename)
        } else {
            PathBuf::from(&display_filename)
        };
        
        // 创建临时输出文件
        let tmp_path = output_path.with_extension("leo.tmp");
        
        {
            let mut input_file = BufReader::with_capacity(IO_BUFFER_SIZE, fs::File::open(input_path)?);
            let mut tmp_file = BufWriter::with_capacity(IO_BUFFER_SIZE, fs::File::create(&tmp_path)?);
            
            if preserve_filename {
                // 保留文件名：旧版兼容模式
                let mut data = Vec::new();
                input_file.read_to_end(&mut data)?;
                let encrypted_data = Self::encrypt_data(&data, key)?;
                tmp_file.write_all(&encrypted_data)?;
            } else {
                // 加密文件名：使用新版流式格式 (V3)
                let encrypted_filename = Self::encrypt_filename(&original_filename, key)?;
                let header = FileHeader::new(encrypted_filename.len() as u32, true);
                let aad = header.to_aad();
                
                let mut header_buf = Vec::new();
                header.write_to_buffer(&mut header_buf)?;
                tmp_file.write_all(&header_buf)?;
                tmp_file.write_all(&encrypted_filename)?;
                
                Self::encrypt_stream(&mut input_file, &mut tmp_file, key, &aad)?;
            }
            tmp_file.flush()?;
        }
        
        // 原子替换：将临时文件重命名为最终文件
        fs::rename(&tmp_path, &output_path)?;
        
        println!("✅ 加密完成: {} -> {}", 
            input_path.display(), 
            output_path.display()
        );
        
        if !preserve_filename {
            println!("  原文件名已加密存储在文件头部 (AAD 保护)");
        }

        // 如果不保留原始文件，安全删除源文件
        if !keep_original {
            Utils::secure_delete_file(input_path)?;
            println!("🗑️  已安全删除源文件: {}", input_path.display());
        }

        Ok(output_path)
    }
    
    /// 解密文件（支持文件名恢复、流式解密和 AAD）
    pub fn decrypt_file_v2(
        input_path: &std::path::Path, 
        key: &[u8; KEY_SIZE], 
        keep_original: bool
    ) -> Result<PathBuf> {
        // 检测文件版本
        let version = Self::detect_file_version(input_path)?;
        
        let mut input_file = BufReader::with_capacity(IO_BUFFER_SIZE, fs::File::open(input_path)?);
        
        match version {
            1 => {
                // 旧版文件：直接解密内容
                let mut encrypted_data = Vec::new();
                input_file.read_to_end(&mut encrypted_data)?;
                
                let decrypted_data = Self::decrypt_data(&encrypted_data, key)?;
                
                // 从文件名推断原文件名
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
                
                if !keep_original {
                    Utils::secure_delete_file(input_path)?;
                }
                
                Ok(output_path)
            }
            2 => {
                // V2 版本：非流式，加密文件名
                let header = FileHeader::read(&mut input_file)?;
                
                let mut encrypted_filename = vec![0u8; header.filename_metadata_len as usize];
                input_file.read_exact(&mut encrypted_filename)?;
                
                let original_filename = Self::decrypt_filename(&encrypted_filename, key)?;
                
                let mut encrypted_data = Vec::new();
                input_file.read_to_end(&mut encrypted_data)?;
                
                let decrypted_data = Self::decrypt_data(&encrypted_data, key)?;
                
                let output_path = if let Some(parent) = input_path.parent() {
                    parent.join(&original_filename)
                } else {
                    PathBuf::from(&original_filename)
                };
                
                fs::write(&output_path, &decrypted_data)?;
                
                println!("✅ 解密完成 (V2): {} -> {}", 
                    input_path.display(), 
                    output_path.display()
                );
                
                if !keep_original {
                    Utils::secure_delete_file(input_path)?;
                }
                
                Ok(output_path)
            }
            3 => {
                // V3 版本：流式加密，AAD 保护
                let header = FileHeader::read(&mut input_file)?;
                let aad = header.to_aad();
                
                // 读取并解密文件名
                let mut encrypted_filename = vec![0u8; header.filename_metadata_len as usize];
                input_file.read_exact(&mut encrypted_filename)?;
                let original_filename = Self::decrypt_filename(&encrypted_filename, key)?;
                
                // 创建临时输出文件
                let output_path = if let Some(parent) = input_path.parent() {
                    parent.join(&original_filename)
                } else {
                    PathBuf::from(&original_filename)
                };
                let tmp_path = output_path.with_extension("tmp");
                
                {
                    let mut output_file = BufWriter::with_capacity(IO_BUFFER_SIZE, fs::File::create(&tmp_path)?);
                    
                    if header.is_streaming {
                        // 执行流式解密，附带 AAD 校验
                        Self::decrypt_stream(&mut input_file, &mut output_file, key, &aad)?;
                    } else {
                        // 非流式 V3 (理论上不应出现，但逻辑上支持)
                        let mut encrypted_data = Vec::new();
                        input_file.read_to_end(&mut encrypted_data)?;
                        let decrypted_data = Self::decrypt_data_with_aad(&encrypted_data, key, &aad)?;
                        output_file.write_all(&decrypted_data)?;
                    }
                    output_file.flush()?;
                }
                
                // 原子替换
                fs::rename(&tmp_path, &output_path)?;
                
                println!("✅ 解密完成 (V3): {} -> {}", 
                    input_path.display(), 
                    output_path.display()
                );
                
                if !keep_original {
                    Utils::secure_delete_file(input_path)?;
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
            .map(DateTime::<Utc>::from);
            
        let modified = metadata.modified()
            .ok()
            .and_then(|t| SystemTime::UNIX_EPOCH.checked_add(t.duration_since(SystemTime::UNIX_EPOCH).ok()?))
            .map(DateTime::<Utc>::from);

        // 检测文件版本
        let version = Self::detect_file_version(file_path)?;
        
        let original_filename;
        let decryptable;

        match version {
            1 => {
                // 旧版文件
                decryptable = key.is_some();
                let filename = file_path.file_name().unwrap_or_default().to_string_lossy();
                if let Some(stripped) = filename.strip_suffix(".leo") {
                    original_filename = Some(stripped.to_string());
                } else {
                    original_filename = Some(filename.to_string());
                }
            }
            2 | 3 => {
                // V2 和 V3 版本：读取并尝试解密文件名
                let mut file = fs::File::open(file_path)?;
                let header = FileHeader::read(&mut file)?;
                
                if let Some(key) = key {
                    let mut encrypted_filename = vec![0u8; header.filename_metadata_len as usize];
                    file.read_exact(&mut encrypted_filename)?;
                    
                    match Self::decrypt_filename(&encrypted_filename, key) {
                        Ok(filename) => {
                            original_filename = Some(filename);
                            decryptable = true;
                        }
                        Err(_) => {
                            original_filename = Some("[需要正确密钥]".to_string());
                            decryptable = false;
                        }
                    }
                } else {
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