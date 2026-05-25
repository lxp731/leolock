use axum::{
    body::Body,
    extract::{ConnectInfo, Multipart, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use zeroize::Zeroizing;

use leolock::crypto::CryptoManager;
use leolock::errors::BjtError;

use crate::middleware;
use crate::state::AppState;

// ─── 请求/响应类型 ──────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UnlockRequest {
    /// 明文密码，消费后立即 zeroize
    password: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct InitRequest {
    /// 明文密码，消费后立即 zeroize
    password: String,
}

impl UnlockRequest {
    fn into_password(self) -> Zeroizing<String> {
        Zeroizing::new(self.password)
    }
}

impl InitRequest {
    fn into_password(self) -> Zeroizing<String> {
        Zeroizing::new(self.password)
    }
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub initialized: bool,
    pub locked: bool,
    pub version: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u32,
    pub token_type: String,
}

#[derive(Serialize)]
pub struct InitResponse {
    pub status: String,
    pub message: String,
    pub api_key: String,
}

// ─── 应用错误 ──────────────────────────────────────────────────

pub enum AppError {
    Locked,
    NotInitialized,
    BadRequest(String),
    CryptoError(String),
    RateLimited,
    Internal(String),
}

impl From<BjtError> for AppError {
    fn from(e: BjtError) -> Self {
        AppError::CryptoError(format!("{}", e))
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Internal(format!("IO: {}", e))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Locked => (
                StatusCode::LOCKED,
                "🔒 服务已锁定，请先 POST /api/v1/unlock".into(),
            ),
            AppError::NotInitialized => (
                StatusCode::PRECONDITION_FAILED,
                "❌ 未初始化，请先执行 leolock init".into(),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::CryptoError(msg) => (StatusCode::BAD_REQUEST, format!("加密错误: {}", msg)),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "⏳ 请求过于频繁，请 1 分钟后再试".into(),
            ),
            AppError::Internal(msg) => {
                eprintln!("[ERROR] {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误".into())
            }
        };
        (
            status,
            Json(MessageResponse {
                status: "error".into(),
                message,
            }),
        )
            .into_response()
    }
}

// ─── multipart 文件提取 ────────────────────────────────────────

struct MultipartFile {
    data: Vec<u8>,
    file_name: String,
}

async fn extract_file(mut multipart: Multipart) -> Result<MultipartFile, AppError> {
    let mut data: Option<Vec<u8>> = None;
    let mut file_name: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    Ok(MultipartFile {
        data: data.ok_or_else(|| AppError::BadRequest("缺少 file 字段".into()))?,
        file_name: file_name.unwrap_or_else(|| "unknown".to_string()),
    })
}

// ─── 轮换 API Key ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RotateApiKeyRequest {
    /// 明文密码（用于验证身份），消费后立即 zeroize
    password: String,
}

impl RotateApiKeyRequest {
    fn into_password(self) -> Zeroizing<String> {
        Zeroizing::new(self.password)
    }
}

/// POST /api/v1/auth/rotate-api-key
/// 用密码验证身份后生成新的 API Key（旧 Key 立即失效）
pub async fn rotate_api_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RotateApiKeyRequest>,
) -> Result<Json<InitResponse>, AppError> {
    let password = body.into_password();

    // 验证密码
    let salt_b64 = state
        .get_salt()
        .ok_or_else(|| AppError::BadRequest("配置中缺少盐值".into()))?;

    use base64::Engine;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(salt_b64)
        .map_err(|e| AppError::BadRequest(format!("盐值解码失败: {}", e)))?;

    CryptoManager::derive_key_from_password(
        &password,
        &salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密码错误: {}", e)))?;

    // 生成新 API Key
    let mut config = leolock::config::Config::load().unwrap_or_default();
    let api_key = config
        .generate_api_key()
        .map_err(|e| AppError::Internal(format!("生成 API Key 失败: {}", e)))?;
    config
        .save()
        .map_err(|e| AppError::Internal(format!("保存配置失败: {}", e)))?;

    // 更新运行时状态（旧 Key 立即失效，无需重启）
    state.update_api_key_hash(config.auth.api_key_hash.clone().unwrap());

    Ok(Json(InitResponse {
        status: "rotated".into(),
        message: "🔑 API Key 已轮换，旧 Key 立即失效。请保存新 Key！".into(),
        api_key,
    }))
}

// ─── 鉴权：登录 ────────────────────────────────────────────────

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if !state.has_api_key() {
        return Err(AppError::BadRequest(
            "服务未生成 API Key，请先执行 leolock init".into(),
        ));
    }

    if !state.verify_api_key(&body.api_key) {
        return Err(AppError::BadRequest("API Key 无效".into()));
    }

    let secret = state
        .jwt_secret
        .as_ref()
        .ok_or_else(|| AppError::Internal("JWT 密钥未配置".into()))?;

    let token = middleware::issue_token(secret)
        .map_err(|e| AppError::Internal(format!("签发 Token 失败: {}", e)))?;

    Ok(Json(LoginResponse {
        token,
        expires_in: 1800,
        token_type: "Bearer".into(),
    }))
}

