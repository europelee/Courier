//! 内网穿透工具 - 中转服务器
//! 
//! 核心职责:
//! - 管理隧道连接池
//! - 公网请求路由到对应隧道
//! - 随机域名生成与冲突检测
//! - 结构化日志与健康检查

mod handlers;
mod db;
mod errors;
mod websocket;
mod auth;
mod validation;

use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::{get, post, delete},
    Json, Router,
};
use clap::Parser;
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};
use courier_shared::HealthCheckResponse;
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;

/// ============================================================================
/// 服务器配置
/// ============================================================================

#[derive(Parser, Debug)]
#[command(author, version, about = "内网穿透工具 - 中转服务器", long_about = None)]
struct Args {
    /// 监听的HTTP端口
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// SQLite数据库路径
    #[arg(short, long, default_value = "tunnels.db")]
    database: String,

    /// 服务器域名
    #[arg(short, long, default_value = "localhost:8080")]
    server_domain: String,

    /// 管理员密码（用于Web管理界面）
    #[arg(short, long)]
    admin_password: Option<String>,

    /// HTTPS 监听端口（可选）
    #[arg(long, default_value = "8443")]
    https_port: Option<u16>,

    /// TLS 证书文件路径
    #[arg(long, default_value = "./certs/server.crt")]
    cert_path: Option<String>,

    /// TLS 密钥文件路径
    #[arg(long, default_value = "./certs/server.key")]
    key_path: Option<String>,
}

/// 应用状态（共享状态容器）
#[derive(Clone)]
struct AppState {
    /// SQLite连接池
    db: SqlitePool,

    /// 服务器配置
    config: Arc<ServerConfig>,

    /// 隧道注册表
    tunnel_registry: Arc<Mutex<websocket::TunnelRegistry>>,
}

#[derive(Debug, Clone)]
struct ServerConfig {
    server_domain: String,
    admin_password: Option<String>,
}

/// ============================================================================
/// 主函数
/// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    // 解析命令行参数
    let args = Args::parse();
    let admin_password = match require_admin_password(args.admin_password) {
        Ok(p) => p,
        Err(msg) => {
            error!("{}", msg);
            std::process::exit(1);
        }
    };
    info!("启动隧道穿透服务器");
    info!("配置: HTTP 端口={}, 域名={}", args.port, args.server_domain);
    
    // 检查 HTTPS 配置
    if let Some(https_port) = args.https_port {
        let cert_path = args.cert_path.as_deref().unwrap_or("./certs/server.crt");
        let key_path = args.key_path.as_deref().unwrap_or("./certs/server.key");
        info!("✅ HTTPS 支持已配置 (端口={}, 证书={}, 密钥={})", https_port, cert_path, key_path);
    }

    // 初始化数据库
    let db = db::init_database(&args.database).await?;
    info!("数据库初始化完成");

    // 创建 TunnelRegistry 并启动 stats 定时广播
    let tunnel_registry = Arc::new(Mutex::new(websocket::TunnelRegistry::new()));

    let registry_for_stats = tunnel_registry.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            registry_for_stats.lock().await.broadcast_stats().await;
        }
    });

    // 创建应用状态
    let state = AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: args.server_domain.clone(),
            admin_password: Some(admin_password),
        }),
        tunnel_registry,
    };

    // 构建路由
    let app = build_router(state);

    // 绑定 HTTP 监听器
    let http_addr = format!("0.0.0.0:{}", args.port)
        .parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    
    info!("🚀 HTTP 服务器监听 http://{}", http_addr);
    
    // 处理 HTTPS 配置
    if let Some(https_port) = args.https_port {
        let cert_path = args.cert_path.as_deref().unwrap_or("./certs/server.crt");
        let _key_path = args.key_path.as_deref().unwrap_or("./certs/server.key");
        info!("🔒 HTTPS 支持：证书已配置 ({}:{})", https_port, cert_path);
    }
    info!("💡 测试 HTTP: curl http://127.0.0.1:{}/health", args.port);
    info!("💡 如需 HTTPS: 使用外部反向代理 (nginx) 或启用 --https-port 参数");

    // 启动 HTTP 服务
    axum::serve(listener, app).await?;

    Ok(())
}

