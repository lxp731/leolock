use crate::config::Config;
use crate::crypto::CryptoManager;
use crate::errors::{BjtError, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// 文件操作管理器
pub struct FileOps;

impl FileOps {
    /// 加密文件或目录（主入口）
    pub fn encrypt_path_with_config(
        path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        config: &Config,
    ) -> Result<()> {
        if path.is_dir() {
            Self::encrypt_directory(
                path,
                key,
                keep_original,
                config.program.preserve_original_filename,
                config.program.show_progress,
            )
        } else {
            CryptoManager::encrypt_file_v2(
                path,
                key,
                config.program.preserve_original_filename,
                keep_original,
            )
            .map(|_| ())
        }
    }

    /// 解密文件或目录（主入口）
    pub fn decrypt_path_with_config(
        path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        config: &Config,
    ) -> Result<()> {
        if path.is_dir() {
            Self::decrypt_directory(path, key, keep_original, config.program.show_progress)
        } else {
            CryptoManager::decrypt_file_v2(path, key, keep_original).map(|_| ())
        }
    }

    /// 检查路径是否安全
    pub fn is_safe_path(path: &Path) -> bool {
        match Config::load() {
            Ok(config) => config.is_safe_path(path),
            Err(_) => Config::default().is_safe_path(path),
        }
    }

    /// 过滤目录条目（跳过隐藏文件和危险路径）
    pub fn filter_entry(entry: &walkdir::DirEntry) -> bool {
        let path = entry.path();
        if !Self::is_safe_path(path) {
            return false;
        }
        if entry.file_name().to_string_lossy().starts_with('.') {
            return false;
        }
        true
    }

    // ─── 内部实现 ───────────────────────────────────────────────

    fn encrypt_directory(
        dir_path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        preserve_filename: bool,
        show_progress: bool,
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
                    if path.to_string_lossy().ends_with(".leo") {
                        continue;
                    }
                    if let Ok(canonical) = fs::canonicalize(&path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }
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

        println!(
            "文件数: {}, 总大小: {}",
            file_entries.len(),
            format_bytes(total_bytes)
        );
        println!("{}", "-".repeat(40));

        let pb = maybe_progress_bar(show_progress, file_entries.len() as u64);

        for (path, _file_size) in file_entries {
            match CryptoManager::encrypt_file_v2(&path, key, preserve_filename, keep_original) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    if let Some(ref pb) = pb {
                        pb.suspend(|| println!("❌ 加密失败 {}: {}", path.display(), e));
                    } else {
                        println!("❌ 加密失败 {}: {}", path.display(), e);
                    }
                }
            }
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("加密完成");
        }

        print_summary("加密", success_count, error_count, total_bytes);
        if error_count > 0 {
            Err(BjtError::FileError(format!(
                "加密完成，但有 {} 个文件失败",
                error_count
            )))
        } else {
            Ok(())
        }
    }

    fn decrypt_directory(
        dir_path: &Path,
        key: &[u8; 32],
        keep_original: bool,
        show_progress: bool,
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
                    if !path.to_string_lossy().ends_with(".leo") {
                        skip_count += 1;
                        continue;
                    }
                    if let Ok(canonical) = fs::canonicalize(&path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }
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

        println!(
            "加密文件数: {}, 总大小: {}",
            file_entries.len(),
            format_bytes(total_bytes)
        );
        println!("{}", "-".repeat(40));

        let pb = maybe_progress_bar(show_progress, file_entries.len() as u64);

        for (path, _file_size) in file_entries {
            match CryptoManager::decrypt_file_v2(&path, key, keep_original) {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error_count += 1;
                    if let Some(ref pb) = pb {
                        pb.suspend(|| println!("❌ 解密失败 {}: {}", path.display(), e));
                    } else {
                        println!("❌ 解密失败 {}: {}", path.display(), e);
                    }
                }
            }
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
        }

        if let Some(pb) = pb {
            pb.finish_with_message("解密完成");
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
        println!("  📊 总大小: {}", format_bytes(total_bytes));

        if error_count > 0 {
            Err(BjtError::FileError(format!(
                "解密完成，但有 {} 个文件失败",
                error_count
            )))
        } else {
            Ok(())
        }
    }
}

// ─── 辅助函数 ──────────────────────────────────────────────────

fn maybe_progress_bar(show: bool, total: u64) -> Option<ProgressBar> {
    if !show {
        return None;
    }
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} 文件 ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    Some(pb)
}

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

fn print_summary(operation: &str, success: u32, errors: u32, total_bytes: u64) {
    println!("{}", "-".repeat(40));
    println!("{operation}完成:");
    println!("  ✅ 成功: {success} 个文件");
    if errors > 0 {
        println!("  ❌ 失败: {errors} 个文件");
    }
    println!("  📊 总大小: {}", format_bytes(total_bytes));
}
