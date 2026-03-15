mod config;
mod crypto;
mod errors;
mod fileops;
mod keymgmt;
mod password;
mod utils;

use crate::config::Config;
use crate::errors::{BjtError, Result};
use crate::fileops::FileOps;
use crate::keymgmt::KeyManager;
use crate::password::PasswordManager;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use base64::Engine;

use std::path::{Path, PathBuf};

/// 排序顺序
#[derive(clap::ValueEnum, Clone, Debug)]
enum SortOrder {
    /// 升序排列
    Asc,
    /// 降序排列
    Desc,
}

/// 输出格式
#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// 表格格式
    Table,
    /// JSON格式
    Json,
    /// 简单列表
    Simple,
}

#[derive(Parser)]
#[command(
    name = "leolock",
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS"),
    long_about = "一个安全的文件加密解密工具，使用AES-256-GCM加密算法和Argon2id密码哈希。\n\n初始化: leolock init\n加密文件: leolock encrypt <文件或目录>\n解密文件: leolock decrypt <文件或目录>",
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化工具（首次使用前必须运行）
    Init,
    
    /// 加密文件或目录
    Encrypt {
        /// 要加密的路径（文件或目录）
        path: PathBuf,
        
        /// 是否保留原文件（默认删除原文件）
        #[arg(short, long)]
        keep: bool,
        
        /// 快速模式：不加密文件名，仅加密文件内容
        #[arg(short = 'F', long)]
        fast: bool,
    },
    
    /// 解密文件或目录
    Decrypt {
        /// 要解密的路径（文件或目录）
        path: PathBuf,
        
        /// 是否保留加密文件（默认删除加密文件）
        #[arg(short, long)]
        keep: bool,
    },
    
    /// 列出加密文件信息
    List {
        /// 要列出的路径（文件或目录）
        path: PathBuf,

        /// 显示原文件名（需要密码验证）
        #[arg(long)]
        show_original: bool,
        
        /// 按文件大小排序 (asc=升序, desc=降序)
        #[arg(long, value_enum)]
        sort_by_size: Option<SortOrder>,
    },
    
    /// 生成shell补全脚本
    Completions {
        /// 要生成的shell类型
        #[arg(value_enum)]
        shell: Shell,
        
        /// 输出目录（默认：当前目录）
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,
    },
    
    /// 从备份文件恢复密钥
    Recover {
        /// 备份文件路径
        #[arg(long)]
        backup: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            handle_init()
        },
        Some(Commands::Encrypt { path, keep, fast }) => {
            handle_encrypt(&path, keep, fast)
        },
        Some(Commands::Decrypt { path, keep }) => {
            handle_decrypt(&path, keep)
        },
        Some(Commands::List { 
            path, 
            show_original, 
            sort_by_size,
        }) => {
            Ok(handle_list(&path, show_original, sort_by_size)?)
        },
        Some(Commands::Completions { shell, output_dir }) => {
            handle_completions(shell, &output_dir)
        },
        Some(Commands::Recover { backup }) => {
            handle_recover(&backup)
        },
        None => {
            let mut cmd = Cli::command();
            cmd.print_help()?;
            Ok(())
        }
    }
}