// ─── 健康检查 / 状态 ───────────────────────────────────────────

pub async fn health() -> &'static str {
    "ok"
}

pub async fn status(State(state): State<Arc<AppState>>) -> Json<StatusResponse> {
    Json(StatusResponse {
        initialized: state.is_initialized,
        locked: !state.is_unlocked(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

// ─── 锁定 / 解锁 ───────────────────────────────────────────────

pub async fn unlock(
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<UnlockRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    if !state.check_unlock_rate(addr.ip()).await {
        return Err(AppError::RateLimited);
    }

    let password = body.into_password();

    if !state.is_initialized {
        return Err(AppError::NotInitialized);
    }

    let salt_b64 = state
        .get_salt()
        .ok_or_else(|| AppError::BadRequest("配置中缺少盐值".into()))?;

    use base64::Engine;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(salt_b64)
        .map_err(|e| AppError::BadRequest(format!("盐值解码失败: {}", e)))?;

    let key = CryptoManager::derive_key_from_password(
        &password,
        &salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密钥派生失败: {}", e)))?;

    state.unlock(key);

    Ok(Json(MessageResponse {
        status: "unlocked".into(),
        message: "🔓 服务已解锁，密钥已加载到内存".into(),
    }))
}

pub async fn lock(State(state): State<Arc<AppState>>) -> Json<MessageResponse> {
    state.lock();
    Json(MessageResponse {
        status: "locked".into(),
        message: "🔒 服务已锁定，密钥已从内存擦除".into(),
    })
}

// ─── 初始化 ────────────────────────────────────────────────────

pub async fn init(
    State(state): State<Arc<AppState>>,
    Json(body): Json<InitRequest>,
) -> Result<Json<InitResponse>, AppError> {
    let password = body.into_password();

    if state.is_initialized {
        return Err(AppError::BadRequest("服务已初始化".into()));
    }

    // 验证密码强度
    leolock::password::PasswordManager::validate_password_strength(&password)
        .map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    use base64::Engine;
    use leolock::config::Config;

    let mut config = Config::load().unwrap_or_default();

    // 生成盐值
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt)
        .map_err(|e| AppError::CryptoError(format!("生成盐值失败: {}", e)))?;
    let salt_b64 = base64::engine::general_purpose::STANDARD.encode(salt);

    // 派生主密钥
    let key = CryptoManager::derive_key_from_password(
        &password,
        &salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密钥派生失败: {}", e)))?;

    // 保存密钥
    leolock::keymgmt::KeyManager::save_key(&key)?;

    // 保存盐值
    config.core.salt = Some(salt_b64);

    // 生成 API Key 和 JWT Secret
    let api_key = config
        .generate_api_key()
        .map_err(|e| AppError::Internal(format!("生成 API Key 失败: {}", e)))?;
    config
        .generate_jwt_secret()
        .map_err(|e| AppError::Internal(format!("生成 JWT 密钥失败: {}", e)))?;

    config
        .save()
        .map_err(|e| AppError::Internal(format!("保存配置失败: {}", e)))?;

    // 创建备份
    let key_zeroizing = zeroize::Zeroizing::new(key);
    let backup_path = leolock::keymgmt::KeyManager::create_backup(&key_zeroizing, &password)
        .map_err(|e| AppError::Internal(format!("创建备份失败: {}", e)))?;

    Ok(Json(InitResponse {
        status: "initialized".into(),
        message: format!("✅ 初始化完成，备份已保存至 {}", backup_path.display()),
        api_key,
    }))
}

// ─── 加密（内存直通，无磁盘中转）───────────────────────────────

pub async fn encrypt(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let key = state.get_key().ok_or(AppError::Locked)?;
    let file = extract_file(multipart).await?;

    // 直接使用内存加解密，不走临时文件
    let encrypted = CryptoManager::encrypt_data_v3(&file.data, &file.file_name, &key)?;

    // 输出文件名使用哈希（与 CLI 一致）
    let hash = leolock::utils::Utils::get_display_filename(&file.file_name, false);

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", hash),
        ),
    ];

    Ok((headers, encrypted).into_response())
}

// ─── 解密（内存直通，无磁盘中转）───────────────────────────────

pub async fn decrypt(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Response, AppError> {
    let key = state.get_key().ok_or(AppError::Locked)?;
    let file = extract_file(multipart).await?;

    let (original_name, decrypted) = CryptoManager::decrypt_data_v3(&file.data, &key)?;

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", original_name),
        ),
    ];

    Ok((headers, decrypted).into_response())
}

// ─── 文件 ID 编解码 ────────────────────────────────────────────

fn encode_id(path: &std::path::Path) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(path.to_string_lossy().as_bytes())
}

fn decode_id(id: &str) -> Result<PathBuf, AppError> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(id)
        .map_err(|_| AppError::BadRequest("无效的文件 ID".into()))?;
    let path_str =
        String::from_utf8(bytes).map_err(|_| AppError::BadRequest("无效的文件 ID 编码".into()))?;
    Ok(PathBuf::from(path_str))
}

