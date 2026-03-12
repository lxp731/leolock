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
use crate::utils::Utils;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "leolock",
    version = env!("CARGO_PKG_VERSION"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS"),
    long_about = "一个安全的文件加密解密工具，使用AES-256-GCM加密算法和Argon2id密码哈希",
    disable_help_flag = true,
    help_template = "\
{before-help}{name} {version}
{about}

{usage-heading} {usage}

{all-args}{after-help}"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 显示帮助信息
    #[arg(short, long, global = true)]
    help: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// 初始化工具（创建配置和密钥）
    Init,

    /// 从备份文件恢复密钥
    Recover {
        /// 备份文件路径
        #[arg(short, long)]
        backup: PathBuf,
    },

    /// 密码管理
    #[command(subcommand)]
    Password(PasswordCommands),

    /// 密钥管理
    #[command(subcommand)]
    Key(KeyCommands),

    /// 配置管理
    #[command(subcommand)]
    Config(ConfigCommands),

    /// 加密文件或文件夹
    Encrypt {
        /// 要加密的文件或文件夹路径
        path: PathBuf,

        /// 保留原始文件（不删除）
        #[arg(long)]
        keep_original: bool,
    },

    /// 解密文件或文件夹
    Decrypt {
        /// 要解密的文件或文件夹路径
        path: PathBuf,

        /// 保留加密文件（不删除）
        #[arg(long)]
        keep_original: bool,
    },

    /// 生成shell补全脚本
    Complete {
        /// Shell类型
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Subcommand)]
enum PasswordCommands {
    /// 修改密码
    Update,
}

#[derive(Subcommand)]
enum KeyCommands {
    /// 重新生成密钥（危险！）
    Update,
}

/// 配置管理命令
#[derive(Subcommand)]
enum ConfigCommands {
    /// 显示当前配置
    Show,
    /// 验证配置文件
    Validate,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // 显示帮助信息
    if cli.help && cli.command.is_none() {
        let mut cmd = Cli::command();
        cmd.print_help()?;
        return Ok(());
    }