/// 处理初始化命令
fn handle_init() -> Result<()> {
    println!("🚀 开始初始化 leolock 工具...");
    println!();
    
    // 检查是否已初始化
    let mut config = Config::load().unwrap_or_default();
    if config.is_initialized() {
        println!("⚠️  工具已经初始化过");
        return Ok(());
    }
    
    // 创建配置目录
    let config_dir = Config::config_dir()?;
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)?;
        println!("📁 创建配置目录: {}", config_dir.display());
    }
    
    println!("🔐 设置初始密码");
    
    // 读取并验证密码
    let password = loop {
        let password = PasswordManager::read_password_interactive("请输入密码（至少8位，包含数字和字母）")?;
        let confirm = PasswordManager::read_password_interactive("请确认密码")?;
        
        if password != confirm {
            println!("❌ 两次输入的密码不一致，请重新输入");
            continue;
        }
        
        // 验证密码强度
        if let Err(e) = PasswordManager::validate_password_strength(&password) {
            println!("❌ {}", e);
            continue;
        }
        
        break password;
    };
    
    println!();
    println!("🔑 生成加密密钥...");
    
    // 生成随机盐值
    use getrandom::getrandom;
    let mut salt = [0u8; 16];
    getrandom(&mut salt).map_err(|e| BjtError::CryptoError(format!("生成盐值失败: {}", e)))?;
    let salt_base64 = base64::engine::general_purpose::STANDARD.encode(salt);
    
    // 使用密码派生主密钥
    let key = crate::crypto::CryptoManager::derive_key_from_password(&password, &salt)?;
    
    // 保存密钥和盐值
    KeyManager::save_key(&key)?;
    println!("✅ 主密钥已保存");
    
    // 保存盐值到配置
    config.salt = Some(salt_base64);
    config.initialized = true;
    
    println!();
    println!("📁 创建配置文件...");
    
    // 保存配置（包含盐值和初始化状态）
    config.save()?;
    let config_path = Config::config_file_path().unwrap_or_default();
    println!("✅ 已生成配置文件: {}", config_path.display());
    
    // 设置配置文件权限（仅所有者可读写）
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&config_path)?.permissions();
    perms.set_mode(0o600); // rw-------
    std::fs::set_permissions(&config_path, perms)?;
    println!("🔒 已设置配置文件权限: 600（仅所有者可读写）");
    
    println!();
    println!("你可以编辑此文件来自定义设置:");
    println!(" - 危险路径列表 (forbidden_paths)");
    println!(" - 最大文件大小 (max_file_size)");
    println!(" - 显示进度 (show_progress)");
    println!(" - 默认扩展名 (default_extension)");
    println!(" - 密钥文件路径 (key_file_path)");
    
    println!();
    println!("💾 创建备份文件...");
    
    // 创建备份文件
    let backup_path = KeyManager::create_backup(&key, &password)?;
    
    // 显示备份警告
    KeyManager::show_backup_warning(&backup_path);
    
    println!();
    println!("✅ 初始化完成！");
    println!("请妥善保管备份文件: {}", backup_path.display());
    
    Ok(())
}

/// 验证密码并获取密钥
fn get_key_from_password() -> Result<[u8; 32]> {
    // 读取密码
    let password = PasswordManager::read_password_interactive("请输入密码")?;
    
    // 加载配置和盐值
    let config = Config::load().unwrap_or_default();
    
    // 检查是否已初始化
    if !config.initialized {
        return Err(BjtError::PasswordError("工具未初始化，请先运行 'leolock init'".to_string()));
    }
    
    // 检查配置文件安全性
    check_config_security()?;
    
    // 获取盐值
    let salt_base64 = config.salt.ok_or_else(|| {
        BjtError::PasswordError("配置中缺少盐值，请重新初始化".to_string())
    })?;
    
    let salt = base64::engine::general_purpose::STANDARD.decode(salt_base64).map_err(|e| {
        BjtError::PasswordError(format!("解码盐值失败: {}", e))
    })?;
    
    // 使用密码派生密钥
    crate::crypto::CryptoManager::derive_key_from_password(&password, &salt)
}

/// 检查配置文件安全性
fn check_config_security() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    
    if let Ok(config_path) = Config::config_file_path() {
        if config_path.exists() {
            let metadata = std::fs::metadata(&config_path)?;
            let permissions = metadata.permissions();
            let mode = permissions.mode();
            
            // 检查是否其他用户可读 (---rwxrwx)
            if mode & 0o077 != 0 {
                println!("⚠️  警告: 配置文件权限过于宽松 ({:o})", mode & 0o777);
                println!("   建议运行: chmod 600 {}", config_path.display());
                println!("   或重新初始化: leolock init");
            }
        }
    }
    
    Ok(())
}

/// 处理加密命令
fn handle_encrypt(path: &std::path::Path, keep_original: bool, fast: bool) -> Result<()> {
    if fast {
        println!("🔒 开始加密: {} (快速模式)", path.display());
        println!("  模式: 仅加密文件内容，不加密文件名");
        println!("  优势: 速度更快，适合大文件");
        println!("  注意: 原始文件名将保持可读");
    } else {
        println!("🔒 开始加密: {} (完全模式)", path.display());
        println!("  模式: 加密文件内容和文件名");
        println!("  优势: 最高安全性，隐藏文件信息");
    }
    
    // 检查路径是否存在
    if !path.exists() {
        return Err(BjtError::FileError(
            format!("路径不存在: {}", path.display())
        ));
    }
    
    // 加载配置
    let mut config = Config::load().unwrap_or_default();
    
    // 临时覆盖配置中的 preserve_original_filename 设置
    let original_preserve_setting = config.preserve_original_filename;
    config.preserve_original_filename = fast; // fast=true 表示保留文件名
    
    // 从密码获取密钥
    let key = get_key_from_password()?;
    
    // 执行加密（使用临时配置）
    FileOps::encrypt_path_with_config(path, &key, keep_original, &config)?;
    
    // 恢复原始配置（不保存到文件）
    config.preserve_original_filename = original_preserve_setting;
    
    println!("✅ 加密完成！");
    Ok(())
}

