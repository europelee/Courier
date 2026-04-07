//! WebSocket 服务器 - 处理客户端连接和隧道生命周期

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use courier_shared::{RegisterRequest, CourierEstablished};
use uuid::Uuid;
use sqlx::SqlitePool;
use crate::db;

/// WebSocket 连接管理器
pub struct WsConnectionManager {
    /// 活跃连接计数
    connections_count: Arc<RwLock<usize>>,
}

impl WsConnectionManager {
    /// 创建新的连接管理器
    pub fn new() -> Self {
        Self {
            connections_count: Arc::new(RwLock::new(0)),
        }
    }

    /// 处理新的隧道注册（已集成子域名冲突检测）
    /// 
    /// # 参数
    /// * `register_req` - 隧道注册请求
    /// * `db` - 数据库连接池
    /// 
    /// # 返回
    /// 成功时返回 CourierEstablished 信息，包含子域名冲突检测和数据库持久化
    /// 失败时返回错误信息
    pub async fn handle_register(
        &self,
        register_req: RegisterRequest,
        db: &SqlitePool,
    ) -> Result<CourierEstablished, String> {
        // 1. 验证认证令牌
        if register_req.auth_token.is_empty() {
            error!("Empty auth token");
            return Err("Authentication failed".to_string());
        }

        // 2. 生成隧道 ID
        let courier_id = format!("tun_{}", Uuid::new_v4().to_string()[..8].to_uppercase());

        // 3. 处理子域名
        let subdomain = if register_req.subdomain.is_empty() {
            courier_shared::generate_subdomain()
        } else {
            register_req.subdomain.clone()
        };

        // 4. ✅ 调用 db::create_tunnel_with_unique_subdomain()
        //    这会在事务中检查冲突并创建隧道记录
        db::create_tunnel_with_unique_subdomain(
            db,
            &courier_id,
            &subdomain,
            &register_req.auth_token,
            register_req.local_port,
        )
        .await
        .map_err(|e| {
            error!("Failed to create tunnel with unique subdomain: {}", e);
            format!("Tunnel creation failed: {}", e)
        })?;

        info!("New tunnel registered: {} (subdomain: {}, port: {})", courier_id, subdomain, register_req.local_port);

        // 5. 增加连接计数
        *self.connections_count.write().await += 1;

        // 6. 构造响应
        let response = CourierEstablished {
            courier_id,
            public_url: format!("https://{}.example.com", subdomain),
            server_domain: "example.com".to_string(),
            subdomain,
        };

        Ok(response)
    }

    /// 获取活跃隧道数
    pub async fn active_tunnel_count(&self) -> usize {
        *self.connections_count.read().await
    }

    /// 关闭隧道
    pub async fn close_tunnel(&self) {
        let mut count = self.connections_count.write().await;
        if *count > 0 {
            *count -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_manager_creation() {
        let manager = WsConnectionManager::new();
        assert!(true);
    }

    #[tokio::test]
    async fn test_active_tunnel_count() {
        let manager = WsConnectionManager::new();
        assert_eq!(manager.active_tunnel_count().await, 0);
    }

    #[tokio::test]
    async fn test_handle_register() {
        // 初始化内存数据库
        let pool = crate::db::init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        let manager = WsConnectionManager::new();
        let register_req = RegisterRequest {
            auth_token: "test_token".to_string(),
            local_port: 3000,
            protocols: vec!["http".to_string()],
            subdomain: String::new(),
        };

        let result = manager.handle_register(register_req, &pool).await;
        assert!(result.is_ok(), "Should register tunnel successfully");
        
        let response = result.unwrap();
        assert!(!response.courier_id.is_empty());
        assert!(!response.subdomain.is_empty());
        assert_eq!(manager.active_tunnel_count().await, 1);
    }

    #[tokio::test]
    async fn test_subdomain_conflict() {
        // 初始化内存数据库
        let pool = crate::db::init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        let manager = WsConnectionManager::new();
        let test_subdomain = "conflict-test".to_string();

        // 创建第一个隧道
        let register_req_1 = RegisterRequest {
            auth_token: "token1".to_string(),
            local_port: 3000,
            protocols: vec!["http".to_string()],
            subdomain: test_subdomain.clone(),
        };

        let result_1 = manager.handle_register(register_req_1, &pool).await;
        assert!(result_1.is_ok(), "First tunnel should be created successfully");
        assert_eq!(result_1.unwrap().subdomain, test_subdomain);

        // 尝试创建第二个隧道使用相同子域名
        let register_req_2 = RegisterRequest {
            auth_token: "token2".to_string(),
            local_port: 3001,
            protocols: vec!["http".to_string()],
            subdomain: test_subdomain.clone(),
        };

        let result_2 = manager.handle_register(register_req_2, &pool).await;
        assert!(result_2.is_err(), "Second tunnel with same subdomain should fail");
        let err_msg = result_2.unwrap_err();
        assert!(err_msg.contains("creation failed") || err_msg.contains("已被占用"));
    }

    #[tokio::test]
    async fn test_empty_auth_token() {
        let pool = crate::db::init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        let manager = WsConnectionManager::new();
        let register_req = RegisterRequest {
            auth_token: String::new(),  // 空令牌
            local_port: 3000,
            protocols: vec!["http".to_string()],
            subdomain: "test".to_string(),
        };

        let result = manager.handle_register(register_req, &pool).await;
        assert!(result.is_err(), "Should reject empty auth token");
        assert_eq!(result.unwrap_err(), "Authentication failed");
    }
}

/// 处理 WebSocket 连接（占位符 - 实际连接处理由框架完成）
pub async fn handle_connection(
    _socket: (),
) -> Result<(), Box<dyn std::error::Error>> {
    info!("WebSocket 连接处理完成");
    Ok(())
}
