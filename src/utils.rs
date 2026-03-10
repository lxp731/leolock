use crate::errors::{BjtError, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

/// 工具函数集合
pub struct Utils;

#[allow(dead_code)]
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

    /// 解码base64盐值
    #[allow(dead_code)]
    pub fn decode_salt(salt_str: &str) -> Result<Vec<u8>> {
        STANDARD.decode(salt_str).map_err(|e| {
            BjtError::CryptoError(format!("解码base64盐值失败: {}", e))
        })
    }

    /// 检查文件是否可读
    #[allow(dead_code)]
    pub fn check_file_readable(path: &Path) -> Result<()> {
        fs::metadata(path).map_err(|e| {
            BjtError::FileError(format!("无法访问文件 {}: {}", path.display(), e))
        })?;

        // 尝试打开文件读取
        let file = fs::File::open(path);
        if file.is_err() {
            return Err(BjtError::FileError(format!(
                "无法读取文件 {}: 权限不足",
                path.display()
            )));
        }

        Ok(())
    }

    /// 检查文件是否可写
    #[allow(dead_code)]
    pub fn check_file_writable(path: &Path) -> Result<()> {
        // 如果文件不存在，检查父目录是否可写
        if !path.exists() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return Err(BjtError::FileError(format!(
                        "父目录不存在: {}",
                        parent.display()
                    )));
                }

                // 检查父目录是否可写
                let metadata = fs::metadata(parent)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let perms = metadata.permissions();
                    if perms.mode() & 0o200 == 0 {
                        return Err(BjtError::FileError(format!(
                            "父目录不可写: {}",
                            parent.display()
                        )));
                    }
                }
            }
        } else {
            // 文件存在，检查是否可写
            let metadata = fs::metadata(path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let perms = metadata.permissions();
                if perms.mode() & 0o200 == 0 {
                    return Err(BjtError::FileError(format!(
                        "文件不可写: {}",
                        path.display()
                    )));
                }
            }
        }

        Ok(())
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

    /// 计算文件哈希（SHA-256）
    #[allow(dead_code)]
    pub fn calculate_file_hash(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        
        io::copy(&mut file, &mut hasher)?;
        let hash = hasher.finalize();
        
        Ok(hex::encode(hash))
    }

    /// 显示进度条（简单版本）
    #[allow(dead_code)]
    pub fn show_progress(current: usize, total: usize, message: &str) {
        if total == 0 {
            return;
        }

        let percentage = (current as f32 / total as f32 * 100.0) as usize;
        let bar_width = 50;
        let filled = (percentage as f32 / 100.0 * bar_width as f32) as usize;
        let empty = bar_width - filled;

        print!("\r{}: [", message);
        for _ in 0..filled {
            print!("=");
        }
        for _ in 0..empty {
            print!(" ");
        }
        print!("] {}% ({}/{})", percentage, current, total);
        
        if current == total {
            println!();
        }
        
        io::stdout().flush().ok();
    }

    /// 格式化文件路径（相对路径显示）
    pub fn format_path(path: &Path) -> String {
        if let Ok(current_dir) = std::env::current_dir() {
            if let Ok(relative) = path.strip_prefix(&current_dir) {
                return format!("./{}", relative.display());
            }
        }
        path.display().to_string()
    }

    /// 验证文件后缀
    pub fn validate_file_extension(path: &Path, expected_ext: &str) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext == expected_ext)
            .unwrap_or(false)
    }

    /// 创建目录（如果不存在）
    pub fn ensure_dir_exists(path: &Path) -> Result<()> {
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| {
                BjtError::FileError(format!("创建目录失败 {}: {}", path.display(), e))
            })?;
        }
        Ok(())
    }

    /// 复制文件（带进度回调）
    pub fn copy_file_with_progress(
        src: &Path,
        dst: &Path,
        progress_callback: Option<Box<dyn Fn(u64, u64)>>,
    ) -> Result<()> {
        let metadata = fs::metadata(src)?;
        let total_size = metadata.len();

        let mut src_file = fs::File::open(src)?;
        let mut dst_file = fs::File::create(dst)?;

        let mut buffer = [0u8; 8192];
        let mut copied = 0u64;

        loop {
            let bytes_read = src_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            dst_file.write_all(&buffer[..bytes_read])?;
            copied += bytes_read as u64;

            if let Some(callback) = &progress_callback {
                callback(copied, total_size);
            }
        }

        dst_file.sync_all()?;
        Ok(())
    }
}