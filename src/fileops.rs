use crate::crypto::CryptoManager;
use crate::errors::{BjtError, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 危险系统路径（禁止处理）
const FORBIDDEN_PATHS: &[&str] = &[
    "/bin", "/sbin", "/usr/bin", "/usr/sbin",
    "/lib", "/lib64", "/usr/lib", "/usr/lib64",
    "/boot", "/dev", "/proc", "/sys", "/run",
    "/etc", "/root", "/var", "/tmp",
];

/// 文件操作管理器
pub struct FileOps;

impl FileOps {
    /// 检查路径是否安全
    pub fn is_safe_path(path: &Path) -> bool {
        let canonical = match fs::canonicalize(path) {
            Ok(p) => p,
            Err(_) => return false,
        };

        // 检查是否在危险路径下
        for forbidden in FORBIDDEN_PATHS {
            if canonical.starts_with(forbidden) {
                return false;
            }
        }

        true
    }

    /// 递归加密文件夹
    pub fn encrypt_directory(dir_path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        if !Self::is_safe_path(dir_path) {
            return Err(BjtError::FileError(format!(
                "路径不安全，跳过: {}",
                dir_path.display()
            )));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut visited = HashSet::new();

        println!("开始加密目录: {}", dir_path.display());
        println!("{}", "-".repeat(40));

        for entry in WalkDir::new(dir_path)
            .follow_links(false) // 不跟随符号链接
            .into_iter()
            .filter_entry(|e| Self::filter_entry(e))
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    // 跳过目录本身（只处理文件）
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(path) {
                        if visited.contains(&canonical) {
                            println!("⚠️  跳过循环符号链接: {}", path.display());
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 处理文件
                    match Self::process_file(path, key, true, keep_original) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            error_count += 1;
                            println!("❌ 加密失败 {}: {}", path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ 遍历错误: {}", e);
                }
            }
        }

        println!("{}", "-".repeat(40));
        println!("加密完成:");
        println!("  ✅ 成功: {} 个文件", success_count);
        println!("  ❌ 失败: {} 个文件", error_count);

        if error_count > 0 {
            println!("注意：部分文件加密失败，请检查错误信息");
        }

        Ok(())
    }

    /// 递归解密文件夹
    pub fn decrypt_directory(dir_path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        if !Self::is_safe_path(dir_path) {
            return Err(BjtError::FileError(format!(
                "路径不安全，跳过: {}",
                dir_path.display()
            )));
        }

        let mut success_count = 0;
        let mut skip_count = 0;
        let mut error_count = 0;
        let mut visited = HashSet::new();

        println!("开始解密目录: {}", dir_path.display());
        println!("{}", "-".repeat(40));

        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| Self::filter_entry(e))
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 只处理.leo后缀的文件
                    if !path.to_string_lossy().ends_with(".leo") {
                        skip_count += 1;
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(path) {
                        if visited.contains(&canonical) {
                            println!("⚠️  跳过循环符号链接: {}", path.display());
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 处理文件
                    match Self::process_file(path, key, false, keep_original) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            error_count += 1;
                            println!("❌ 解密失败 {}: {}", path.display(), e);
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ 遍历错误: {}", e);
                }
            }
        }

        println!("{}", "-".repeat(40));
        println!("解密完成:");
        println!("  ✅ 成功: {} 个文件", success_count);
        println!("  ⏭️  跳过: {} 个非加密文件", skip_count);
        println!("  ❌ 失败: {} 个文件", error_count);

        if skip_count > 0 {
            println!("提示：跳过了非加密文件（无.leo后缀）");
        }
        if error_count > 0 {
            println!("注意：部分文件解密失败，请检查错误信息");
        }

        Ok(())
    }

    /// 处理单个文件（加密或解密）
    fn process_file(path: &Path, key: &[u8; 32], encrypt: bool, keep_original: bool) -> Result<()> {
        // 检查符号链接
        let metadata = fs::symlink_metadata(path)?;
        
        if metadata.file_type().is_symlink() {
            // 对于符号链接，处理源文件
            let target = fs::read_link(path)?;
            if target.exists() {
                println!("🔗 处理符号链接: {} -> {}", path.display(), target.display());
                return if encrypt {
                    Self::process_file(&target, key, true, keep_original)
                } else {
                    Self::process_file(&target, key, false, keep_original)
                };
            } else {
                return Err(BjtError::FileError(format!(
                    "符号链接目标不存在: {}",
                    path.display()
                )));
            }
        }

        // 处理普通文件
        if encrypt {
            CryptoManager::encrypt_file(path, key, keep_original)
        } else {
            CryptoManager::decrypt_file(path, key, keep_original)
        }
    }

    /// 过滤目录条目
    fn filter_entry(entry: &walkdir::DirEntry) -> bool {
        let path = entry.path();
        
        // 跳过危险路径
        if !Self::is_safe_path(path) {
            return false;
        }

        // 跳过隐藏文件（以.开头）
        if entry.file_name().to_string_lossy().starts_with('.') {
            return false;
        }

        true
    }

    /// 加密文件或文件夹
    pub fn encrypt_path(path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        if !path.exists() {
            return Err(BjtError::FileError(format!(
                "路径不存在: {}",
                path.display()
            )));
        }

        let metadata = fs::metadata(path)?;
        
        if metadata.is_dir() {
            Self::encrypt_directory(path, key, keep_original)
        } else if metadata.is_file() {
            Self::process_file(path, key, true, keep_original)
        } else {
            Err(BjtError::FileError(format!(
                "不支持的文件类型: {}",
                path.display()
            )))
        }
    }

    /// 解密文件或文件夹
    pub fn decrypt_path(path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        if !path.exists() {
            return Err(BjtError::FileError(format!(
                "路径不存在: {}",
                path.display()
            )));
        }

        let metadata = fs::metadata(path)?;
        
        if metadata.is_dir() {
            Self::decrypt_directory(path, key, keep_original)
        } else if metadata.is_file() {
            Self::process_file(path, key, false, keep_original)
        } else {
            Err(BjtError::FileError(format!(
                "不支持的文件类型: {}",
                path.display()
            )))
        }
    }

    /// 检查文件是否已加密（根据后缀）
    pub fn is_encrypted_file(path: &Path) -> bool {
        path.to_string_lossy().ends_with(".leo")
    }

    /// 获取文件大小（人类可读格式）
    pub fn get_file_size(path: &Path) -> Result<String> {
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        
        let units = ["B", "KB", "MB", "GB", "TB"];
        let mut size_f64 = size as f64;
        let mut unit_index = 0;
        
        while size_f64 >= 1024.0 && unit_index < units.len() - 1 {
            size_f64 /= 1024.0;
            unit_index += 1;
        }
        
        Ok(format!("{:.2} {}", size_f64, units[unit_index]))
    }
}