// ─── 文件列表查询参数 ──────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct ListQuery {
    pub(crate) path: String,
    page: Option<usize>,
    per_page: Option<usize>,
    sort: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct FileItem {
    pub(crate) id: String,
    version: u8,
    pub(crate) encrypted_size: u64,
    pub(crate) original_name: Option<String>,
    pub(crate) decryptable: bool,
}

#[derive(Serialize)]
pub(crate) struct ListResponse {
    items: Vec<FileItem>,
    total: usize,
    page: usize,
    per_page: usize,
}

#[derive(Serialize)]
pub(crate) struct FileDetailResponse {
    id: String,
    path: String,
    version: u8,
    pub(crate) encrypted_size: u64,
    pub(crate) original_name: Option<String>,
    pub(crate) decryptable: bool,
    exists: bool,
}

// ─── 文件列表 ─────────────────────────────────────────────────

/// GET /api/v1/files?path=/data&page=1&per_page=50&sort=size_desc
pub async fn list_files(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let dir = PathBuf::from(&query.path);
    if !dir.is_dir() {
        return Err(AppError::BadRequest(format!(
            "目录不存在: {}",
            dir.display()
        )));
    }

    let key = state.get_key();
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(50).clamp(1, 200);

    let mut items = Vec::new();

    match std::fs::read_dir(&dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if !file_path.is_file() {
                    continue;
                }
                if file_path.extension().map_or(true, |e| e != "leo") {
                    continue;
                }

                match CryptoManager::get_file_info(&file_path, key.as_ref()) {
                    Ok(info) => {
                        items.push(FileItem {
                            id: encode_id(&file_path),
                            version: info.version,
                            encrypted_size: info.encrypted_size,
                            original_name: info.original_filename,
                            decryptable: info.decryptable,
                        });
                    }
                    Err(_) => {
                        // 跳过无法解析的文件
                    }
                }
            }
        }
        Err(e) => {
            return Err(AppError::BadRequest(format!("读取目录失败: {}", e)));
        }
    }

    // 排序
    if let Some(sort) = &query.sort {
        match sort.as_str() {
            "size_asc" => items.sort_by(|a, b| a.encrypted_size.cmp(&b.encrypted_size)),
            "size_desc" => items.sort_by(|a, b| b.encrypted_size.cmp(&a.encrypted_size)),
            "name_asc" => items.sort_by(|a, b| a.original_name.cmp(&b.original_name)),
            "name_desc" => items.sort_by(|a, b| b.original_name.cmp(&a.original_name)),
            _ => {}
        }
    }

    let total = items.len();
    let start = (page - 1) * per_page;
    let paged: Vec<FileItem> = items.into_iter().skip(start).take(per_page).collect();

    Ok(Json(ListResponse {
        items: paged,
        total,
        page,
        per_page,
    }))
}

// ─── ID 查询参数 ─────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct IdQuery {
    id: String,
}

// ─── 单个文件详情 ──────────────────────────────────────────────

