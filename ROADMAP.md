# leolock 项目路线图

## 概述

leolock 是一个安全的文件加密解密工具，使用AES-256-GCM加密算法和Argon2id密码哈希。本路线图规划了从基础功能到企业级特性的发展路径。

## 核心原则

1. **安全性优先**：所有功能必须符合安全最佳实践
2. **向后兼容**：新版本必须能处理旧版本创建的文件
3. **渐进增强**：每个版本都保持可用性和稳定性
4. **用户为中心**：功能设计基于实际用户需求

## 路线图总览

### 阶段一：核心功能完善（版本 1.1.0）
**目标：** 完善基础加密功能，增加文件名加密

### 阶段二：用户体验提升（版本 1.2.0）
**目标：** 改进交互体验，增加实用功能

### 阶段三：性能优化（版本 1.3.0）
**目标：** 提升大规模文件处理能力

### 阶段四：企业级功能（版本 2.0.0）
**目标：** 满足专业和企业用户需求

### 阶段五：生态系统集成（版本 2.1.0+）
**目标：** 与其他工具和平台集成

---

## 详细路线图

### 阶段一：核心功能完善 ✅ 已完成
**版本目标：1.1.0** ✅ **已实现（当前版本 1.0.2）**

#### 1.1 文件名加密功能（P0） ✅ 已完成

**产品需求：**
- ✅ 用户可选择是否加密文件名（通过 `preserve_original_filename` 配置）
- ✅ 默认加密文件名以增强隐私保护（默认 `false`）
- ✅ 解密时自动恢复原文件名

**技术实现细节：**

##### 1.1.1 配置文件扩展 ✅ 已实现
```rust
// config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // ... 现有字段
    
    /// 是否保留原文件名（false=加密文件名，true=保留文件名）
    pub preserve_original_filename: bool,
    
    /// 加密文件格式版本
    pub file_format_version: u8,
    
    /// 密码文件位置
    pub password_file_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // ... 现有默认值
            preserve_original_filename: false,  // 默认加密文件名
            file_format_version: 2,             // 新文件格式版本
            password_file_path: "~/.config/leolock/password.bin".to_string(),
        }
    }
}
```

##### 1.1.2 新文件格式设计 ✅ 已实现
```
新版加密文件格式（版本2）：
+----------------+----------------+----------------+----------------+
| 魔术字节       | 版本号         | 文件名元数据   | 文件内容       |
| (4字节)        | (1字节)        | (变长)         | (变长)         |
+----------------+----------------+----------------+----------------+

魔术字节：0x4C 0x45 0x4F 0x32 ("LEO2")
版本号：2

文件名元数据结构：
+----------------+----------------+----------------+----------------+
| 元数据长度     | 加密的文件名   | 文件名认证标签 | 保留字段       |
| (4字节)        | (变长)         | (16字节)       | (4字节)        |
+----------------+----------------+----------------+----------------+

文件内容结构（保持不变）：
+----------------+----------------+----------------+
| nonce          | 加密的内容     | 内容认证标签   |
| (12字节)       | (变长)         | (16字节)       |
+----------------+----------------+----------------+
```

**实际实现：**
- ✅ `FileHeader` 结构体定义在 `crypto.rs`
- ✅ 魔术字节：`b"LEO2"`
- ✅ 版本号：`2`
- ✅ 文件名元数据长度：`u32` 小端字节序
- ✅ 加密的文件名：AES-256-GCM 加密的原文件名
- ✅ 文件内容：保持原有 AES-256-GCM 加密结构

##### 1.1.3 文件名加密实现 ✅ 已实现
```rust
// crypto.rs
impl CryptoManager {
    /// 加密文件名 ✅ 已实现
    pub fn encrypt_filename(filename: &str, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
        // 使用独立的nonce加密文件名
        let mut filename_nonce = [0u8; NONCE_SIZE];
        getrandom(&mut filename_nonce)?;
        
        let cipher = Self::create_cipher(key)?;
        let nonce = Nonce::from_slice(&filename_nonce);
        
        let ciphertext = cipher.encrypt(nonce, filename.as_bytes())?;
        
        // 组合：nonce + ciphertext
        let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        result.extend_from_slice(&filename_nonce);
        result.extend_from_slice(&ciphertext);
        
        Ok(result)
    }
    
    /// 解密文件名 ✅ 已实现
    pub fn decrypt_filename(encrypted_filename: &[u8], key: &[u8; KEY_SIZE]) -> Result<String> {
        if encrypted_filename.len() < NONCE_SIZE {
            return Err(BjtError::CryptoError("加密文件名数据太短".to_string()));
        }
        
        let cipher = Self::create_cipher(key)?;
        let nonce = Nonce::from_slice(&encrypted_filename[..NONCE_SIZE]);
        let ciphertext = &encrypted_filename[NONCE_SIZE..];
        
        let plaintext = cipher.decrypt(nonce, ciphertext)?;
        String::from_utf8(plaintext).map_err(|e| {
            BjtError::CryptoError(format!("文件名解码失败: {}", e))
        })
    }
}
```

