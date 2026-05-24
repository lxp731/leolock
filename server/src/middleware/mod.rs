use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

/// JWT 载荷
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    /// 签发者
    sub: String,
    /// 签发时间
    iat: usize,
    /// 过期时间
    exp: usize,
}

/// JWT 验证中间件
///
/// 从请求的 Authorization header 中提取并验证 JWT Token。
/// 验证通过则放行，失败则返回 401。
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    // 从 Authorization header 提取 token
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "缺少 Authorization header，请先登录",
            )
                .into_response();
        }
    };

    // 获取 JWT 密钥
    let secret = match &state.jwt_secret {
        Some(s) => s.clone(),
        None => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "JWT 未配置").into_response();
        }
    };

    // 验证 Token
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ) {
        Ok(_) => {
            // Token 有效，放行
            next.run(req).await
        }
        Err(_) => (StatusCode::UNAUTHORIZED, "Token 无效或已过期，请重新登录").into_response(),
    }
}

/// 签发 JWT Token
pub fn issue_token(secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    use chrono::Utc;

    let now = Utc::now();
    let claims = Claims {
        sub: "leolock-api".into(),
        iat: now.timestamp() as usize,
        exp: (now.timestamp() + 1800) as usize, // 30 分钟过期
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}
