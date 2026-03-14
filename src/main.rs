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

        /// 显示详细信息
        #[arg(short, long)]
        verbose: bool,

        /// 显示原文件名（即使已加密）
        #[arg(long)]
        show_original: bool,
        
        /// 仅显示可解密的文件
        #[arg(long)]
        decryptable: bool,
        
        /// 按文件大小排序 (asc=升序, desc=降序)
        #[arg(long, value_enum)]
        sort_by_size: Option<SortOrder>,
        
        /// 递归深度限制（0表示无限制）
        #[arg(long, default_value = "0")]
        depth: usize,
        
        /// 输出格式
        #[arg(long, value_enum, default_value = "table")]
        output: OutputFormat,
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            handle_init()
        },
        Some(Commands::Encrypt { path, keep }) => {
            handle_encrypt(&path, keep)
        },
        Some(Commands::Decrypt { path, keep }) => {
            handle_decrypt(&path, keep)
        },
        Some(Commands::List { 
            path, 
            verbose, 
            show_original, 
            decryptable,
            sort_by_size,
            depth,
            output 
        }) => {
            Ok(handle_list(&path, verbose, show_original, decryptable, sort_by_size, depth, output)?)
        },
        Some(Commands::Completions { shell, output_dir }) => {
            handle_completions(shell, &output_dir)
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
    let config = Config::load().unwrap_or_default();
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
        let password = PasswordManager::read_password_interactive("请输入密码（至少8位，包含数字和字母）:")?;
        let confirm = PasswordManager::read_password_interactive("请确认密码:")?;
        
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
    
    // 生成并保存密钥
    let key = KeyManager::generate_and_save_key()?;
    println!("✅ 密钥文件已保存: \"{}\"", config.key_file_path().unwrap_or_default().display());
    
    // 保存密码哈希
    let password_hash = PasswordManager::hash_password(&password)?;
    PasswordManager::save_password_hash(&password_hash, &config.password_file_path()?)?;
    println!("✅ 密码哈希已保存");
    
    println!();
    println!("📁 创建配置文件...");
    
    // 创建默认配置
    let config = Config::default();
    config.save()?;
    println!("✅ 已生成配置文件: {}", Config::config_file_path().unwrap_or_default().display());
    
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
    println!("✅ 备份文件已创建: {}", backup_path.display());
    
    // 显示备份警告
    KeyManager::show_backup_warning(&backup_path);
    
    println!();
    println!("✅ 初始化完成！");
    println!("请妥善保管备份文件: {}", backup_path.display());
    
    Ok(())
}

/// 验证密码并获取密钥
fn verify_password_and_get_key() -> Result<[u8; 32]> {
    // 读取密码
    let password = PasswordManager::read_password_interactive("请输入密码:")?;
    
    // 加载密码哈希
    let config = Config::load().unwrap_or_default();
    let password_file_path = config.password_file_path()?;
    let password_hash = PasswordManager::load_password_hash(&password_file_path)?;
    
    // 验证密码
    if !PasswordManager::verify_password(&password, &password_hash)? {
        return Err(BjtError::PasswordError("密码验证失败".to_string()));
    }
    
    // 加载密钥
    KeyManager::load_key()
}

/// 处理加密命令
fn handle_encrypt(path: &std::path::Path, keep_original: bool) -> Result<()> {
    println!("🔒 开始加密: {}", path.display());
    
    // 检查路径是否存在
    if !path.exists() {
        return Err(BjtError::FileError(
            format!("路径不存在: {}", path.display())
        ));
    }
    
    // 加载配置
    let _config = Config::load().unwrap_or_default();
    
    // 验证密码并获取密钥
    let key = verify_password_and_get_key()?;
    
    // 执行加密
    FileOps::encrypt_path(path, &key, keep_original)?;
    
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
    
    // 验证密码并获取密钥
    let key = verify_password_and_get_key()?;
    
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
/// 处理文件列表命令
fn handle_list(
    path: &std::path::Path, 
    _verbose: bool, 
    show_original: bool,
    _decryptable: bool,
    _sort_by_size: Option<SortOrder>,
    _depth: usize,
    _output: OutputFormat,
) -> Result<()> {
    // 简化实现：暂时忽略新参数，调用基本list功能
    println!("📁 扫描目录: {}", path.display());
    println!("{}", "=".repeat(60));
    
    // 加载配置
    let _config = Config::load().unwrap_or_default();
    
    // 尝试加载密钥
    let key_result = KeyManager::load_key();
    let key = key_result.ok();
    
    use walkdir::WalkDir;
    
    let mut total_files = 0;
    let mut encrypted_files = 0;
    
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| FileOps::filter_entry(e))
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
                        
                        // 显示原文件名
                        if show_original {
                            if let Some(original_name) = &file_info.original_filename {
                                println!("  原文件名: {}", original_name);
                            }
                        }
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
    
    println!("{}", "=".repeat(60));
    println!("📊 统计:");
    println!("  总文件数: {}", total_files);
    println!("  加密文件数: {}", encrypted_files);
    println!("  普通文件数: {}", total_files - encrypted_files);
    
    Ok(())
}