/// GET /api/v1/files/get?id=xxx
pub async fn get_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IdQuery>,
) -> Result<Json<FileDetailResponse>, AppError> {
    let file_path = decode_id(&query.id)?;
    let key = state.get_key();

    let info = CryptoManager::get_file_info(&file_path, key.as_ref())
        .map_err(|e| AppError::BadRequest(format!("读取文件信息失败: {}", e)))?;

    Ok(Json(FileDetailResponse {
        id: query.id,
        path: file_path.to_string_lossy().into(),
        version: info.version,
        encrypted_size: info.encrypted_size,
        original_name: info.original_filename,
        decryptable: info.decryptable,
        exists: file_path.exists(),
    }))
}

// ─── 下载解密 ──────────────────────────────────────────────────

/// GET /api/v1/files/download?id=xxx
pub async fn download_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IdQuery>,
) -> Result<Response, AppError> {
    let key = state.get_key().ok_or(AppError::Locked)?;
    let file_path = decode_id(&query.id)?;

    if !file_path.exists() {
        return Err(AppError::BadRequest(format!(
            "文件不存在: {}",
            file_path.display()
        )));
    }

    let encrypted_data = std::fs::read(&file_path)?;
    let (original_name, decrypted) = CryptoManager::decrypt_data_v3(&encrypted_data, &key)?;

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (
            header::CONTENT_DISPOSITION,
            &format!("attachment; filename=\"{}\"", original_name),
        ),
    ];

    Ok((headers, decrypted).into_response())
}

// ─── 删除加密文件 ──────────────────────────────────────────────

/// DELETE /api/v1/files/delete?id=xxx
pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    Query(query): Query<IdQuery>,
) -> Result<Json<MessageResponse>, AppError> {
    let _key = state.get_key().ok_or(AppError::Locked)?;
    let file_path = decode_id(&query.id)?;

    if !file_path.exists() {
        return Err(AppError::BadRequest(format!(
            "文件不存在: {}",
            file_path.display()
        )));
    }

    if file_path.extension().map_or(true, |e| e != "leo") {
        return Err(AppError::BadRequest("只能删除 .leo 加密文件".into()));
    }

    leolock::utils::Utils::secure_delete_file(&file_path)?;

    Ok(Json(MessageResponse {
        status: "deleted".into(),
        message: format!("已删除: {}", file_path.display()),
    }))
}

// ─── 流式加密（原始二进制 body）─────────────────────────────

/// POST /api/v1/encrypt-stream
/// 接收原始二进制 body（非 multipart），X-Filename 头指定文件名
/// 比 multipart 端点更高效，无 MIME 解析开销
pub async fn encrypt_stream(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, AppError> {
    let key = state.get_key().ok_or(AppError::Locked)?;
    let filename = headers
        .get("X-Filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let data = body
        .collect()
        .await
        .map(|b| b.to_bytes().to_vec())
        .map_err(|e| AppError::BadRequest(format!("读取请求体失败: {}", e)))?;

    let encrypted = CryptoManager::encrypt_data_v3(&data, filename, &key)?;
    let hash = leolock::utils::Utils::get_display_filename(filename, false);

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", hash),
        )
        .body(Body::from(encrypted))
        .unwrap())
}

// ─── 流式解密（原始二进制 body）─────────────────────────────

/// POST /api/v1/decrypt-stream
/// 接收原始 V3 加密二进制 body，返回解密数据
/// 比 multipart 端点更高效，无 MIME 解析开销
pub async fn decrypt_stream(
    State(state): State<Arc<AppState>>,
    body: Body,
) -> Result<Response, AppError> {
    let key = state.get_key().ok_or(AppError::Locked)?;

    let data = body
        .collect()
        .await
        .map(|b| b.to_bytes().to_vec())
        .map_err(|e| AppError::BadRequest(format!("读取请求体失败: {}", e)))?;

    let (original_name, decrypted) = CryptoManager::decrypt_data_v3(&data, &key)?;

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", original_name),
        )
        .body(Body::from(decrypted))
        .unwrap())
}

// ─── 配置读写 ─────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ConfigResponse {
    program: ProgramConfigView,
    core: CoreConfigView,
    auth: AuthConfigView,
    api: ApiConfigView,
}

#[derive(Serialize)]
struct ProgramConfigView {
    forbidden_paths: Vec<String>,
    max_file_size: u64,
    default_extension: String,
    key_file_path: String,
    preserve_original_filename: bool,
    show_progress: bool,
    file_format_version: u8,
}