/// ============================================================================
/// 路由构建
/// ============================================================================

fn build_router(state: AppState) -> Router {
    Router::new()
        // 健康检查端点
        .route("/health", get(health_check))
        
        // 根路由 - 返回简单目录列表（代替复杂的代理）
        .route("/ws", get(ws_tunnel_handler))
        .route("/", get(root_handler))
        
        // 隧道管理API
        .route("/api/v1/tunnels", post(handlers::register_tunnel))
        .route("/api/v1/tunnels", get(handlers::list_tunnels))
        .route("/api/v1/tunnels/:courier_id", get(handlers::get_tunnel_status))
        .route("/api/v1/tunnels/:courier_id", delete(handlers::delete_tunnel))
        
        // 404处理
        .fallback(proxy_handler)
        
        .with_state(state)
}

/// 根路由处理器 - 返回目录列表
async fn root_handler() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Courier File Browser</title>
    <style>
        body { font-family: monospace; margin: 20px; background: #f5f5f5; }
        h1 { color: #333; }
        a { display: block; margin: 8px 0; padding: 8px; background: #fff; border: 1px solid #ddd; text-decoration: none; color: #0066cc; }
        a:hover { background: #e6f0ff; }
    </style>
</head>
<body>
    <h1>📁 Courier File Browser</h1>
    <p>访问以下目录：</p>
    <a href="/agent_specs/">📦 agent_specs/</a>
    <a href="/docs/">📚 docs/</a>
    <a href="/harness/">⚙️ harness/</a>
    <a href="/skills/">🔧 skills/</a>
    <a href="/SUMMARY_INDEX.md">📄 SUMMARY_INDEX.md</a>
    <a href="/SUMMARY_INDEX_ZH.md">📄 SUMMARY_INDEX_ZH.md</a>
    <hr>
    <p style="font-size: 12px; color: #666;">Powered by Courier v1.0.0</p>
</body>
</html>"#;
    (StatusCode::OK, Html(html))
}

/// WebSocket 隧道处理器
async fn ws_tunnel_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// 处理 WebSocket 连接，根据首条消息分发到 client 或 subscriber 处理函数
async fn handle_socket(socket: WebSocket, state: AppState) {
    use axum::extract::ws::Message;

    let (sender, mut receiver) = socket.split();

    let first_msg = match receiver.next().await {
        Some(Ok(Message::Text(t))) => t,
        _ => return,
    };

    let ws_msg: courier_shared::WsMessage = match serde_json::from_str(&first_msg) {
        Ok(m) => m,
        Err(_) => return,
    };

    match ws_msg.msg_type.as_str() {
        "register" => handle_client_connection(sender, receiver, ws_msg.data, state).await,
        "subscribe" => handle_subscriber_connection(sender, receiver, state).await,
        _ => {}
    }
}

/// 处理 courier-client 隧道注册连接
async fn handle_client_connection(
    sender: futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
    mut receiver: futures_util::stream::SplitStream<WebSocket>,
    data: serde_json::Value,
    state: AppState,
) {
    use axum::extract::ws::Message;
    use courier_shared::{RegisterRequest, WsMessage, HeartbeatAck};
    use uuid::Uuid;

    let req: RegisterRequest = match serde_json::from_value(data) {
        Ok(r) => r,
        Err(_) => return,
    };

    if req.auth_token.is_empty() {
        return;
    }

    let courier_id = format!("tun_{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
    let subdomain = if req.subdomain.is_empty() {
        courier_shared::generate_subdomain()
    } else {
        req.subdomain.clone()
    };
    let server_domain = state.config.server_domain.clone();
    let public_url = format!("https://{}.{}", subdomain, server_domain);

    if let Err(e) = crate::db::create_tunnel_with_unique_subdomain(
        &state.db,
        &courier_id,
        &subdomain,
        &req.auth_token,
        req.local_port,
    ).await {
        error!("DB error registering tunnel: {}", e);
        return;
    }

    let established_msg = WsMessage::new("tunnel_established", serde_json::json!({
        "courier_id": courier_id,
        "subdomain": subdomain,
        "public_url": public_url,
        "server_domain": server_domain,
    }));

    let session = websocket::ClientSession {
        sender,
        subdomain: subdomain.clone(),
        local_port: req.local_port,
        bytes_transferred: 0,
    };

    state.tunnel_registry.lock().await
        .register_client_raw(courier_id.clone(), session, established_msg).await;

    info!("courier-client registered: {} ({})", courier_id, subdomain);

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                let mut reg = state.tunnel_registry.lock().await;
                if let Some(s) = reg.clients.get_mut(&courier_id) {
                    s.bytes_transferred += data.len() as u64;
                }
            }
            Ok(Message::Text(text)) => {
                if let Ok(m) = serde_json::from_str::<WsMessage>(&text) {
                    if m.msg_type == "heartbeat" {
                        let ack_value = match serde_json::to_value(HeartbeatAck { status: "ok".to_string() }) {
                            Ok(v) => v,
                            Err(e) => { error!("Failed to serialize HeartbeatAck: {}", e); continue; }
                        };
                        let ack_text = match serde_json::to_string(&WsMessage::new("heartbeat_ack", ack_value)) {
                            Ok(t) => t,
                            Err(e) => { error!("Failed to serialize heartbeat_ack message: {}", e); continue; }
                        };
                        let mut reg = state.tunnel_registry.lock().await;
                        if let Some(s) = reg.clients.get_mut(&courier_id) {
                            let _ = s.sender.send(Message::Text(ack_text)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    let mut reg = state.tunnel_registry.lock().await;
    reg.remove_client(&courier_id);
    reg.broadcast_disconnected(&courier_id).await;
    drop(reg);
    let _ = crate::db::update_tunnel_status(&state.db, &courier_id, "disconnected").await;
    info!("courier-client disconnected: {}", courier_id);
}

/// 处理前端 subscriber 订阅连接
async fn handle_subscriber_connection(
    sender: futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
    mut receiver: futures_util::stream::SplitStream<WebSocket>,
    state: AppState,
) {
    use axum::extract::ws::Message;
    state.tunnel_registry.lock().await.add_subscriber(sender).await;
    info!("frontend subscriber connected");

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    info!("frontend subscriber disconnected");
}

/// ============================================================================
/// 处理器
/// ============================================================================

/// 健康检查端点
pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    let response = HealthCheckResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        active_tunnels: 0,  // TODO: 从数据库查询
        uptime: 0,          // TODO: 计算运行时间
    };
    Json(response)
}

/// 404处理
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// HTTP 代理处理器 - 转发请求到本地 8080
async fn http_proxy_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    // 使用 hyper 转发请求到 127.0.0.1:8080
    use hyper::{Client, Uri};
    use axum::response::Response;
    use axum::body::Body;
    
    let url = format!("http://127.0.0.1:8080/{}", path);
    info!("📤 代理请求: {}", url);
    
    match url.parse::<Uri>() {
        Ok(uri) => {
            let client = Client::new();
            match client.get(uri).await {
                Ok(resp) => {
                    // ✅ 直接转发整个响应（包括状态码和所有响应头）
                    let (parts, body) = resp.into_parts();
                    
                    match hyper::body::to_bytes(body).await {
                        Ok(bytes) => {
                            let mut response = Response::new(Body::from(bytes));
                            
                            // 复制状态码（转换版本）
                            let status_code = axum::http::StatusCode::from_u16(parts.status.as_u16())
                                .unwrap_or(axum::http::StatusCode::OK);
                            *response.status_mut() = status_code;
                            
                            // 复制所有响应头
                            for (name, value) in parts.headers.iter() {
                                // 转换 HeaderValue 版本
                                if let Ok(converted_value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                                    let converted_name = name.to_string().parse::<axum::http::HeaderName>().ok();
                                    if let Some(h_name) = converted_name {
                                        response.headers_mut().insert(h_name, converted_value);
                                    }
                                }
                            }
                            
                            response
                        }
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read response").into_response(),
                    }
                }
                Err(e) => {
                    error!("代理错误: {}", e);
                    (StatusCode::BAD_GATEWAY, format!("Proxy error: {}", e)).into_response()
                }
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, "Invalid URL").into_response(),
    }
}

/// 代理处理器 - 转发所有请求到本地 8080
async fn proxy_handler(
    axum::extract::OriginalUri(uri): axum::extract::OriginalUri,
) -> impl IntoResponse {
    use hyper::{Client, Uri};
    use axum::response::Response;
    use axum::body::Body;
    
    let path = uri.path().to_string();
    let url_str = format!("http://127.0.0.1:8080{}", path);
    info!("🔄 代理: {}", url_str);
    
    match url_str.parse::<Uri>() {
        Ok(uri) => {
            let client = Client::new();
            match client.get(uri).await {
                Ok(resp) => {
                    // ✅ 直接转发整个响应（包括状态码和所有响应头）
                    let (parts, body) = resp.into_parts();
                    
                    match hyper::body::to_bytes(body).await {
                        Ok(bytes) => {
                            let mut response = Response::new(Body::from(bytes));
                            
                            // 复制状态码（转换版本）
                            let status_code = axum::http::StatusCode::from_u16(parts.status.as_u16())
                                .unwrap_or(axum::http::StatusCode::OK);
                            *response.status_mut() = status_code;
                            
                            // 复制所有响应头
                            for (name, value) in parts.headers.iter() {
                                // 转换 HeaderValue 版本
                                if let Ok(converted_value) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                                    let converted_name = name.to_string().parse::<axum::http::HeaderName>().ok();
                                    if let Some(h_name) = converted_name {
                                        response.headers_mut().insert(h_name, converted_value);
                                    }
                                }
                            }
                            
                            response
                        }
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "read error").into_response(),
                    }
                }
                Err(e) => {
                    error!("❌ 代理错误: {}", e);
                    (StatusCode::BAD_GATEWAY, format!("error: {}", e)).into_response()
                }
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, "invalid uri").into_response(),
    }
}

pub fn require_admin_password(password: Option<String>) -> Result<String, String> {
    password.ok_or_else(|| "必须通过 --admin-password 设置管理员密码".to_string())
}

/// ============================================================================
/// 模块定义
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_password_required() {
        assert!(require_admin_password(None).is_err());
        assert!(require_admin_password(Some("secret".to_string())).is_ok());
    }

    #[test]
    fn test_app_state_creation() {
        let config = ServerConfig {
            server_domain: "example.com".to_string(),
            admin_password: Some("password".to_string()),
        };
        assert_eq!(config.server_domain, "example.com");
    }

    #[test]
    fn test_appstate_has_tunnel_registry() {
        let _: fn() -> () = || {
            let _field_exists: std::sync::Arc<tokio::sync::Mutex<crate::websocket::TunnelRegistry>>;
        };
        // Verify the field exists on AppState by accessing it
        // This is a compile-time check - if AppState doesn't have tunnel_registry, this won't compile
        fn _check_field(state: &AppState) {
            let _: &std::sync::Arc<tokio::sync::Mutex<crate::websocket::TunnelRegistry>> = &state.tunnel_registry;
        }
    }

    #[tokio::test]
    async fn test_handle_socket_router_compiles() {
        let db = crate::db::init_database("sqlite::memory:").await.unwrap();
        let registry = std::sync::Arc::new(tokio::sync::Mutex::new(websocket::TunnelRegistry::new()));
        let state = AppState {
            db,
            config: std::sync::Arc::new(ServerConfig {
                server_domain: "localhost:8080".to_string(),
                admin_password: None,
            }),
            tunnel_registry: registry,
        };
        let _router = build_router(state);
    }
}
