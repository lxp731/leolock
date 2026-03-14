use crate::config::Config;
use crate::crypto::CryptoManager;
use crate::errors::{BjtError, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

// ========== 进度跟踪器模块（从progress.rs合并） ==========

/// 进度跟踪器
#[derive(Debug, Clone)]
pub struct ProgressTracker {
    inner: Arc<Mutex<ProgressInner>>,
}

#[derive(Debug)]
struct ProgressInner {
    /// 总任务数
    total_tasks: usize,
    /// 已完成任务数
    completed_tasks: usize,
    /// 总字节数
    total_bytes: u64,
    /// 已处理字节数
    processed_bytes: u64,
    /// 开始时间
    start_time: Instant,
    /// 最后更新时间
    last_update: Instant,
    /// 是否已完成
    finished: bool,
}

impl ProgressTracker {
    /// 创建新的进度跟踪器
    pub fn new(total_tasks: usize, total_bytes: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressInner {
                total_tasks,
                completed_tasks: 0,
                total_bytes,
                processed_bytes: 0,
                start_time: Instant::now(),
                last_update: Instant::now(),
                finished: false,
            })),
        }
    }

    /// 创建不确定总数的进度跟踪器
    #[allow(dead_code)]
    pub fn new_unknown() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ProgressInner {
                total_tasks: 0,
                completed_tasks: 0,
                total_bytes: 0,
                processed_bytes: 0,
                start_time: Instant::now(),
                last_update: Instant::now(),
                finished: false,
            })),
        }
    }

    /// 更新总任务数
    #[allow(dead_code)]
    pub fn set_total_tasks(&self, total_tasks: usize) {
        let mut inner = self.inner.lock().unwrap();
        inner.total_tasks = total_tasks;
    }

    /// 更新总字节数
    #[allow(dead_code)]
    pub fn set_total_bytes(&self, total_bytes: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.total_bytes = total_bytes;
    }

    /// 标记一个任务完成
    pub fn complete_task(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.completed_tasks += 1;
        inner.last_update = Instant::now();
    }

    /// 更新已处理字节数
    pub fn update_bytes(&self, bytes: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.processed_bytes += bytes;
        inner.last_update = Instant::now();
    }

    /// 标记所有任务完成
    pub fn finish(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.finished = true;
        inner.completed_tasks = inner.total_tasks;
        inner.processed_bytes = inner.total_bytes;
        inner.last_update = Instant::now();
    }

    /// 获取当前进度百分比（0-100）
    pub fn percentage(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        
        if inner.total_tasks > 0 && inner.total_bytes > 0 {
            // 使用任务和字节的加权平均
            let task_percent = if inner.total_tasks > 0 {
                (inner.completed_tasks as f64 / inner.total_tasks as f64) * 50.0
            } else {
                0.0
            };
            
            let byte_percent = if inner.total_bytes > 0 {
                (inner.processed_bytes as f64 / inner.total_bytes as f64) * 50.0
            } else {
                0.0
            };
            
            task_percent + byte_percent
        } else if inner.total_tasks > 0 {
            // 只有任务数
            (inner.completed_tasks as f64 / inner.total_tasks as f64) * 100.0
        } else if inner.total_bytes > 0 {
            // 只有字节数
            (inner.processed_bytes as f64 / inner.total_bytes as f64) * 100.0
        } else {
            // 未知进度
            0.0
        }
    }

    /// 获取处理速度（字节/秒）
    pub fn speed(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        let elapsed = inner.start_time.elapsed().as_secs_f64();
        
        if elapsed > 0.0 {
            inner.processed_bytes as f64 / elapsed
        } else {
            0.0
        }
    }

    /// 获取估计剩余时间（秒）
    pub fn remaining_time(&self) -> Option<f64> {
        let inner = self.inner.lock().unwrap();
        
        if inner.finished {
            return Some(0.0);
        }
        
        let speed = self.speed();
        if speed > 0.0 && inner.total_bytes > inner.processed_bytes {
            let remaining_bytes = inner.total_bytes - inner.processed_bytes;
            Some(remaining_bytes as f64 / speed)
        } else {
            None
        }
    }

    /// 获取进度条字符串
    pub fn progress_bar(&self, width: usize) -> String {
        let percent = self.percentage();
        let filled = (percent * width as f64 / 100.0).round() as usize;
        let empty = width.saturating_sub(filled);
        
        format!("[{}{}]", "=".repeat(filled), " ".repeat(empty))
    }

    /// 获取格式化状态字符串
    pub fn status_string(&self) -> String {
        let inner = self.inner.lock().unwrap();
        let percent = self.percentage();
        
        let mut parts = Vec::new();
        
        // 进度条
        parts.push(format!("{} {:.1}%", self.progress_bar(10), percent));
        
        // 任务计数
        if inner.total_tasks > 0 {
            parts.push(format!("({}/{})", inner.completed_tasks, inner.total_tasks));
        }
        
        // 处理速度
        let speed = self.speed();
        if speed > 0.0 {
            let speed_str = if speed < 1024.0 {
                format!("{:.1} B/s", speed)
            } else if speed < 1024.0 * 1024.0 {
                format!("{:.1} KB/s", speed / 1024.0)
            } else {
                format!("{:.1} MB/s", speed / (1024.0 * 1024.0))
            };
            parts.push(format!("速度: {}", speed_str));
        }
        
        // 剩余时间
        if let Some(remaining) = self.remaining_time() {
            if remaining > 0.0 {
                let remaining_str = if remaining < 60.0 {
                    format!("{:.0}秒", remaining)
                } else if remaining < 3600.0 {
                    format!("{:.0}分{:.0}秒", remaining / 60.0, remaining % 60.0)
                } else {
                    format!("{:.0}时{:.0}分", remaining / 3600.0, (remaining % 3600.0) / 60.0)
                };
                parts.push(format!("剩余: {}", remaining_str));
            }
        }
        
        parts.join(" ")
    }

    /// 检查是否已完成
    pub fn is_finished(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.finished
    }

    /// 获取已用时间（秒）
    #[allow(dead_code)]
    pub fn elapsed_time(&self) -> f64 {
        let inner = self.inner.lock().unwrap();
        inner.start_time.elapsed().as_secs_f64()
    }
}

