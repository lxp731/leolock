mod middleware;
mod routes;
mod state;

use axum::{
    middleware as axum_mw,
    routing::{delete, get, post},
    Router,
};
use leolock::config::Config;
use std::net::SocketAddr;
use std::sync::Arc;
use tower::limit::ConcurrencyLimitLayer;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;

const MAX_REQUEST_BYTES: usize = 2 * 1024 * 1024 * 1024; // 2GB
const MAX_CONCURRENT: usize = 8;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 启动时一次加载配置，缓存到 AppState
    let config = Config::load().unwrap_or_default();

    let app_state = Arc::new(state::AppState::new(
        config.auth.jwt_secret.clone(),
        config.core.salt.clone(),
        config.auth.api_key_hash.clone(),
        config.is_initialized(),
        config.core.argon2_m_cost,
        config.core.argon2_t_cost,
        config.core.argon2_p_cost,
    ));

    let has_api_key = app_state.has_api_key();
    let addr = format!("{}:{}", config.api.bind_address, config.api.port);

    // 公开路由（无需 Token）
    let public = Router::new()
        .route("/api/v1/health", get(routes::health))
        .route("/api/v1/status", get(routes::status))
        .route("/api/v1/auth/login", post(routes::login))
        .route("/api/v1/init", post(routes::init));

    // 受保护路由（需要 Token）
    let protected = Router::new()
        .route("/api/v1/auth/rotate-api-key", post(routes::rotate_api_key))
        .route("/api/v1/auth/rotate-key", post(routes::rotate_key))
        .route("/api/v1/unlock", post(routes::unlock))
        .route("/api/v1/lock", post(routes::lock))
        .route("/api/v1/encrypt", post(routes::encrypt))
        .route("/api/v1/decrypt", post(routes::decrypt))
        .route("/api/v1/encrypt-stream", post(routes::encrypt_stream))
        .route("/api/v1/decrypt-stream", post(routes::decrypt_stream))
        .route(
            "/api/v1/config",
            get(routes::get_config).put(routes::update_config),
        )
        .route("/api/v1/files", get(routes::list_files))
        .route("/api/v1/files/get", get(routes::get_file))
        .route("/api/v1/files/download", get(routes::download_file))
        .route("/api/v1/files/delete", delete(routes::delete_file))
        .route("/api/v1/stats", get(routes::stats))
        .layer(axum_mw::from_fn_with_state(
            app_state.clone(),
            middleware::auth_middleware,
        ));

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .layer(axum_mw::from_fn(middleware::logging_middleware))
        .layer(ConcurrencyLimitLayer::new(MAX_CONCURRENT))
        .layer(RequestBodyLimitLayer::new(MAX_REQUEST_BYTES))
        .layer(CorsLayer::permissive())
        .with_state(app_state);

    println!("🚀 LeoLock API 服务已启动");
    println!("   地址: http://{}", addr);
    println!("   状态: 🔒 LOCKED（需要 POST /api/v1/unlock 解锁）");
    if has_api_key {
        println!("   🔐 鉴权已启用（需要 POST /api/v1/auth/login 获取 Token）");
    } else {
        println!("   ⚠️  未配置 API Key，请运行 leolock init 生成");
    }
    println!();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