**额外实现的功能：**
- ✅ `encrypt_file_v2()`: 新版加密函数，支持文件名加密
- ✅ `decrypt_file_v2()`: 新版解密函数，支持文件名恢复
- ✅ `detect_file_version()`: 自动检测文件版本
- ✅ `generate_filename_hash()`: 生成哈希显示文件名（`utils.rs`）
- ✅ `get_display_filename()`: 根据配置生成显示文件名

##### 1.1.4 向后兼容处理
```rust
// 文件版本检测
fn detect_file_version(file_path: &Path) -> Result<u8> {
    let mut file = fs::File::open(file_path)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)?;
    
    if &magic == b"LEO2" {
        // 读取版本号
        let mut version = [0u8; 1];
        file.read_exact(&mut version)?;
        Ok(version[0])
    } else {
        // 旧版文件（无魔术字节）
        Ok(1)
    }
}
```

#### 1.2 基础批量优化（P1）

**技术实现细节：**

##### 1.2.1 简单线程池
```rust
// utils.rs
use std::sync::{Arc, Mutex};
use std::thread;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: crossbeam_channel::Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        
        let (sender, receiver) = crossbeam_channel::unbounded();
        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);
        
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        
        ThreadPool { workers, sender }
    }
    
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}
```

##### 1.2.2 批量处理管理器
```rust
// fileops.rs
pub struct BatchProcessor {
    config: Config,
    thread_pool: ThreadPool,
    progress_tracker: Arc<Mutex<ProgressTracker>>,
}

impl BatchProcessor {
    pub fn new(config: Config) -> Self {
        let thread_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(8); // 最大8个线程
        
        Self {
            config,
            thread_pool: ThreadPool::new(thread_count),
            progress_tracker: Arc::new(Mutex::new(ProgressTracker::new())),
        }
    }
    
    pub fn encrypt_directory(&self, dir_path: &Path, key: &[u8; 32]) -> Result<()> {
        // 收集所有文件
        let files = Self::collect_files(dir_path)?;
        let total_files = files.len();
        
        // 初始化进度跟踪
        let progress = self.progress_tracker.clone();
        progress.lock().unwrap().start(total_files);
        
        // 并行处理文件
        for file in files {
            let file_clone = file.clone();
            let key_clone = *key;
            let config_clone = self.config.clone();
            let progress_clone = progress.clone();
            
            self.thread_pool.execute(move || {
                match CryptoManager::encrypt_file(&file_clone, &key_clone, config_clone.preserve_original_filename) {
                    Ok(_) => {
                        progress_clone.lock().unwrap().increment_success();
                    }
                    Err(e) => {
                        progress_clone.lock().unwrap().increment_error(&file_clone, e);
                    }
                }
            });
        }
        
        // 等待所有任务完成
        self.thread_pool.wait_for_completion();
        
        // 显示结果
        let final_progress = progress.lock().unwrap();
        final_progress.display_summary();
        
        Ok(())
    }
}
```

#### 1.3 配置管理增强

**技术实现细节：**

##### 1.3.1 配置验证
```rust
// config.rs
impl Config {
    pub fn validate(&self) -> Result<()> {
        // 验证文件大小限制
        if self.max_file_size == 0 {
            return Err(BjtError::ConfigError(
                "最大文件大小不能为0".to_string()
            ));
        }
        
        // 验证文件扩展名
        if self.default_extension.is_empty() {
            return Err(BjtError::ConfigError(
                "默认文件扩展名不能为空".to_string()
            ));
        }
        
        // 验证危险路径
        for path in &self.forbidden_paths {
            if path.is_empty() {
                return Err(BjtError::ConfigError(
                    "危险路径不能为空".to_string()
                ));
            }
        }
        
        Ok(())
    }
    
    /// 迁移旧配置到新版本
    pub fn migrate_from_v1(v1_config: &str) -> Result<Self> {
        let mut config: Config = toml::from_str(v1_config)?;
        
        // 添加新字段的默认值
        config.preserve_original_filename = false;
        config.file_format_version = 2;
        
        Ok(config)
    }
}
```

