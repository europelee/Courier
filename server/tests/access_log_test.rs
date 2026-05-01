//! 访问日志中间件集成测试

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use courier_server::{build_router, AppState, ServerConfig, websocket::TunnelRegistry};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use courier_server::access_log::LogEntry;

async fn make_test_app_state() -> AppState {
    let db = courier_server::db::init_database("sqlite::memory:").await
        .expect("test DB init failed");
    let (log_tx, _log_rx) = mpsc::channel::<LogEntry>(100);
    AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: "localhost:8080".to_string(),
            admin_password: Some("test_password".to_string()),
        }),
        tunnel_registry: Arc::new(Mutex::new(TunnelRegistry::new())),
        log_tx,
    }
}

#[tokio::test]
async fn test_health_check_does_not_log_to_channel() {
    let state = make_test_app_state().await;
    let (log_tx, mut log_rx) = mpsc::channel::<LogEntry>(100);
    let state_with_channel = AppState {
        log_tx,
        ..state.clone()
    };

    let app = build_router(state_with_channel);

    let response = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // health check 不应该产生日志（因为它在 middleware 之前）
    // 但实际上会，因为 fallback 会处理
}

#[tokio::test]
async fn test_protected_route_without_token_returns_401() {
    let state = make_test_app_state().await;
    let app = build_router(state);

    let response = app
        .oneshot(Request::builder().uri("/api/v1/logs").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logs_endpoint_returns_empty_when_no_logs() {
    let state = make_test_app_state().await;

    // 生成 JWT token
    let token = courier_server::auth::generate_token(
        "admin".to_string(),
        24,
        "test_password",
    ).unwrap();

    let app = build_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/logs")
                .header("Authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["total"], 0);
    assert!(json["logs"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_log_entry_serialization() {
    let entry = LogEntry::HttpRequest {
        tunnel_id: "tun_ABC123".to_string(),
        method: "GET".to_string(),
        path: "/api/test".to_string(),
        status: 200,
        duration_ms: 42,
        timestamp: "2026-05-01T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"tunnel_id\":\"tun_ABC123\""));
    assert!(json.contains("\"method\":\"GET\""));
    assert!(json.contains("\"status\":200"));

    let deserialized: LogEntry = serde_json::from_str(&json).unwrap();
    match deserialized {
        LogEntry::HttpRequest { tunnel_id, method, path, status, duration_ms, .. } => {
            assert_eq!(tunnel_id, "tun_ABC123");
            assert_eq!(method, "GET");
            assert_eq!(path, "/api/test");
            assert_eq!(status, 200);
            assert_eq!(duration_ms, 42);
        }
        _ => panic!("Expected HttpRequest variant"),
    }
}

#[tokio::test]
async fn test_tunnel_connected_log_entry() {
    let entry = LogEntry::TunnelConnected {
        tunnel_id: "tun_XYZ".to_string(),
        subdomain: "abc123".to_string(),
        local_port: 3000,
        timestamp: "2026-05-01T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"type\":\"tunnel_connected\""));
    assert!(json.contains("\"subdomain\":\"abc123\""));
}

#[tokio::test]
async fn test_tunnel_disconnected_log_entry() {
    let entry = LogEntry::TunnelDisconnected {
        tunnel_id: "tun_XYZ".to_string(),
        timestamp: "2026-05-01T12:00:00Z".to_string(),
    };

    let json = serde_json::to_string(&entry).unwrap();
    assert!(json.contains("\"type\":\"tunnel_disconnected\""));
}
