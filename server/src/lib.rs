pub mod access_log;
pub mod auth;
pub mod db;
pub mod errors;
pub mod handlers;
pub mod middleware;
pub mod websocket;
pub mod validation;
pub mod test_helpers;

use axum::{
    extract::{State, ws::{WebSocket, WebSocketUpgrade}},
    http::StatusCode,
    response::{IntoResponse, Html},
    routing::{get, post, delete},
    Json, Router,
};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info, warn};
use courier_shared::HealthCheckResponse;
use futures_util::stream::StreamExt;
use futures_util::sink::SinkExt;

use crate::access_log::LogEntry;

/// 应用状态（共享状态容器）
#[derive(Clone)]
pub struct AppState {
    /// SQLite连接池
    pub db: SqlitePool,

    /// 服务器配置
    pub config: Arc<ServerConfig>,

    /// 隧道注册表
    pub tunnel_registry: Arc<Mutex<websocket::TunnelRegistry>>,

    /// 访问日志发送端
    pub log_tx: mpsc::Sender<LogEntry>,
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub server_domain: String,
    pub admin_password: Option<String>,
}

pub fn build_router(state: AppState) -> Router {
    use axum::middleware;
    use tower::ServiceBuilder;

    let protected = Router::new()
        .route("/api/v1/tunnels", post(handlers::register_tunnel))
        .route("/api/v1/tunnels", get(handlers::list_tunnels))
        .route("/api/v1/tunnels/:courier_id", get(handlers::get_tunnel_status))
        .route("/api/v1/tunnels/:courier_id", delete(handlers::delete_tunnel))
        .route("/api/v1/logs", get(handlers::get_logs))
        .layer(middleware::from_fn_with_state(state.clone(), auth::auth_middleware));

    let log_tx = state.log_tx.clone();

    Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws_tunnel_handler))
        .route("/", get(root_handler))
        .route("/api/v1/auth/login", post(handlers::login))
        .merge(protected)
        .fallback(proxy_handler)
        .layer(ServiceBuilder::new().layer(crate::middleware::AccessLogLayer::new(log_tx)))
        .with_state(state)
}

pub fn require_admin_password(password: Option<String>) -> Result<String, String> {
    password.ok_or_else(|| "必须通过 --admin-password 设置管理员密码".to_string())
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
    <h1>Courier File Browser</h1>
    <a href="/health">/health</a>
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
        "subscribe" => {
            let token = ws_msg.data
                .get("token")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let secret = match state.config.admin_password.as_deref() {
                Some(s) => s,
                None => {
                    if let Ok(mut ws) = sender.reunite(receiver) {
                        let _ = ws.close().await;
                    }
                    return;
                }
            };
            if auth::validate_auth_token(token, secret).is_err() {
                warn!("WebSocket subscribe rejected: invalid or missing token");
                if let Ok(mut ws) = sender.reunite(receiver) {
                    let _ = ws.close().await;
                }
                return;
            }
            handle_subscriber_connection(sender, receiver, state).await
        }
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

    // 记录隧道连接事件
    let connected_entry = LogEntry::TunnelConnected {
        tunnel_id: courier_id.clone(),
        subdomain: subdomain.clone(),
        local_port: req.local_port,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = state.log_tx.try_send(connected_entry);

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

    // 记录隧道断开事件
    let disconnected_entry = LogEntry::TunnelDisconnected {
        tunnel_id: courier_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let _ = state.log_tx.try_send(disconnected_entry);

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

/// 健康检查端点
pub async fn health_check(State(_state): State<AppState>) -> impl IntoResponse {
    let response = HealthCheckResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        active_tunnels: 0,
        uptime: 0,
    };
    Json(response)
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
    info!("proxy: {}", url_str);

    match url_str.parse::<Uri>() {
        Ok(uri) => {
            let client = Client::new();
            match client.get(uri).await {
                Ok(resp) => {
                    let (parts, body) = resp.into_parts();

                    match hyper::body::to_bytes(body).await {
                        Ok(bytes) => {
                            let mut response = Response::new(Body::from(bytes));

                            let status_code = axum::http::StatusCode::from_u16(parts.status.as_u16())
                                .unwrap_or(axum::http::StatusCode::OK);
                            *response.status_mut() = status_code;

                            for (name, value) in parts.headers.iter() {
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
                    error!("proxy error: {}", e);
                    (StatusCode::BAD_GATEWAY, format!("error: {}", e)).into_response()
                }
            }
        }
        Err(_) => (StatusCode::BAD_REQUEST, "invalid uri").into_response(),
    }
}
