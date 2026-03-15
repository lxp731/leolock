use crate::config::Config;
use crate::crypto::CryptoManager;
use crate::errors::{BjtError, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 文件操作管理器
pub struct FileOps;

impl FileOps {
    /// 检查路径是否安全
    pub fn is_safe_path(path: &Path) -> bool {
        // 从配置文件加载配置
        match Config::load() {
            Ok(config) => config.is_safe_path(path),
            Err(_) => {
                // 如果加载配置失败，使用默认配置检查
                let default_config = Config::default();
                default_config.is_safe_path(path)
            }
        }
    }

    /// 递归加密文件夹
    pub fn encrypt_directory_with_progress(
        dir_path: &Path, 
        key: &[u8; 32], 
        keep_original: bool,
        _show_progress: bool, // 保留参数但不使用
    ) -> Result<()> {
        if !Self::is_safe_path(dir_path) {
            return Err(BjtError::FileError(format!(
                "路径不安全，跳过: {}",
                dir_path.display()
            )));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut total_bytes = 0;
        let mut visited = HashSet::new();

        println!("开始加密目录: {}", dir_path.display());

        // 先收集所有文件路径，避免在遍历时修改文件系统
        let mut file_entries = Vec::new();
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 跳过已经加密的文件（.leo扩展名）
                    if path.to_string_lossy().ends_with(".leo") {
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(&path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 获取文件大小
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    
                    file_entries.push((path, file_size));
                    total_bytes += file_size;
                }
                Err(e) => {
                    println!("❌ 遍历错误: {}", e);
                }
            }
        }

        if file_entries.is_empty() {
            println!("📁 目录中没有可加密的文件: {}", dir_path.display());
            return Ok(());
        }

        println!("文件数: {}, 总大小: {}", file_entries.len(), Self::format_bytes(total_bytes));
        println!("{}", "-".repeat(40));

        // 处理收集到的文件
        for (path, _file_size) in file_entries {
            // 处理文件
            match Self::process_file(&path, key, true, keep_original) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ 加密失败 {}: {}", path.display(), e);
                }
            }
        }

        println!("{}", "-".repeat(40));
        println!("加密完成:");
        println!("  ✅ 成功: {} 个文件", success_count);
        if error_count > 0 {
            println!("  ❌ 失败: {} 个文件", error_count);
        }
        println!("  📊 总大小: {}", Self::format_bytes(total_bytes));

        if error_count > 0 {
            Err(BjtError::FileError(format!(
                "加密完成，但有 {} 个文件失败",
                error_count
            )))
        } else {
            Ok(())
        }
    }

    /// 递归解密文件夹
    pub fn decrypt_directory_with_progress(
        dir_path: &Path, 
        key: &[u8; 32], 
        keep_original: bool,
        _show_progress: bool, // 保留参数但不使用
    ) -> Result<()> {
        if !Self::is_safe_path(dir_path) {
            return Err(BjtError::FileError(format!(
                "路径不安全，跳过: {}",
                dir_path.display()
            )));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut skip_count = 0;
        let mut total_bytes = 0;
        let mut visited = HashSet::new();

        println!("开始解密目录: {}", dir_path.display());

        // 先收集所有文件路径
        let mut file_entries = Vec::new();
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 只处理.leo后缀的文件
                    if !path.to_string_lossy().ends_with(".leo") {
                        skip_count += 1;
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(&path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 获取文件大小
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    
                    file_entries.push((path, file_size));
                    total_bytes += file_size;
                }
                Err(e) => {
                    println!("❌ 遍历错误: {}", e);
                }
            }
        }

        if file_entries.is_empty() {
            if skip_count > 0 {
                println!("📁 目录中没有加密文件 (.leo): {}", dir_path.display());
                println!("  跳过了 {} 个非加密文件", skip_count);
            } else {
                println!("📁 目录中没有文件: {}", dir_path.display());
            }
            return Ok(());
        }

        println!("加密文件数: {}, 总大小: {}", file_entries.len(), Self::format_bytes(total_bytes));
        println!("{}", "-".repeat(40));

        // 处理收集到的文件
        for (path, _file_size) in file_entries {
            // 处理文件
            match Self::process_file(&path, key, false, keep_original) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ 解密失败 {}: {}", path.display(), e);
                }
            }
        }

        println!("{}", "-".repeat(40));
        println!("解密完成:");
        println!("  ✅ 成功: {} 个文件", success_count);
        if skip_count > 0 {
            println!("  ⏭️  跳过: {} 个非加密文件", skip_count);
        }
        if error_count > 0 {
            println!("  ❌ 失败: {} 个文件", error_count);
        }
        println!("  📊 总大小: {}", Self::format_bytes(total_bytes));

        if error_count > 0 {
            Err(BjtError::FileError(format!(
                "解密完成，但有 {} 个文件失败",
                error_count
            )))
        } else {
            Ok(())
        }
    }

    /// 加密文件或文件夹
    pub fn encrypt_path_with_config(
        path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        config: &Config,
    ) -> Result<()> {
        if path.is_dir() {
            Self::encrypt_directory_with_config(path, key, keep_original, config)
        } else {
            Self::process_file_with_options(path, key, true, config.preserve_original_filename, keep_original).map(|_| ())
        }
    }
    
    /// 递归加密文件夹（使用配置）
    pub fn encrypt_directory_with_config(
        dir_path: &Path, 
        key: &[u8; 32], 
        keep_original: bool,
        config: &Config,
    ) -> Result<()> {
        // 使用配置中的 preserve_original_filename 设置
        Self::encrypt_directory_with_options(dir_path, key, keep_original, config.preserve_original_filename)
    }
    
    /// 递归加密文件夹（带文件名加密选项）
    pub fn encrypt_directory_with_options(
        dir_path: &Path, 
        key: &[u8; 32], 
        keep_original: bool,
        preserve_filename: bool,
    ) -> Result<()> {
        if !Self::is_safe_path(dir_path) {
            return Err(BjtError::FileError(format!(
                "路径不安全，跳过: {}",
                dir_path.display()
            )));
        }

        let mut success_count = 0;
        let mut error_count = 0;
        let mut total_bytes = 0;
        let mut visited = HashSet::new();

        println!("开始加密目录: {}", dir_path.display());

        // 先收集所有文件路径，避免在遍历时修改文件系统
        let mut file_entries = Vec::new();
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path().to_path_buf();
                    
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 跳过已经加密的文件（.leo扩展名）
                    if path.to_string_lossy().ends_with(".leo") {
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(&path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 获取文件大小
                    let file_size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    
                    file_entries.push((path, file_size));
                    total_bytes += file_size;
                }
                Err(e) => {
                    println!("❌ 遍历错误: {}", e);
                }
            }
        }

        if file_entries.is_empty() {
            println!("📁 目录中没有可加密的文件: {}", dir_path.display());
            return Ok(());
        }

        println!("文件数: {}, 总大小: {}", file_entries.len(), Self::format_bytes(total_bytes));
        println!("{}", "-".repeat(40));

        // 处理收集到的文件
        for (path, _file_size) in file_entries {
            // 处理文件（使用指定的文件名加密选项）
            match Self::process_file_with_options(&path, key, true, preserve_filename, keep_original) {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    error_count += 1;
                    println!("❌ 加密失败 {}: {}", path.display(), e);
                }
            }
        }

        println!("{}", "-".repeat(40));
        println!("加密完成:");
        println!("  ✅ 成功: {} 个文件", success_count);
        if error_count > 0 {
            println!("  ❌ 失败: {} 个文件", error_count);
        }
        println!("  📊 总大小: {}", Self::format_bytes(total_bytes));

        if error_count > 0 {
            Err(BjtError::FileError(format!(
                "加密完成，但有 {} 个文件失败",
                error_count
            )))
        } else {
            Ok(())
        }
    }

    /// 解密文件或文件夹
    pub fn decrypt_path_with_config(
        path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        config: &Config,
    ) -> Result<()> {
        if path.is_dir() {
            Self::decrypt_directory_with_progress(path, key, keep_original, config.show_progress)
        } else {
            Self::process_file(path, key, false, keep_original).map(|_| ())
        }
    }

    /// 处理单个文件（加密或解密）
    pub fn process_file(
        path: &Path,
        key: &[u8; 32],
        is_encrypt: bool,
        keep_original: bool,
    ) -> Result<()> {
        if is_encrypt {
            CryptoManager::encrypt_file(path, key, keep_original).map(|_| ())
        } else {
            CryptoManager::decrypt_file(path, key, keep_original).map(|_| ())
        }
    }
    
    /// 处理单个文件（带文件名加密选项）
    pub fn process_file_with_options(
        path: &Path,
        key: &[u8; 32],
        is_encrypt: bool,
        preserve_filename: bool,
        keep_original: bool,
    ) -> Result<()> {
        if is_encrypt {
            CryptoManager::encrypt_file_with_options(path, key, preserve_filename, keep_original).map(|_| ())
        } else {
            CryptoManager::decrypt_file(path, key, keep_original).map(|_| ())
        }
    }

    /// 过滤目录条目
    pub fn filter_entry(entry: &walkdir::DirEntry) -> bool {
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

    /// 格式化字节数为可读字符串
    fn format_bytes(bytes: u64) -> String {
        if bytes == 0 {
            return "0 B".to_string();
        }

        const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }

    /// 加密文件或文件夹（已弃用，使用 encrypt_path_with_config 代替）
    #[allow(dead_code)]
    pub fn encrypt_path(path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        // 从配置文件加载配置
        let config = Config::load().unwrap_or_default();
        Self::encrypt_path_with_config(path, key, keep_original, &config)
    }

    /// 解密文件或文件夹（已弃用，使用 decrypt_path_with_config 代替）
    #[allow(dead_code)]
    pub fn decrypt_path(path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        // 从配置文件加载配置
        let config = Config::load().unwrap_or_default();
        Self::decrypt_path_with_config(path, key, keep_original, &config)
    }

    /// 加密目录（已弃用，使用 encrypt_directory_with_progress 代替）
    #[allow(dead_code)]
    pub fn encrypt_directory(dir_path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        // 从配置文件加载配置
        let config = Config::load().unwrap_or_default();
        Self::encrypt_directory_with_progress(dir_path, key, keep_original, config.show_progress)
    }

    /// 解密目录（已弃用，使用 decrypt_directory_with_progress 代替）
    #[allow(dead_code)]
    pub fn decrypt_directory(dir_path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        // 从配置文件加载配置
        let config = Config::load().unwrap_or_default();
        Self::decrypt_directory_with_progress(dir_path, key, keep_original, config.show_progress)
    }
}