#[derive(Serialize)]
struct CoreConfigView {
    salt: String,
    argon2_m_cost: u32,
    argon2_t_cost: u32,
    argon2_p_cost: u32,
}

#[derive(Serialize)]
struct AuthConfigView {
    api_key_hash: String,
    jwt_secret: String,
}

#[derive(Serialize)]
struct ApiConfigView {
    bind_address: String,
    port: u16,
}

#[derive(Deserialize)]
pub struct ConfigUpdateRequest {
    pub program: Option<ProgramUpdate>,
    pub api: Option<ApiUpdate>,
}

#[derive(Deserialize)]
pub(crate) struct ProgramUpdate {
    pub(crate) forbidden_paths: Option<Vec<String>>,
    pub(crate) max_file_size: Option<u64>,
    pub(crate) default_extension: Option<String>,
    pub(crate) key_file_path: Option<String>,
    pub(crate) preserve_original_filename: Option<bool>,
    pub(crate) show_progress: Option<bool>,
    pub(crate) file_format_version: Option<u8>,
}

#[derive(Deserialize)]
pub(crate) struct ApiUpdate {
    pub(crate) bind_address: Option<String>,
    pub(crate) port: Option<u16>,
}

/// GET /api/v1/config
/// 返回当前配置，敏感字段脱敏
pub async fn get_config() -> Result<Json<ConfigResponse>, AppError> {
    let config = leolock::config::Config::load().unwrap_or_default();

    let mask = |v: &Option<String>| {
        if v.is_some() {
            "***".to_string()
        } else {
            "未配置".to_string()
        }
    };

    Ok(Json(ConfigResponse {
        program: ProgramConfigView {
            forbidden_paths: config.program.forbidden_paths,
            max_file_size: config.program.max_file_size,
            default_extension: config.program.default_extension,
            key_file_path: config.program.key_file_path,
            preserve_original_filename: config.program.preserve_original_filename,
            show_progress: config.program.show_progress,
            file_format_version: config.program.file_format_version,
        },
        core: CoreConfigView {
            salt: mask(&config.core.salt),
            argon2_m_cost: config.core.argon2_m_cost,
            argon2_t_cost: config.core.argon2_t_cost,
            argon2_p_cost: config.core.argon2_p_cost,
        },
        auth: AuthConfigView {
            api_key_hash: mask(&config.auth.api_key_hash),
            jwt_secret: mask(&config.auth.jwt_secret),
        },
        api: ApiConfigView {
            bind_address: config.api.bind_address,
            port: config.api.port,
        },
    }))
}

/// PUT /api/v1/config
/// 更新 [program] 和 [api] 段，不允许修改 [core] 和 [auth]
pub async fn update_config(
    Json(body): Json<ConfigUpdateRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let mut config = leolock::config::Config::load().unwrap_or_default();

    if let Some(p) = body.program {
        if let Some(v) = p.forbidden_paths {
            config.program.forbidden_paths = v;
        }
        if let Some(v) = p.max_file_size {
            config.program.max_file_size = v;
        }
        if let Some(v) = p.default_extension {
            config.program.default_extension = v;
        }
        if let Some(v) = p.key_file_path {
            config.program.key_file_path = v;
        }
        if let Some(v) = p.preserve_original_filename {
            config.program.preserve_original_filename = v;
        }
        if let Some(v) = p.show_progress {
            config.program.show_progress = v;
        }
        if let Some(v) = p.file_format_version {
            config.program.file_format_version = v;
        }
    }

    if let Some(s) = body.api {
        if let Some(v) = s.bind_address {
            config.api.bind_address = v;
        }
        if let Some(v) = s.port {
            config.api.port = v;
        }
    }

    config
        .save()
        .map_err(|e| AppError::Internal(format!("保存配置失败: {}", e)))?;

    Ok(Json(MessageResponse {
        status: "updated".into(),
        message: "✅ 配置已更新（部分修改需重启服务生效）".into(),
    }))
}

// ─── 统计信息 ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct StatsQuery {
    path: String,
}

#[derive(Serialize)]
pub(crate) struct StatsResponse {
    path: String,
    total_files: usize,
    encrypted_files: usize,
    total_encrypted_size: u64,
    decryptable_count: usize,
    versions: std::collections::HashMap<String, usize>,
}