### 阶段二：用户体验提升（2-3周）
**版本目标：1.2.0**

#### 2.1 文件列表和查看功能（P1）

**技术实现细节：**

##### 2.1.1 列表命令实现
```rust
// main.rs - 扩展CLI命令
#[derive(Subcommand)]
enum Commands {
    // ... 现有命令
    
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
    },
}
```

##### 2.1.2 文件信息提取
```rust
// crypto.rs
impl CryptoManager {
    /// 获取加密文件信息
    pub fn get_file_info(file_path: &Path, key: Option<&[u8; KEY_SIZE]>) -> Result<FileInfo> {
        let version = Self::detect_file_version(file_path)?;
        
        match version {
            1 => {
                // 旧版文件：只有基本信息
                Ok(FileInfo {
                    path: file_path.to_path_buf(),
                    version: 1,
                    original_filename: None,
                    encrypted_size: fs::metadata(file_path)?.len(),
                    created: Utils::get_file_creation_time(file_path)?,
                    modified: Utils::get_file_modification_time(file_path)?,
                })
            }
            2 => {
                // 新版文件：可提取文件名
                let mut file = fs::File::open(file_path)?;
                
                // 跳过魔术字节和版本号
                file.seek(SeekFrom::Start(5))?;
                
                // 读取文件名元数据长度
                let mut len_bytes = [0u8; 4];
                file.read_exact(&mut len_bytes)?;
                let metadata_len = u32::from_le_bytes(len_bytes) as usize;
                
                let mut original_filename = None;
                
                if let Some(key) = key {
                    // 读取并解密文件名
                    let mut encrypted_filename = vec![0u8; metadata_len];
                    file.read_exact(&mut encrypted_filename)?;
                    
                    match Self::decrypt_filename(&encrypted_filename, key) {
                        Ok(filename) => original_filename = Some(filename),
                        Err(_) => {
                            // 解密失败，可能密钥错误
                            original_filename = Some("[需要正确密钥]".to_string());
                        }
                    }
                }
                
                Ok(FileInfo {
                    path: file_path.to_path_buf(),
                    version: 2,
                    original_filename,
                    encrypted_size: fs::metadata(file_path)?.len(),
                    created: Utils::get_file_creation_time(file_path)?,
                    modified: Utils::get_file_modification_time(file_path)?,
                })
            }
            _ => Err(BjtError::CryptoError(
                format!("不支持的文件版本: {}", version)
            )),
        }
    }
}
```

#### 2.2 智能进度反馈（P1）

**技术实现细节：**

##### 2.2.1 进度跟踪器
```rust
// utils.rs
pub struct ProgressTracker {
    total_files: usize,
    processed_files: usize,
    successful_files: usize,
    failed_files: Vec<(PathBuf, String)>,
    start_time: Instant,
    speed_tracker: SpeedTracker,
}

impl ProgressTracker {
    pub fn display_progress(&self) {
        let elapsed = self.start_time.elapsed();
        let progress = if self.total_files > 0 {
            (self.processed_files as f64 / self.total_files as f64) * 100.0
        } else {
            0.0
        };
        
        let remaining = if self.processed_files > 0 {
            let avg_time_per_file = elapsed / self.processed_files as u32;
            let remaining_files = self.total_files - self.processed_files;
            avg_time_per_file * remaining_files as u32
        } else {
            Duration::from_secs(0)
        };
        
        println!(
            "进度: {:>5.1}% ({}/{}), 已用: {}, 剩余: ~{}, 速度: {}/s",
            progress,
            self.processed_files,
            self.total_files,
            Self::format_duration(elapsed),
            Self::format_duration(remaining),
            self.speed_tracker.get_speed_string()
        );
    }
}
```

#### 2.3 断点续传（P2.1）

**技术实现细节：**

