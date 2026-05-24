use axum::{
    extract::{Multipart, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
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
            AppError::Locked => (StatusCode::LOCKED, "🔒 服务已锁定，请先 POST /api/v1/unlock".into()),
            AppError::NotInitialized => (StatusCode::PRECONDITION_FAILED, "❌ 未初始化，请先执行 leolock init".into()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::CryptoError(msg) => (StatusCode::BAD_REQUEST, format!("加密错误: {}", msg)),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };
        (status, Json(MessageResponse { status: "error".into(), message })).into_response()
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
                data = Some(field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("读取文件失败: {}", e))
                })?.to_vec());
            }
            _ => {}
        }
    }

    Ok(MultipartFile {
        data: data.ok_or_else(|| AppError::BadRequest("缺少 file 字段".into()))?,
        file_name: file_name.unwrap_or_else(|| "unknown".to_string()),
    })
}

// ─── 鉴权：登录 ────────────────────────────────────────────────

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    if !state.has_api_key() {
        return Err(AppError::BadRequest("服务未生成 API Key，请先执行 leolock init".into()));
    }

    if !state.verify_api_key(&body.api_key) {
        return Err(AppError::BadRequest("API Key 无效".into()));
    }

    let secret = state.jwt_secret.as_ref()
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

pub async fn status(
    State(state): State<Arc<AppState>>,
) -> Json<StatusResponse> {
    Json(StatusResponse {
        initialized: state.is_initialized,
        locked: !state.is_unlocked(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

// ─── 锁定 / 解锁 ───────────────────────────────────────────────

pub async fn unlock(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UnlockRequest>,
) -> Result<Json<MessageResponse>, AppError> {
    let password = body.into_password();

    if !state.is_initialized {
        return Err(AppError::NotInitialized);
    }

    let salt_b64 = state.salt.as_ref()
        .ok_or_else(|| AppError::BadRequest("配置中缺少盐值".into()))?;

    use base64::Engine;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(salt_b64)
        .map_err(|e| AppError::BadRequest(format!("盐值解码失败: {}", e)))?;

    let key = CryptoManager::derive_key_from_password(&password, &salt)
        .map_err(|e| AppError::CryptoError(format!("密钥派生失败: {}", e)))?;

    state.unlock(key);

    Ok(Json(MessageResponse {
        status: "unlocked".into(),
        message: "🔓 服务已解锁，密钥已加载到内存".into(),
    }))
}

pub async fn lock(
    State(state): State<Arc<AppState>>,
) -> Json<MessageResponse> {
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
    let key = CryptoManager::derive_key_from_password(&password, &salt)
        .map_err(|e| AppError::CryptoError(format!("密钥派生失败: {}", e)))?;

    // 保存密钥
    leolock::keymgmt::KeyManager::save_key(&key)?;

    // 保存盐值
    config.salt = Some(salt_b64);

    // 生成 API Key 和 JWT Secret
    let api_key = config.generate_api_key()
        .map_err(|e| AppError::Internal(format!("生成 API Key 失败: {}", e)))?;
    config.generate_jwt_secret()
        .map_err(|e| AppError::Internal(format!("生成 JWT 密钥失败: {}", e)))?;

    config.save()
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
        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", hash)),
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
        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", original_name)),
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
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(id)
        .map_err(|_| AppError::BadRequest("无效的文件 ID".into()))?;
    let path_str = String::from_utf8(bytes)
        .map_err(|_| AppError::BadRequest("无效的文件 ID 编码".into()))?;
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
        return Err(AppError::BadRequest(format!("目录不存在: {}", dir.display())));
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
        return Err(AppError::BadRequest(format!("文件不存在: {}", file_path.display())));
    }

    let encrypted_data = std::fs::read(&file_path)?;
    let (original_name, decrypted) = CryptoManager::decrypt_data_v3(&encrypted_data, &key)?;

    let headers = [
        (header::CONTENT_TYPE, "application/octet-stream"),
        (header::CONTENT_DISPOSITION, &format!("attachment; filename=\"{}\"", original_name)),
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
        return Err(AppError::BadRequest(format!("文件不存在: {}", file_path.display())));
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