/// GET /api/v1/metrics — Prometheus 格式指标
pub async fn metrics(State(state): State<Arc<AppState>>) -> Result<String, AppError> {
    let uptime = state.start_time.elapsed().as_secs();
    let locked = if state.is_unlocked() { 0u8 } else { 1u8 };

    let mut out = String::new();
    out.push_str(&format!(
        "# HELP leolock_uptime_seconds Service uptime\n# TYPE leolock_uptime_seconds gauge\nleolock_uptime_seconds {}\n",
        uptime
    ));
    out.push_str(&format!(
        "# HELP leolock_service_locked Is service locked (1=locked)\n# TYPE leolock_service_locked gauge\nleolock_service_locked {}\n",
        locked
    ));
    out.push_str("# HELP leolock_requests_total Total requests by path\n# TYPE leolock_requests_total counter\n");
    let counts = state.request_count.lock().await;
    let mut paths: Vec<_> = counts.iter().collect();
    paths.sort_by_key(|(p, _)| *p);
    for (path, count) in paths {
        out.push_str(&format!(
            "leolock_requests_total{{path=\"{}\"}} {}\n",
            path, count
        ));
    }

    Ok(out)
}

/// GET /api/v1/stats?path=/data
pub async fn stats(
    State(state): State<Arc<AppState>>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<StatsResponse>, AppError> {
    let dir = PathBuf::from(&query.path);
    if !dir.is_dir() {
        return Err(AppError::BadRequest(format!(
            "目录不存在: {}",
            dir.display()
        )));
    }

    let key = state.get_key();
    let mut total_files = 0usize;
    let mut encrypted_files = 0usize;
    let mut total_encrypted_size = 0u64;
    let mut decryptable_count = 0usize;
    let mut versions: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            total_files += 1;

            let is_leo = path.extension().map_or(false, |e| e == "leo");
            if !is_leo {
                continue;
            }
            encrypted_files += 1;

            if let Ok(info) = CryptoManager::get_file_info(&path, key.as_ref()) {
                total_encrypted_size += info.encrypted_size;
                if info.decryptable {
                    decryptable_count += 1;
                }
                *versions.entry(format!("v{}", info.version)).or_insert(0) += 1;
            }
        }
    }

    Ok(Json(StatsResponse {
        path: query.path,
        total_files,
        encrypted_files,
        total_encrypted_size,
        decryptable_count,
        versions,
    }))
}

// ─── 密钥轮换 ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct RotateKeyRequest {
    /// 当前密码（用于验证身份）
    password: String,
    /// 可选：重新加密此目录下的所有 .leo 文件
    re_encrypt_path: Option<String>,
}

impl RotateKeyRequest {
    fn into_password(self) -> Zeroizing<String> {
        Zeroizing::new(self.password)
    }
}

#[derive(Serialize)]
pub(crate) struct RotateKeyResponse {
    status: String,
    message: String,
    re_encrypted: usize,
    re_encrypt_errors: usize,
}

/// POST /api/v1/auth/rotate-key
/// 生成新盐值 + 主密钥，可选批量重加密已有文件
pub async fn rotate_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<RotateKeyRequest>,
) -> Result<Json<RotateKeyResponse>, AppError> {
    let re_encrypt_path = body.re_encrypt_path.clone();
    let password = body.into_password();

    // 验证当前密码
    let old_salt_b64 = state
        .get_salt()
        .ok_or_else(|| AppError::BadRequest("配置中缺少盐值".into()))?;

    use base64::Engine;
    let old_salt = base64::engine::general_purpose::STANDARD
        .decode(&old_salt_b64)
        .map_err(|e| AppError::BadRequest(format!("盐值解码失败: {}", e)))?;

    let old_key = CryptoManager::derive_key_from_password(
        &password,
        &old_salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密码错误: {}", e)))?;

    // 生成新盐值 + 密钥
    let mut new_salt = [0u8; 16];
    getrandom::getrandom(&mut new_salt)
        .map_err(|e| AppError::CryptoError(format!("生成盐值失败: {}", e)))?;
    let new_salt_b64 = base64::engine::general_purpose::STANDARD.encode(new_salt);

    let new_key = CryptoManager::derive_key_from_password(
        &password,
        &new_salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密钥派生失败: {}", e)))?;

    // 保存到磁盘
    leolock::keymgmt::KeyManager::save_key(&new_key)?;

    let mut config = leolock::config::Config::load().unwrap_or_default();
    config.core.salt = Some(new_salt_b64.clone());
    config
        .save()
        .map_err(|e| AppError::Internal(format!("保存配置失败: {}", e)))?;

    // 可选批量重加密
    let mut re_encrypted = 0usize;
    let mut re_encrypt_errors = 0usize;

    if let Some(ref path_str) = re_encrypt_path {
        let dir = PathBuf::from(path_str);
        if dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if !file_path.is_file() || !file_path.extension().map_or(false, |e| e == "leo")
                    {
                        continue;
                    }
                    match CryptoManager::decrypt_file_v2(&file_path, &old_key, true) {
                        Ok(decrypted_path) => {
                            match CryptoManager::encrypt_file_v2(
                                &decrypted_path,
                                &new_key,
                                false,
                                true,
                            ) {
                                Ok(_) => {
                                    // 删除解密出的中间文件
                                    let _ = std::fs::remove_file(&decrypted_path);
                                    // 删除旧的加密文件
                                    let _ = leolock::utils::Utils::secure_delete_file(&file_path);
                                    re_encrypted += 1;
                                }
                                Err(_) => {
                                    re_encrypt_errors += 1;
                                }
                            }
                        }
                        Err(_) => {
                            re_encrypt_errors += 1;
                        }
                    }
                }
            }
        }
    }

    // 更新运行时状态（新密钥即时生效）
    state.update_salt(new_salt_b64);
    state.unlock(new_key);

    Ok(Json(RotateKeyResponse {
        status: "rotated".into(),
        message: format!(
            "🔑 主密钥已轮换{}",
            if re_encrypted > 0 {
                format!("，{} 个文件已重加密", re_encrypted)
            } else {
                String::new()
            }
        ),
        re_encrypted,
        re_encrypt_errors,
    }))
}