##### 2.3.1 检查点系统
```rust
// fileops.rs
#[derive(Serialize, Deserialize)]
pub struct Checkpoint {
    pub batch_id: String,
    pub operation: String, // "encrypt" 或 "decrypt"
    pub target_path: PathBuf,
    pub total_files: usize,
    pub processed_files: Vec<PathBuf>,
    pub failed_files: Vec<(PathBuf, String)>,
    pub remaining_files: Vec<PathBuf>,
    pub timestamp: DateTime<Utc>,
    pub config_hash: String,
}

impl Checkpoint {
    pub fn save(&self, checkpoint_path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(checkpoint_path, content)?;
        Ok(())
    }
    
    pub fn load(checkpoint_path: &Path) -> Result<Self> {
        let content = fs::read_to_string(checkpoint_path)?;
        let checkpoint: Checkpoint = serde_json::from_str(&content)?;
        Ok(checkpoint)
    }
}
```

### 阶段三：性能优化（3-4周）
**版本目标：1.3.0**

#### 3.1 高级批量优化（P2.2）

**技术实现细节：**

##### 3.1.1 智能调度器
```rust
// utils.rs
pub struct SmartScheduler {
    files: Vec<ScheduledFile>,
    strategy: SchedulingStrategy,
}

pub enum SchedulingStrategy {
    SmallFirst,      // 小文件优先
    LargeFirst,      // 大文件优先
    Mixed,           // 混合策略
    Custom(fn(&ScheduledFile, &ScheduledFile) -> Ordering),
}

impl SmartScheduler {
    pub fn schedule_files(&mut self, files: Vec<PathBuf>) -> Vec<PathBuf> {
        // 获取文件大小信息
        let mut scheduled_files: Vec<ScheduledFile> = files
            .into_iter()
            .filter_map(|path| {
                fs::metadata(&path).ok().map(|metadata| ScheduledFile {
                    path,
                    size: metadata.len(),
                    priority: Self::calculate_priority(&metadata),
                })
            })
            .collect();
        
        // 根据策略排序
        match self.strategy {
            SchedulingStrategy::SmallFirst => {
                scheduled_files.sort_by_key(|f| f.size);
            }
            SchedulingStrategy::LargeFirst => {
                scheduled_files.sort_by_key(|f| std::cmp::Reverse(f.size));
            }
            SchedulingStrategy::Mixed => {
                // 混合策略：先处理中等大小文件，然后小文件，最后大文件
                scheduled_files.sort_by(|a, b| {
                    let a_group = Self::size_group(a.size);
                    let b_group = Self::size_group(b.size);
                    
                    match (a_group, b_group) {
                        (SizeGroup::Medium, SizeGroup::Medium) => a.size.cmp(&b.size),
                        (SizeGroup::Medium, _) => Ordering::Less,
                        (_, SizeGroup::Medium) => Ordering::Greater,
                        (SizeGroup::Small, SizeGroup::Large) => Ordering::Less,
                        (SizeGroup::Large, SizeGroup::Small) => Ordering::Greater,
                        _ => a.size.cmp(&b.size),
                    }
                });
            }
            SchedulingStrategy::Custom(cmp) => {
                scheduled_files.sort_by(cmp);
            }
        }
        
        scheduled_files.into_iter().map(|f| f.path).collect()
    }
}
```

##### 3.1.2 大文件分片处理
```rust
// crypto.rs
impl CryptoManager {
    /// 分片加密大文件
    pub fn encrypt_file_chunked(
        input_path: &Path,
        key: &[u8; KEY_SIZE],
        chunk_size: usize,
    ) -> Result<()> {
        let output_path = PathBuf::from(format!("{}.leo", input_path.display()));
        let mut output_file = fs::File::create(&output_path)?;
        
        // 写入文件头（包含分片信息）
        let header = FileHeader {
            version: 2,
            chunk_size: chunk_size as u64,
            total_chunks: 0, // 稍后更新
            original_size: fs::metadata(input_path)?.len(),
        };
        header.write(&mut output_file)?;
        
        // 分片读取和加密
        let mut input_file = fs::File::open(input_path)?;
        let mut buffer = vec![0u8; chunk_size];
        let mut chunk_index = 0;
        
        loop {
            let bytes_read = input_file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            
            // 加密当前分片
            let encrypted_chunk = Self::encrypt_data(&buffer[..bytes_read], key)?;
            
            // 写入分片头和数据
            let chunk_header = ChunkHeader {
                index: chunk_index,
                size: encrypted_chunk.len() as u32,
            };
            chunk_header.write(&mut output_file)?;
            output_file.write_all(&encrypted_chunk)?;
            
            chunk_index += 1;
        }
        
        // 更新文件头中的总分片数
        output_file.seek(SeekFrom::Start(0))?;
        let mut updated_header = header;
        updated_header.total_chunks = chunk_index;
        updated_header.write(&mut output_file)?;
        
        Ok(())
    }
}
```