/// 处理解密命令
fn handle_decrypt(path: &std::path::Path, keep_encrypted: bool) -> Result<()> {
    println!("🔓 开始解密: {}", path.display());
    
    // 检查路径是否存在
    if !path.exists() {
        return Err(BjtError::FileError(
            format!("路径不存在: {}", path.display())
        ));
    }
    
    // 加载配置
    let _config = Config::load().unwrap_or_default();
    
    // 从密码获取密钥
    let key = get_key_from_password()?;
    
    // 执行解密
    FileOps::decrypt_path(path, &key, keep_encrypted)?;
    
    println!("✅ 解密完成！");
    Ok(())
}

/// 处理补全脚本生成
fn handle_completions(shell: Shell, output_dir: &Path) -> Result<()> {
    println!("🔧 生成 {} shell 补全脚本...", shell);
    
    // 确保输出目录存在
    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }
    
    let mut cmd = Cli::command();
    let app_name = cmd.get_name().to_string();
    
    match shell {
        Shell::Bash => {
            let output_path = output_dir.join(format!("{}.bash", app_name));
            let mut file = std::fs::File::create(&output_path)?;
            generate(Shell::Bash, &mut cmd, &app_name, &mut file);
            println!("✅ Bash 补全脚本已生成: {}", output_path.display());
        }
        Shell::Zsh => {
            let output_path = output_dir.join(format!("_{}", app_name));
            let mut file = std::fs::File::create(&output_path)?;
            generate(Shell::Zsh, &mut cmd, &app_name, &mut file);
            println!("✅ Zsh 补全脚本已生成: {}", output_path.display());
        }
        Shell::Fish => {
            let output_path = output_dir.join(format!("{}.fish", app_name));
            let mut file = std::fs::File::create(&output_path)?;
            generate(Shell::Fish, &mut cmd, &app_name, &mut file);
            println!("✅ Fish 补全脚本已生成: {}", output_path.display());
        }
        Shell::PowerShell => {
            let output_path = output_dir.join(format!("_{}.ps1", app_name));
            let mut file = std::fs::File::create(&output_path)?;
            generate(Shell::PowerShell, &mut cmd, &app_name, &mut file);
            println!("✅ PowerShell 补全脚本已生成: {}", output_path.display());
        }
        Shell::Elvish => {
            let output_path = output_dir.join(format!("{}.elv", app_name));
            let mut file = std::fs::File::create(&output_path)?;
            generate(Shell::Elvish, &mut cmd, &app_name, &mut file);
            println!("✅ Elvish 补全脚本已生成: {}", output_path.display());
        }
        _ => {
            println!("⚠️  不支持的shell类型: {:?}", shell);
            println!("   支持的类型: bash, zsh, fish, powershell, elvish");
        }
    }
    
    println!();
    println!("📖 使用说明:");
    println!("1. 将生成的补全脚本移动到对应的shell配置目录");
    println!("2. 重新加载shell配置");
    println!();
    println!("💡 示例 (Bash):");
    println!("  sudo mv leolock.bash /usr/share/bash-completion/completions/leolock");
    println!("  source ~/.bashrc");
    
    Ok(())
}

/// 文件信息结构
#[derive(Debug, serde::Serialize)]
#[allow(dead_code)]
struct FileInfo {
    path: String,
    version: u8,
    encrypted_size: u64,
    decryptable: bool,
    original_filename: Option<String>,
    is_encrypted: bool,
}