// ─── 备份下载 ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct BackupRequest {
    password: String,
}

impl BackupRequest {
    fn into_password(self) -> Zeroizing<String> {
        Zeroizing::new(self.password)
    }
}

/// POST /api/v1/backup
/// 生成加密密钥备份文件并返回（可反复调用）
pub async fn backup(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BackupRequest>,
) -> Result<Response, AppError> {
    let password = body.into_password();
    let key = state.get_key().ok_or(AppError::Locked)?;

    // 验证密码
    let salt_b64 = state
        .get_salt()
        .ok_or_else(|| AppError::BadRequest("配置中缺少盐值".into()))?;
    use base64::Engine;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(&salt_b64)
        .map_err(|e| AppError::BadRequest(format!("盐值解码失败: {}", e)))?;
    CryptoManager::derive_key_from_password(
        &password,
        &salt,
        state.argon2_m,
        state.argon2_t,
        state.argon2_p,
    )
    .map_err(|e| AppError::CryptoError(format!("密码错误: {}", e)))?;

    // 生成备份到临时文件
    let key_z = zeroize::Zeroizing::new(key);
    let backup_path = leolock::keymgmt::KeyManager::create_backup(&key_z, &password)
        .map_err(|e| AppError::Internal(format!("创建备份失败: {}", e)))?;

    let data = std::fs::read(&backup_path)
        .map_err(|e| AppError::Internal(format!("读取备份失败: {}", e)))?;
    let _ = std::fs::remove_file(&backup_path);

    let filename = backup_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(Body::from(data))
        .unwrap())
}

// ─── 备份恢复 ──────────────────────────────────────────────────

/// POST /api/v1/recover
/// multipart: 备份文件 + password 字段
pub async fn recover(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<MessageResponse>, AppError> {
    let mut backup_data: Option<Vec<u8>> = None;
    let mut password: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name().unwrap_or("") {
            "backup" => {
                backup_data = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?
                        .to_vec(),
                );
            }
            "password" => {
                password = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("读取密码失败: {}", e)))?,
                );
            }
            _ => {}
        }
    }

    let backup_data = backup_data.ok_or_else(|| AppError::BadRequest("缺少 backup 字段".into()))?;
    let password =
        Zeroizing::new(password.ok_or_else(|| AppError::BadRequest("缺少 password 字段".into()))?);

    // 写备份到临时文件
    let tmp_dir =
        tempfile::tempdir().map_err(|e| AppError::Internal(format!("创建临时目录失败: {}", e)))?;
    let tmp_path = tmp_dir.path().join("backup.enc");
    std::fs::write(&tmp_path, &backup_data)
        .map_err(|e| AppError::Internal(format!("写入临时文件失败: {}", e)))?;

    let recovered_key = leolock::keymgmt::KeyManager::recover_from_backup(&tmp_path, &password)
        .map_err(|e| AppError::CryptoError(format!("恢复失败: {}", e)))?;

    // 保存恢复的密钥并更新运行状态
    leolock::keymgmt::KeyManager::save_key(&recovered_key)?;
    state.unlock(recovered_key);

    Ok(Json(MessageResponse {
        status: "recovered".into(),
        message: "✅ 密钥已从备份恢复，服务已解锁".into(),
    }))
}