#### 3.2 资源控制（P2.2）

**技术实现细节：**

##### 3.2.1 内存管理器
```rust
// utils.rs
pub struct MemoryManager {
    max_memory_mb: usize,
    current_usage: Arc<AtomicUsize>,
    semaphore: Arc<Semaphore>,
}

impl MemoryManager {
    pub fn new(max_memory_mb: usize) -> Self {
        let max_bytes = max_memory_mb * 1024 * 1024;
        Self {
            max_memory_mb,
            current_usage: Arc::new(AtomicUsize::new(0)),
            semaphore: Arc::new(Semaphore::new(max_bytes)),
        }
    }
    
    pub fn allocate(&self, size: usize) -> Result<MemoryGuard> {
        let permit = self.semaphore.try_acquire_many(size as u32)
            .map_err(|_| BjtError::ResourceError("内存不足".to_string()))?;
        
        self.current_usage.fetch_add(size, Ordering::SeqCst);
        
        Ok(MemoryGuard {
            size,
            current_usage: Arc::clone(&self.current_usage),
            _permit: permit,
        })
    }
    
    pub fn get_usage_percentage(&self) -> f64 {
        let current = self.current_usage.load(Ordering::SeqCst) as f64;
        let max = (self.max_memory_mb * 1024 * 1024) as f64;
        (current / max) * 100.0
    }
}
```

##### 3.2.2 动态并行度调整
```rust
// fileops.rs
pub struct AdaptiveThreadPool {
    min_threads: usize,
    max_threads: usize,
    current_threads: usize,
    cpu_usage_tracker: CpuUsageTracker,
    memory_manager: MemoryManager,
}

impl AdaptiveThreadPool {
    pub fn adjust_parallelism(&mut self) -> usize {
        let cpu_usage = self.cpu_usage_tracker.get_usage();
        let memory_usage = self.memory_manager.get_usage_percentage();
        
        // 根据资源使用情况调整线程数
        let target_threads = if cpu_usage > 80.0 || memory_usage > 70.0 {
            // 资源紧张，减少线程数
            (self.current_threads as f64 * 0.7).max(self.min_threads as f64) as usize
        } else if cpu_usage < 40.0 && memory_usage < 50.0 {
            // 资源充足，增加线程数
            (self.current_threads as f64 * 1.3).min(self.max_threads as f64) as usize
        } else {
            // 保持当前线程数
            self.current_threads
        };
        
        if target_threads != self.current_threads {
            println!(
                "调整并行度: {} -> {} (CPU: {:.1}%, 内存: {:.1}%)",
                self.current_threads, target_threads, cpu_usage, memory_usage
            );
            self.current_threads = target_threads;
        }
        
        self.current_threads
    }
}
```

#### 3.3 性能监控（P2.3）

**技术实现细节：**

##### 3.3.1 性能指标收集
```rust
// utils.rs
#[derive(Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub total_files: usize,
    pub total_size: u64,
    pub successful_files: usize,
    pub failed_files: usize,
    pub avg_speed_mbps: f64,
    pub peak_memory_mb: f64,
    pub cpu_usage_avg: f64,
    pub io_wait_avg: f64,
    pub details: Vec<FileMetrics>,
}

impl PerformanceMetrics {
    pub fn generate_report(&self) -> String {
        let duration = self.end_time.unwrap_or(Utc::now()) - self.start_time;
        let duration_secs = duration.num_seconds() as f64;
        
        format!(
            "性能报告 - {}\n\
            ========================================\n\
            操作类型: {}\n\
            持续时间: {:.1} 秒\n\
            处理文件: {} 个 ({} 成功, {} 失败)\n\
            总数据量: {}\n\
            平均速度: {:.2} MB/s\n\
            峰值内存: {:.1} MB\n\
            CPU使用率: {:.1}%\n\
            IO等待率: {:.1}%\n\
            ========================================",
            self.start_time.format("%Y-%m-%d %H:%M:%S"),
            self.operation,
            duration_secs,
            self.total_files,
            self.successful_files,
            self.failed_files,
            Self::format_size(self.total_size),
            self.avg_speed_mbps,
            self.peak_memory_mb,
            self.cpu_usage_avg,
            self.io_wait_avg
        )
    }
}
```

### 阶段四：企业级功能（4-6周）
**版本目标：2.0.0**