/// 进度显示器
pub struct ProgressDisplay {
    tracker: ProgressTracker,
    last_display: Instant,
    min_interval: Duration,
}

impl ProgressDisplay {
    /// 创建新的进度显示器
    pub fn new(tracker: ProgressTracker) -> Self {
        Self {
            tracker,
            last_display: Instant::now(),
            min_interval: Duration::from_millis(200), // 最小显示间隔200ms
        }
    }

    /// 更新并显示进度（如果需要）
    pub fn update(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_display) >= self.min_interval {
            self.display();
            self.last_display = now;
        }
    }

    /// 强制显示当前进度
    pub fn display(&self) {
        if !self.tracker.is_finished() {
            print!("\r{}", self.tracker.status_string());
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
    }

    /// 完成并显示最终状态
    pub fn finish(&mut self) {
        self.tracker.finish();
        self.display();
        println!(); // 换行
    }
}

// ========== 文件操作模块 ==========

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

    /// 递归加密文件夹（带进度显示）
    pub fn encrypt_directory_with_progress(
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

        // 先统计文件总数和总大小
        let mut total_files = 0;
        let mut total_bytes = 0;
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() {
                        total_files += 1;
                        if let Ok(metadata) = entry.metadata() {
                            total_bytes += metadata.len();
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        if total_files == 0 {
            println!("📁 目录中没有可加密的文件: {}", dir_path.display());
            return Ok(());
        }

        println!("开始加密目录: {}", dir_path.display());
        println!("文件数: {}, 总大小: {}", total_files, Self::format_bytes(total_bytes));
        println!("{}", "-".repeat(40));

        // 创建进度跟踪器
        let tracker = ProgressTracker::new(total_files, total_bytes);
        let mut progress_display: Option<ProgressDisplay> = if show_progress {
            Some(ProgressDisplay::new(tracker.clone()))
        } else {
            None
        };

        let mut success_count = 0;
        let mut error_count = 0;
        let mut visited = HashSet::new();

        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    
                    if !entry.file_type().is_file() {
                        continue;
                    }

                    // 检查循环链接
                    if let Ok(canonical) = fs::canonicalize(path) {
                        if visited.contains(&canonical) {
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 处理文件
                    match Self::process_file(path, key, true, keep_original) {
                        Ok(_) => {
                            success_count += 1;
                            // 更新进度
                            tracker.complete_task();
                            if let Ok(metadata) = entry.metadata() {
                                tracker.update_bytes(metadata.len());
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            println!("\n❌ 加密失败 {}: {}", path.display(), e);
                        }
                    }

                    // 更新进度显示
                    if let Some(_) = progress_display.as_mut() {
                        progress_display.as_mut().unwrap().update();
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("\n❌ 遍历错误: {}", e);
                }
            }
        }

        // 完成进度显示
        if let Some(_) = progress_display.as_mut() {
            progress_display.as_mut().unwrap().finish();
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

    /// 递归解密文件夹（带进度显示）
    #[allow(dead_code)]
    pub fn decrypt_directory_with_progress(
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

        // 先统计文件总数和总大小
        let mut total_files = 0;
        let mut total_bytes = 0;
        
        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
        {
            match entry {
                Ok(entry) => {
                    if entry.file_type().is_file() && entry.path().to_string_lossy().ends_with(".leo") {
                        total_files += 1;
                        if let Ok(metadata) = entry.metadata() {
                            total_bytes += metadata.len();
                        }
                    }
                }
                Err(_) => continue,
            }
        }

        if total_files == 0 {
            println!("📁 目录中没有加密文件 (.leo): {}", dir_path.display());
            return Ok(());
        }

        println!("开始解密目录: {}", dir_path.display());
        println!("加密文件数: {}, 总大小: {}", total_files, Self::format_bytes(total_bytes));
        println!("{}", "-".repeat(40));

        // 创建进度跟踪器
        let tracker = ProgressTracker::new(total_files, total_bytes);
        let mut progress_display: Option<ProgressDisplay> = if show_progress {
            Some(ProgressDisplay::new(tracker.clone()))
        } else {
            None
        };

        let mut success_count = 0;
        let mut skip_count = 0;
        let mut error_count = 0;
        let mut visited = HashSet::new();

        for entry in WalkDir::new(dir_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(Self::filter_entry)
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
                            continue;
                        }
                        visited.insert(canonical);
                    }

                    // 处理文件
                    match Self::process_file(path, key, false, keep_original) {
                        Ok(_) => {
                            success_count += 1;
                            // 更新进度
                            tracker.complete_task();
                            if let Ok(metadata) = entry.metadata() {
                                tracker.update_bytes(metadata.len());
                            }
                        }
                        Err(e) => {
                            error_count += 1;
                            println!("\n❌ 解密失败 {}: {}", path.display(), e);
                        }
                    }

                    // 更新进度显示
                    if let Some(_) = progress_display.as_mut() {
                        progress_display.as_mut().unwrap().update();
                    }
                }
                Err(e) => {
                    error_count += 1;
                    println!("\n❌ 遍历错误: {}", e);
                }
            }
        }

        // 完成进度显示
        if let Some(_) = progress_display.as_mut() {
            progress_display.as_mut().unwrap().finish();
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

    /// 递归加密文件夹（使用配置中的进度设置）
    pub fn encrypt_directory(dir_path: &Path, key: &[u8; 32], keep_original: bool) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        Self::encrypt_directory_with_progress(dir_path, key, keep_original, config.show_progress)
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
            .filter_entry(Self::filter_entry)
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
            CryptoManager::encrypt_file(path, key, keep_original)?;
            Ok(())
        } else {
            CryptoManager::decrypt_file(path, key, keep_original)?;
            Ok(())
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

    /// 格式化字节数为可读字符串
    fn format_bytes(bytes: u64) -> String {
        if bytes == 0 {
            "0 B".to_string()
        } else if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

}