    match cli.command {
        Some(Commands::Init) => Ok(handle_init()?),
        Some(Commands::Recover { backup }) => Ok(handle_recover(&backup)?),
        Some(Commands::Password(cmd)) => match cmd {
            PasswordCommands::Update => Ok(handle_password_update()?),
        },
        Some(Commands::Key(cmd)) => match cmd {
            KeyCommands::Update => Ok(handle_key_update()?),
        },
        Some(Commands::Config(cmd)) => match cmd {
            ConfigCommands::Show => Ok(handle_config_show()?),
            ConfigCommands::Validate => Ok(handle_config_validate()?),
        },
        Some(Commands::Encrypt { path, keep_original }) => Ok(handle_encrypt(&path, keep_original)?),
        Some(Commands::Decrypt { path, keep_original }) => Ok(handle_decrypt(&path, keep_original)?),
        Some(Commands::Complete { shell }) => {
            generate_completions(shell);
            Ok(())
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
    // 检查是否已初始化
    let config = Config::load()?;
    if config.is_initialized() {
        println!("⚠️  工具已经初始化");
        println!("配置目录: {:?}", Config::config_dir()?);
        return Ok(());
    }

    println!("🚀 开始初始化 leolock 工具...");

    // 1. 创建配置目录
    Config::create_config_dir()?;

    // 2. 交互式设置初始密码
    println!("\n🔐 设置初始密码");
    let password = loop {
        let pwd1 = PasswordManager::read_password_interactive("请输入密码（至少8位，包含数字和字母）")?;
        
        // 验证密码强度
        if let Err(e) = PasswordManager::validate_password_strength(&pwd1) {
            println!("❌ {}", e);
            continue;
        }

        let pwd2 = PasswordManager::read_password_interactive("请确认密码")?;
        
        if pwd1 != pwd2 {
            println!("❌ 两次输入的密码不一致，请重新输入");
            continue;
        }

        break pwd1;
    };

    // 3. 生成并保存密钥
    println!("\n🔑 生成加密密钥...");
    let key = KeyManager::generate_and_save_key()?;

    // 4. 创建配置文件
    println!("\n📁 创建配置文件...");
    let password_hash = PasswordManager::hash_password(&password)?;
    let salt = Utils::generate_salt()?;
    
    // 创建配置并保存
    let mut config = Config::default();
    config.password_hash = Some(password_hash.clone());
    config.salt = Some(salt);
    config.initialized = true;
    config.save()?;
    
    // 保存密码哈希到文件
    let password_file_path = config.password_file_path()?;
    PasswordManager::save_password_hash(&password_hash, &password_file_path)?;
    
    let config_dir = Config::get_default_config_dir()?;
    let config_path = config_dir.join("config.toml");
    
    println!("✅ 已生成配置文件: {}", config_path.display());
    println!("✅ 已保存密码哈希到: {}", password_file_path.display());
    println!("\n你可以编辑此文件来自定义设置:");
    println!("  - 危险路径列表 (forbidden_paths)");
    println!("  - 最大文件大小 (max_file_size)");
    println!("  - 显示进度 (show_progress)");
    println!("  - 默认扩展名 (default_extension)");
    println!("  - 密钥文件路径 (key_file_path)");
    println!("  - 是否保留原文件名 (preserve_original_filename)");

    // 5. 创建备份文件
    println!("\n💾 创建备份文件...");
    let backup_path = KeyManager::create_backup(&key, &password)?;

    // 6. 显示警告信息
    KeyManager::show_backup_warning(&backup_path);

    println!("\n✅ 初始化完成！");
    println!("请妥善保管备份文件: {}", backup_path.display());
    
    Ok(())
}

/// 处理恢复命令
fn handle_recover(backup_path: &std::path::Path) -> Result<()> {
    println!("🔄 从备份文件恢复密钥...");
    println!("备份文件: {}", backup_path.display());

    // 方案A：先验证操作密码，然后用操作密码解密备份
    // 但如果用户修改过密码，备份是用旧密码加密的
    // 所以我们需要：验证当前密码，然后询问备份密码
    
    // 1. 如果已初始化，验证当前操作密码
    let config = Config::load()?;
    if config.is_initialized() {
        println!("\n🔐 验证当前操作密码");
        let current_password = PasswordManager::read_password_interactive("请输入当前操作密码")?;
        
        // 验证密码（从密码文件）
        let password_file_path = config.password_file_path()?;
        PasswordManager::verify_and_get_password_hash(&current_password, &password_file_path)?;
        println!("✅ 当前操作密码验证通过");
    }

    // 2. 询问备份密码（可能是初始密码或旧密码）
    println!("\n🔐 输入备份文件密码");
    println!("提示：如果修改过密码，请输入修改前的密码（备份创建时的密码）");
    let backup_password = PasswordManager::read_password_interactive("请输入备份密码")?;

    // 3. 从备份恢复密钥
    let key = KeyManager::recover_from_backup(backup_path, &backup_password)?;

    // 4. 检查是否已存在配置
    if config.is_initialized() {
        println!("\n⚠️  检测到现有配置");
        if !Utils::confirm("这将覆盖现有密钥文件，继续吗？")? {
            println!("恢复操作已取消");
            return Ok(());
        }
    }

    // 5. 保存恢复的密钥
    KeyManager::save_key(&key)?;

    println!("\n✅ 密钥恢复成功！");
    println!("密钥文件已保存到: {:?}", Config::default_key_file_path()?);
    
    Ok(())
}

/// 处理密码修改命令
fn handle_password_update() -> Result<()> {
    // 检查是否已初始化
    let mut config = Config::load()?;
    if !config.is_initialized() {
        return Err(BjtError::ConfigError(
            "工具未初始化，请先运行 'leolock init'".to_string(),
        ));
    }

    // 交互式修改密码
    let (old_password, new_password) = PasswordManager::change_password_interactive()?;

    // 验证旧密码（从密码文件）
    let password_file_path = config.password_file_path()?;
    PasswordManager::verify_and_get_password_hash(&old_password, &password_file_path)?;

    // 更新密码哈希
    let new_password_hash = PasswordManager::hash_password(&new_password)?;
    PasswordManager::save_password_hash(&new_password_hash, &password_file_path)?;
    
    // 更新配置中的密码哈希（内存中）
    config.password_hash = Some(new_password_hash);
    config.salt = Some(Utils::generate_salt()?);

    println!("\n✅ 密码修改成功！");
    
    Ok(())
}

/// 处理密钥更新命令
fn handle_key_update() -> Result<()> {
    // 检查是否已初始化
    let config = Config::load()?;
    if !config.is_initialized() {
        return Err(BjtError::ConfigError(
            "工具未初始化，请先运行 'leolock init'".to_string(),
        ));
    }

    // 确认危险操作
    KeyManager::confirm_dangerous_operation()?;

    // 生成新密钥
    println!("\n🔑 生成新密钥...");
    let new_key = KeyManager::generate_and_save_key()?;

    // 询问是否创建备份
    println!("\n🔐 为新密钥创建备份");
    if Utils::confirm("是否创建新密钥的备份文件？")? {
        // 验证密码（备份需要用密码加密）
        println!("请输入密码以加密备份文件：");
        let password = PasswordManager::read_password_interactive("密码")?;
        
        // 验证密码（从密码文件）
        let password_file_path = config.password_file_path()?;
        PasswordManager::verify_and_get_password_hash(&password, &password_file_path)?;
        
        // 创建备份
        let backup_path = KeyManager::create_backup(&new_key, &password)?;
        println!("✅ 新备份已创建: {}", backup_path.display());
    } else {
        println!("⚠️  未创建备份，请务必手动备份密钥文件！");
    }

    println!("\n⚠️  重要提醒：");
    println!("1. 旧密钥加密的所有文件将无法解密！");
    println!("2. 旧的备份文件已失效！");
    println!("3. 请立即备份新密钥文件: {:?}", config.key_file_path()?);
    println!("4. 建议手动复制密钥文件到安全位置");
    
    Ok(())
}

/// 处理加密命令
fn handle_encrypt(path: &std::path::Path, keep_original: bool) -> Result<()> {
    // 检查是否已初始化
    let config = Config::load()?;
    if !config.is_initialized() {
        return Err(BjtError::ConfigError(
            "工具未初始化，请先运行 'leolock init'".to_string(),
        ));
    }

    // 交互式输入密码
    println!("🔐 加密操作需要验证密码");
    let password = PasswordManager::read_password_interactive("请输入密码")?;

    // 验证密码（从密码文件）
    let password_file_path = config.password_file_path()?;
    PasswordManager::verify_and_get_password_hash(&password, &password_file_path)?;

    // 加载密钥
    let key = KeyManager::load_key()?;

    // 加密文件或目录
    FileOps::encrypt_path(path, &key, keep_original)?;
    
    Ok(())
}

/// 处理解密命令
fn handle_decrypt(path: &std::path::Path, keep_original: bool) -> Result<()> {
    // 检查是否已初始化
    let config = Config::load()?;
    if !config.is_initialized() {
        return Err(BjtError::ConfigError(
            "工具未初始化，请先运行 'leolock init'".to_string(),
        ));
    }

    // 交互式输入密码
    println!("🔐 解密操作需要验证密码");
    let password = PasswordManager::read_password_interactive("请输入密码")?;

    // 验证密码（从密码文件）
    let password_file_path = config.password_file_path()?;
    PasswordManager::verify_and_get_password_hash(&password, &password_file_path)?;

    // 加载密钥
    let key = KeyManager::load_key()?;

    // 解密文件或目录
    FileOps::decrypt_path(path, &key, keep_original)?;
    
    Ok(())
}

fn generate_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let _script = clap_complete::generate(shell, &mut cmd, "leolock", &mut std::io::stdout());
}

/// 显示当前配置
fn handle_config_show() -> Result<()> {
    let config = config::Config::load()?;
    
    println!("当前配置:");
    println!("{}", "=".repeat(50));
    
    println!("危险路径列表 ({} 个):", config.forbidden_paths.len());
    for path in &config.forbidden_paths {
        println!("  - {}", path);
    }
    
    println!("\n最大文件大小: {} bytes ({:.2} GB)", 
        config.max_file_size,
        config.max_file_size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    
    println!("显示进度: {}", config.show_progress);
    println!("默认扩展名: {}", config.default_extension);
    println!("密钥文件路径: {}", config.key_file_path);
    println!("已初始化: {}", config.is_initialized());
    
    println!("\n配置文件搜索路径:");
    for path in config::Config::get_config_paths() {
        if path.exists() {
            println!("  ✓ {}", path.display());
        } else {
            println!("  - {}", path.display());
        }
    }
    
    Ok(())
}

/// 生成默认配置文件
/// 验证配置文件
fn handle_config_validate() -> Result<()> {
    println!("验证配置文件...");
    
    let config = config::Config::load()?;
    
    println!("✅ 配置文件格式正确");
    println!("✅ 包含 {} 个危险路径", config.forbidden_paths.len());
    println!("✅ 最大文件大小: {:.2} GB", 
        config.max_file_size as f64 / 1024.0 / 1024.0 / 1024.0
    );
    
    // 测试几个常见路径（基于实际配置检查）
    let test_paths = [
        "/bin/ls",
        "/etc/passwd", 
        "/tmp/test.txt",
        "/home/user/document.txt",
        "./test.txt",
        "/usr/bin/bash",
        "/var/log/syslog",
    ];
    
    println!("\n路径安全检查（基于当前配置）:");
    for path_str in test_paths {
        let path = std::path::Path::new(path_str);
        let is_safe = config.is_safe_path(path);
        let status = if is_safe { "✅ 安全" } else { "❌ 危险" };
        
        println!("{} {}", status, path_str);
    }
    
    Ok(())
}