#### 4.1 批量配置文件（P2.3）

**技术实现细节：**

##### 4.1.1 批量配置格式
```toml
# batch-config.toml
[general]
name = "财务文件加密"
description = "每周财务报告加密任务"
schedule = "weekly"  # daily, weekly, monthly, cron表达式

[encryption]
preserve_original_filename = false
default_extension = ".leo"
keep_original = false

[resources]
max_memory_mb = 1024
max_threads = 8
chunk_size_mb = 10

[paths]
source = "/data/financial/reports/"
destination = "/data/encrypted/financial/"
include_patterns = ["*.pdf", "*.xlsx", "*.docx"]
exclude_patterns = ["temp_*", "*.tmp"]

[notifications]
email = "admin@example.com"
on_success = true
on_failure = true
on_completion = true

[retention]
keep_encrypted_days = 90
keep_logs_days = 365
```

##### 4.1.2 批量任务执行器
```rust
// batch.rs
pub struct BatchExecutor {
    config: BatchConfig,
    scheduler: JobScheduler,
    notifier: Notifier,
    auditor: Auditor,
}

impl BatchExecutor {
    pub async fn execute(&self) -> Result<BatchResult> {
        // 验证配置
        self.config.validate()?;
        
        // 准备执行环境
        let env = self.prepare_environment()?;
        
        // 执行批量操作
        let result = self.execute_batch(&env).await?;
        
        // 发送通知
        self.notifier.notify(&result).await?;
        
        // 记录审计日志
        self.auditor.record(&result).await?;
        
        // 清理临时文件
        self.cleanup(&env)?;
        
        Ok(result)
    }
    
    pub async fn schedule(&self) -> Result<()> {
        match &self.config.general.schedule {
            Schedule::Cron(cron_expr) => {
                self.scheduler.schedule_cron(cron_expr, || {
                    self.execute().await
                }).await?;
            }
            Schedule::Interval(interval) => {
                self.scheduler.schedule_interval(*interval, || {
                    self.execute().await
                }).await?;
            }
            Schedule::Manual => {
                // 手动执行，不调度
            }
        }
        
        Ok(())
    }
}
```

#### 4.2 审计和日志（P1）

**技术实现细节：**

##### 4.2.1 审计日志系统
```rust
// audit.rs
#[derive(Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub operation: String,
    pub target: String,
    pub result: OperationResult,
    pub details: AuditDetails,
    pub signature: Option<String>,  // 数字签名
}

impl AuditLog {
    pub fn verify_integrity(&self, public_key: &[u8]) -> Result<bool> {
        if let Some(signature) = &self.signature {
            // 验证数字签名
            let data = serde_json::to_vec(&self.details)?;
            Ok(crypto::verify_signature(&data, signature, public_key)?)
        } else {
            Ok(false)
        }
    }
    
    pub fn generate_report(&self, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Text => Ok(self.to_text()),
            ReportFormat::Json => Ok(serde_json::to_string_pretty(self)?),
            ReportFormat::Html => Ok(self.to_html()),
            ReportFormat::Csv => Ok(self.to_csv()),
        }
    }
}
```

#### 4.3 高级密钥管理

**技术实现细节：**

##### 4.3.1 密钥轮换系统
```rust
// keymgmt.rs
pub struct KeyRotationManager {
    current_key: Arc<RwLock<Key>>,
    previous_keys: Vec<Key>,
    rotation_policy: RotationPolicy,
}

impl KeyRotationManager {
    pub async fn rotate_key(&mut self) -> Result<()> {
        // 生成新密钥
        let new_key = CryptoManager::generate_key()?;
        
        // 使用新密钥重新加密需要保留的文件
        self.reencrypt_files(&new_key).await?;
        
        // 更新密钥链
        let old_key = self.current_key.write().await.clone();
        self.previous_keys.push(old_key);
        *self.current_key.write().await = new_key;
        
        // 清理旧密钥（根据策略）
        self.cleanup_old_keys().await?;
        
        Ok(())
    }
    
    pub fn get_key_for_file(&self, file_path: &Path) -> Result<Arc<Key>> {
        // 检测文件使用的密钥版本
        let key_version = Self::detect_key_version(file_path)?;
        
        match key_version {
            0 => Ok(Arc::clone(&self.current_key)),
            n if n > 0 && n <= self.previous_keys.len() as u32 => {
                // 使用历史密钥
                Ok(Arc::new(self.previous_keys[n as usize - 1].clone()))
            }
            _ => Err(BjtError::KeyError("无法识别的密钥版本".to_string())),
        }
    }
}
```