/// 处理文件列表命令
fn handle_list(
    path: &std::path::Path, 
    show_original: bool,
    sort_by_size: Option<SortOrder>,
) -> Result<()> {
    println!("📁 扫描目录: {}", path.display());
    println!("{}", "=".repeat(60));
    
    // 如果要求显示原文件名，需要密码验证
    let key = if show_original {
        println!("🔐 显示原文件名需要密码验证");
        match get_key_from_password() {
            Ok(key) => {
                println!("✅ 密码验证成功");
                Some(key)
            }
            Err(e) => {
                return Err(BjtError::PasswordError(format!("密码验证失败: {}", e)));
            }
        }
    } else {
        // 不显示原文件名时，尝试从密钥文件加载（向后兼容）
        KeyManager::load_key().ok()
    };
    
    use walkdir::WalkDir;
    
    let mut file_infos = Vec::new();
    let mut total_files = 0;
    let mut encrypted_files = 0;
    
    // 收集所有加密文件信息
    for entry in WalkDir::new(path)
        .follow_links(false)
        .max_depth(1)  // 只扫描当前目录
        .into_iter()
    {
        match entry {
            Ok(entry) => {
                let file_path = entry.path();
                
                if !entry.file_type().is_file() {
                    continue;
                }
                
                total_files += 1;
                
                // 检查是否是加密文件
                let is_leo_file = file_path.extension()
                    .map(|ext| ext == "leo")
                    .unwrap_or(false);
                
                if !is_leo_file {
                    continue;
                }
                
                encrypted_files += 1;
                
                // 获取文件信息
                match crate::crypto::CryptoManager::get_file_info(file_path, key.as_ref()) {
                    Ok(file_info) => {
                        file_infos.push(file_info);
                    }
                    Err(e) => {
                        println!("❌ 无法读取文件信息 {}: {}", file_path.display(), e);
                    }
                }
            }
            Err(e) => {
                println!("⚠️  无法访问条目: {}", e);
            }
        }
    }
    
    // 按文件大小排序
    if let Some(order) = sort_by_size {
        match order {
            SortOrder::Asc => {
                file_infos.sort_by(|a, b| a.encrypted_size.cmp(&b.encrypted_size));
                println!("📊 按文件大小升序排列");
            }
            SortOrder::Desc => {
                file_infos.sort_by(|a, b| b.encrypted_size.cmp(&a.encrypted_size));
                println!("📊 按文件大小降序排列");
            }
        }
    }
    
    // 显示文件信息
    for file_info in &file_infos {
        let path_str = file_info.path.display();
        let version_str = format!("v{}", file_info.version);
        
        // 基本信息
        let mut info_line = format!("📄 {} [{}]", path_str, version_str);
        
        // 添加文件大小
        let size_str = if file_info.encrypted_size == 0 {
            "空文件".to_string()
        } else if file_info.encrypted_size < 1024 {
            format!("{} B", file_info.encrypted_size)
        } else if file_info.encrypted_size < 1024 * 1024 {
            format!("{:.1} KB", file_info.encrypted_size as f64 / 1024.0)
        } else if file_info.encrypted_size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", file_info.encrypted_size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", file_info.encrypted_size as f64 / (1024.0 * 1024.0 * 1024.0))
        };
        info_line.push_str(&format!(" ({})", size_str));
        
        // 添加解密状态
        let decrypt_status = if file_info.decryptable {
            "🔓"
        } else {
            "🔒"
        };
        info_line.push_str(&format!(" {}", decrypt_status));
        
        println!("{}", info_line);
        
        // 显示原文件名（如果要求且可解密）
        if show_original {
            if let Some(original_name) = &file_info.original_filename {
                println!("  原文件名: {}", original_name);
            }
        }
    }
    
    println!("{}", "=".repeat(60));
    println!("📊 统计:");
    println!("  总文件数: {}", total_files);
    println!("  加密文件数: {}", encrypted_files);
    println!("  普通文件数: {}", total_files - encrypted_files);
    
    Ok(())
}

/// 处理恢复命令
fn handle_recover(backup_path: &Path) -> Result<()> {
    println!("🔄 从备份文件恢复密钥");
    println!("备份文件: {}", backup_path.display());
    
    // 检查备份文件是否存在
    if !backup_path.exists() {
        return Err(crate::errors::BjtError::BackupError(
            format!("备份文件不存在: {}", backup_path.display())
        ));
    }
    
    // 读取密码
    let password = crate::password::PasswordManager::read_password_interactive("请输入备份密码")?;
    
    // 从备份恢复密钥
    let key = crate::keymgmt::KeyManager::recover_from_backup(backup_path, &password)?;
    
    // 保存恢复的密钥
    crate::keymgmt::KeyManager::save_key(&key)?;
    
    println!("✅ 密钥恢复成功！");
    println!("  已从备份文件恢复密钥并保存到配置文件");
    println!("  现在可以使用新密钥进行加密/解密操作");
    
    Ok(())
}