// ─── 分享链接 ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub(crate) struct CreateShareRequest {
    /// 文件 ID（base64 编码路径，从 /api/v1/files 列表获取）
    file_id: String,
    /// 过期时间（秒），默认 3600
    expires_in: Option<u64>,
    /// 最大下载次数，默认 1
    max_downloads: Option<u32>,
    /// 分享密码（可选）
    password: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct CreateShareResponse {
    token: String,
    url: String,
    expires_at: String,
    max_downloads: u32,
}

/// POST /api/v1/share — 创建分享链接（需要认证 + 服务已解锁）
pub async fn create_share(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateShareRequest>,
) -> Result<Json<CreateShareResponse>, AppError> {
    let _key = state.get_key().ok_or(AppError::Locked)?;
    let file_path = decode_id(&body.file_id)?;

    if !file_path.exists() {
        return Err(AppError::BadRequest(format!(
            "文件不存在: {}",
            file_path.display()
        )));
    }

    // 生成随机 token
    let mut raw = [0u8; 32];
    getrandom::getrandom(&mut raw)
        .map_err(|e| AppError::CryptoError(format!("生成 token 失败: {}", e)))?;
    use base64::Engine;
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw);

    let expires_in = body.expires_in.unwrap_or(3600);
    let max_downloads = body.max_downloads.unwrap_or(1);

    let info = crate::state::ShareInfo {
        file_path: file_path.to_string_lossy().to_string(),
        expires_at: std::time::Instant::now() + std::time::Duration::from_secs(expires_in),
        max_downloads,
        download_count: 0,
        password: body.password.clone(),
    };

    state.shares.lock().await.insert(token.clone(), info);

    let addr = "127.0.0.1:3000"; // 简化：实际应从请求头获取
    Ok(Json(CreateShareResponse {
        url: format!("http://{}/api/v1/share/download?token={}", addr, token),
        token,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::TimeDelta::seconds(expires_in as i64))
            .unwrap_or_default()
            .to_rfc3339(),
        max_downloads,
    }))
}

/// GET /api/v1/share/{token} — 下载分享文件（公开，无需认证）
pub async fn download_share(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Response, AppError> {
    let token = params.get("token").cloned().unwrap_or_default();
    let mut guard = state.shares.lock().await;
    let info = guard
        .get_mut(&token)
        .ok_or_else(|| AppError::BadRequest("分享链接无效或已过期".into()))?;

    // 检查过期
    if std::time::Instant::now() > info.expires_at {
        let _ = guard.remove(&token);
        return Err(AppError::BadRequest("分享链接已过期".into()));
    }

    // 检查密码
    if let Some(ref share_pw) = info.password {
        let provided = params.get("password").cloned().unwrap_or_default();
        if provided != *share_pw {
            return Err(AppError::BadRequest("分享密码错误".into()));
        }
    }

    // 检查下载次数
    if info.download_count >= info.max_downloads {
        let _ = guard.remove(&token);
        return Err(AppError::BadRequest("下载次数已用完".into()));
    }

    let file_path = PathBuf::from(&info.file_path);
    let file_data = std::fs::read(&file_path)?;

    let key = state.get_key().ok_or(AppError::Locked)?;
    let (original_name, decrypted) = CryptoManager::decrypt_data_v3(&file_data, &key)?;

    info.download_count += 1;

    // 用完自动清理
    if info.download_count >= info.max_downloads {
        let _ = guard.remove(&token);
    }

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", original_name),
        )
        .body(Body::from(decrypted))
        .unwrap())
}

/// DELETE /api/v1/share/{token} — 撤销分享（需要认证）
pub async fn revoke_share(
    State(state): State<Arc<AppState>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<MessageResponse>, AppError> {
    let token = params.get("token").cloned().unwrap_or_default();
    let removed = state.shares.lock().await.remove(&token).is_some();
    if removed {
        Ok(Json(MessageResponse {
            status: "revoked".into(),
            message: "分享链接已撤销".into(),
        }))
    } else {
        Err(AppError::BadRequest("分享链接不存在".into()))
    }
}