### 阶段五：生态系统集成（6-8周）
**版本目标：2.1.0+**

#### 5.1 API接口

**技术实现细节：**

##### 5.1.1 REST API设计
```rust
// api/server.rs
#[derive(Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptRequest {
    pub path: String,
    pub preserve_filename: Option<bool>,
    pub keep_original: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct BatchEncryptRequest {
    pub paths: Vec<String>,
    pub config: Option<BatchConfig>,
}

// API路由
pub fn setup_routes(app: &mut ServiceConfig) {
    app.service(
        web::scope("/api/v1")
            .route("/encrypt", web::post().to(encrypt_file))
            .route("/decrypt", web::post().to(decrypt_file))
            .route("/batch/encrypt", web::post().to(batch_encrypt))
            .route("/batch/decrypt", web::post().to(batch_decrypt))
            .route("/status", web::get().to(get_status))
            .route("/metrics", web::get().to(get_metrics))
            .route("/config", web::get().to(get_config))
            .route("/config", web::put().to(update_config)),
    );
}

// gRPC接口（可选）
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EncryptRequest {
    #[prost(string, tag = "1")]
    pub path: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub preserve_filename: bool,
    #[prost(bool, tag = "3")]
    pub keep_original: bool,
}

#[tonic::async_trait]
impl Leolock for LeolockServer {
    async fn encrypt(
        &self,
        request: Request<EncryptRequest>,
    ) -> Result<Response<EncryptResponse>, Status> {
        let req = request.into_inner();
        // 处理加密请求
        Ok(Response::new(EncryptResponse {
            success: true,
            encrypted_path: format!("{}.leo", req.path),
        }))
    }
}
```

##### 5.1.2 库模式接口
```rust
// lib.rs - 库模式入口
pub struct LeolockLibrary {
    config: Config,
    key_manager: KeyManager,
}

impl LeolockLibrary {
    pub fn new(config: Config) -> Result<Self> {
        let key_manager = KeyManager::load()?;
        Ok(Self { config, key_manager })
    }
    
    pub fn encrypt_file(&self, path: &Path, options: EncryptOptions) -> Result<()> {
        let key = self.key_manager.get_key()?;
        CryptoManager::encrypt_file(path, &key, options.preserve_filename)?;
        Ok(())
    }
    
    pub fn encrypt_data(&self, data: &[u8], options: EncryptOptions) -> Result<Vec<u8>> {
        let key = self.key_manager.get_key()?;
        CryptoManager::encrypt_data(data, &key)
    }
}

// 供其他Rust程序使用
pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
```

#### 5.2 云存储集成

**技术实现细节：**

##### 5.2.1 S3兼容存储支持
```rust
// cloud/s3.rs
pub struct S3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: Option<String>,
}

impl S3Storage {
    pub async fn encrypt_object(&self, key: &str, preserve_filename: bool) -> Result<()> {
        // 下载对象
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        
        let data = response.body.collect().await?.to_vec();
        
        // 加密数据
        let crypto = CryptoManager::new();
        let encrypted_data = crypto.encrypt_data(&data, preserve_filename)?;
        
        // 上传加密后的对象
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(format!("{}.leo", key))
            .body(encrypted_data.into())
            .send()
            .await?;
        
        Ok(())
    }
    
    pub async fn sync_directory(&self, local_path: &Path, remote_prefix: &str) -> Result<()> {
        // 扫描本地目录
        for entry in WalkDir::new(local_path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                let relative_path = entry.path().strip_prefix(local_path)?;
                let remote_key = format!("{}/{}", remote_prefix, relative_path.display());
                
                // 上传并加密
                self.encrypt_object(&remote_key, false).await?;
            }
        }
        
        Ok(())
    }
}
```

#### 5.3 桌面和移动端

**技术实现细节：**

##### 5.3.1 图形用户界面（Tauri）
```rust
// src-tauri/src/main.rs
#[tauri::command]
fn encrypt_file(path: String, preserve_filename: bool) -> Result<String, String> {
    let config = Config::load().map_err(|e| e.to_string())?;
    let key = KeyManager::load().map_err(|e| e.to_string())?.get_key()
        .map_err(|e| e.to_string())?;
    
    CryptoManager::encrypt_file(Path::new(&path), &key, preserve_filename)
        .map_err(|e| e.to_string())?;
    
    Ok(format!("{}.leo", path))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            encrypt_file,
            decrypt_file,
            list_files,
            get_config,
            update_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

##### 5.3.2 移动端（Flutter + FFI）
```dart
// lib/leolock.dart
class Leolock {
  static final DynamicLibrary _lib = Platform.isAndroid
      ? DynamicLibrary.open('libleolock.so')
      : DynamicLibrary.process();
  
