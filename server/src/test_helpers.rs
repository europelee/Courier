use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use crate::{AppState, ServerConfig, websocket::TunnelRegistry, build_router};
use crate::access_log::LogEntry;

pub async fn make_app_state(password: &str) -> AppState {
    let db = crate::db::init_database("sqlite::memory:").await
        .expect("test DB init failed");
    let (log_tx, _log_rx) = mpsc::channel::<LogEntry>(100);
    AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: "localhost:8080".to_string(),
            admin_password: Some(password.to_string()),
        }),
        tunnel_registry: Arc::new(Mutex::new(TunnelRegistry::new())),
        log_tx,
    }
}

pub async fn make_test_router(password: &str) -> axum::Router {
    let state = make_app_state(password).await;
    build_router(state)
}
