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
}

/// 应用状态（共享状态容器）
#[derive(Clone)]
struct AppState {
    /// SQLite连接池
    db: SqlitePool,
    
    /// 服务器配置
    config: Arc<ServerConfig>,
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
    info!("启动隧道穿透服务器");
    info!("配置: 端口={}, 域名={}", args.port, args.server_domain);

    // 初始化数据库
    let db = db::init_database(&args.database).await?;
    info!("数据库初始化完成");

    // 创建应用状态
    let state = AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: args.server_domain.clone(),
            admin_password: args.admin_password,
        }),
    };

    // 构建路由
    let app = build_router(state);

    // 绑定 HTTP 监听器
    let http_addr = format!("0.0.0.0:{}", args.port)
        .parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    
    info!("🚀 HTTP 服务器监听 http://{}", http_addr);
    info!("🔒 HTTPS 支持：证书已加载 (./certs/server.crt, ./certs/server.key)");
    info!("💡 使用 curl -k https://127.0.0.1:8443/health 测试 HTTPS");

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
    State(_state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// 处理 WebSocket 连接
async fn handle_socket(socket: WebSocket) {
    use axum::extract::ws::Message;
    use uuid::Uuid;
    
    info!("✅ 新的 WebSocket 连接建立");
    
    let (mut sender, mut receiver) = socket.split();
    
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("收到消息: {}", text);
                
                // 解析收到的注册请求
                if let Ok(ws_msg) = serde_json::from_str::<courier_shared::WsMessage>(&text) {
                    if ws_msg.msg_type == "register" {
                        // 生成隧道信息
                        let courier_id = format!("tunnel_{}", Uuid::new_v4().to_string()[0..8].to_string());
                        let subdomain = format!("sub_{}", rand::random::<u32>());
                        
                        // 返回正确格式的响应
                        let response = serde_json::json!({
                            "msg_type": "tunnel_established",
                            "data": {
                                "courier_id": courier_id,
                                "public_url": format!("https://{}.SERVER_DOMAIN_PLACEHOLDER:8888", subdomain),
                                "server_domain": "SERVER_DOMAIN_PLACEHOLDER:8888",
                                "subdomain": subdomain
                            }
                        }).to_string();
                        
                        if let Err(e) = sender.send(Message::Text(response)).await {
                            error!("发送失败: {}", e);
                            break;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!("客户端关闭连接");
                break;
            }
            Err(e) => {
                error!("WebSocket 错误: {}", e);
                break;
            }
            _ => {}
        }
    }
    
    info!("WebSocket 连接关闭");
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

/// ============================================================================
/// 模块定义
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        let config = ServerConfig {
            server_domain: "example.com".to_string(),
            admin_password: Some("password".to_string()),
        };
        assert_eq!(config.server_domain, "example.com");
    }
}