  final Pointer<Utf8> Function(Pointer<Utf8>, Bool) _encryptFile = _lib
      .lookup<NativeFunction<Pointer<Utf8> Function(Pointer<Utf8>, Bool)>>(
          'leolock_encrypt_file')
      .asFunction();
  
  Future<String> encryptFile(String path, bool preserveFilename) async {
    final pathPtr = path.toNativeUtf8();
    try {
      final resultPtr = _encryptFile(pathPtr, preserveFilename ? 1 : 0);
      return resultPtr.toDartString();
    } finally {
      malloc.free(pathPtr);
    }
  }
}
```

## 实施时间线

### 里程碑 1：文件名加密（2周）
**目标：** 完成基础文件名加密功能
- [ ] 配置文件扩展
- [ ] 新文件格式实现
- [ ] 文件名加密/解密逻辑
- [ ] 向后兼容处理
- [ ] 基础测试套件

### 里程碑 2：批量处理优化（3周）
**目标：** 提升批量操作性能
- [ ] 线程池实现
- [ ] 进度跟踪系统
- [ ] 错误处理和恢复
- [ ] 性能基准测试

### 里程碑 3：用户体验增强（2周）
**目标：** 改进命令行交互
- [ ] 文件列表命令
- [ ] 智能进度显示
- [ ] 详细错误报告
- [ ] 配置验证和迁移

### 里程碑 4：企业功能（4周）
**目标：** 添加企业级特性
- [ ] 批量配置文件
- [ ] 审计日志系统
- [ ] 密钥轮换管理
- [ ] API接口基础

### 里程碑 5：生态系统（4周）
**目标：** 扩展集成能力
- [ ] REST API服务器
- [ ] 库模式接口
- [ ] 云存储集成
- [ ] 基础GUI界面

## 技术依赖和风险

### 依赖项
1. **Rust生态系统**：保持与最新稳定版的兼容性
2. **加密库**：aes-gcm, argon2 的安全更新
3. **异步运行时**：tokio 或 async-std 的选择
4. **平台支持**：Linux, macOS, Windows 的兼容性

### 技术风险
1. **加密算法过时**：定期评估和更新加密算法
2. **性能瓶颈**：大规模文件处理时的性能问题
3. **兼容性破坏**：新版本无法处理旧文件
4. **安全漏洞**：依赖库的安全问题

### 缓解措施
1. **渐进式升级**：分阶段发布，充分测试
2. **性能监控**：建立性能基准和监控
3. **兼容性测试**：维护旧版本测试套件
4. **安全审计**：定期进行代码安全审计

## 贡献指南

### 开发流程
1. **功能分支**：每个功能在独立分支开发
2. **代码审查**：所有更改需要代码审查
3. **测试要求**：新功能必须包含测试
4. **文档更新**：API变更需要更新文档

### 测试策略
1. **单元测试**：核心功能的单元测试
2. **集成测试**：端到端的功能测试
3. **性能测试**：大规模文件处理测试
4. **兼容性测试**：跨平台和版本测试

### 发布流程
1. **Alpha测试**：内部测试和功能验证
2. **Beta测试**：有限用户群体测试
3. **RC版本**：发布候选，修复关键问题
4. **正式发布**：稳定版本发布

## 维护和支持

### 版本支持策略
- **当前版本**：完全支持，定期更新
- **上一个版本**：安全修复支持
- **更早版本**：社区支持，无官方维护

### 问题跟踪
- **GitHub Issues**：功能请求和bug报告
- **安全漏洞**：通过安全邮件报告
- **文档问题**：GitHub Wiki和文档仓库

### 社区参与
- **贡献者指南**：欢迎代码贡献
- **文档翻译**：多语言文档支持
- **用例分享**：用户案例和最佳实践

---

*本路线图会根据实际开发进度和用户反馈进行调整。建议定期回顾和更新。*

**最后更新：** 2026-03-12  
**维护者：** Burgess Leo  
**项目状态：** 活跃开发中