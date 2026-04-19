//! 数据库操作模块

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use courier_shared::CourierError;
use tracing::info;

/// 隧道信息数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Courier {
    pub id: String,
    pub subdomain: String,
    pub auth_token: String,
    pub local_port: u16,
    pub status: String,
    pub created_at_iso: String,
    pub bytes_transferred: u64,
}

/// 初始化SQLite数据库
pub async fn init_database(db_path: &str) -> Result<SqlitePool, CourierError> {
    // 确保数据库目录存在
    if !db_path.starts_with(":memory:") {
        if let Some(parent) = std::path::Path::new(db_path).parent() {
            if parent.as_os_str().len() > 0 {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CourierError::DatabaseError(format!("创建目录失败: {}", e)))?;
            }
        }
    }

    // 构造连接字符串
    let db_url = if db_path.starts_with(":memory:") {
        "sqlite::memory:".to_string()
    } else {
        format!("sqlite:{}", db_path)
    };

    // 创建连接池
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .map_err(|e| CourierError::DatabaseError(format!("连接数据库失败: {}", e)))?;

    // 执行初始化SQL
    init_schema(&pool).await?;

    info!("数据库初始化完成: {}", db_path);
    Ok(pool)
}

/// 初始化数据库表结构
async fn init_schema(pool: &SqlitePool) -> Result<(), CourierError> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tunnels (
            id TEXT PRIMARY KEY,
            subdomain TEXT UNIQUE NOT NULL,
            auth_token TEXT NOT NULL,
            local_port INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'disconnected',
            created_at INTEGER NOT NULL,
            last_heartbeat INTEGER,
            bytes_transferred INTEGER NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS certificates (
            subdomain TEXT PRIMARY KEY,
            cert_der BLOB NOT NULL,
            private_key_der BLOB NOT NULL,
            expires_at INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS access_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            courier_id TEXT NOT NULL,
            remote_addr TEXT NOT NULL,
            path TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            bytes_sent INTEGER NOT NULL,
            timestamp INTEGER NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    // 创建索引
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_tunnels_subdomain ON tunnels(subdomain);")
        .execute(pool)
        .await
        .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_access_logs_courier_id ON access_logs(courier_id);")
        .execute(pool)
        .await
        .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// 检查子域名是否已被占用
pub async fn check_subdomain_conflict(pool: &SqlitePool, subdomain: &str) -> Result<(), CourierError> {
    let result = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) as cnt FROM tunnels WHERE subdomain = ?",
    )
    .bind(subdomain)
    .fetch_one(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    if result > 0 {
        return Err(CourierError::SubdomainConflict(subdomain.to_string()));
    }

    Ok(())
}

/// 在数据库中创建新隧道记录
pub async fn create_tunnel(
    pool: &SqlitePool,
    courier_id: &str,
    subdomain: &str,
    auth_token: &str,
    local_port: u16,
) -> Result<(), CourierError> {
    let now = Utc::now().timestamp();

    sqlx::query(
        r#"
        INSERT INTO tunnels (id, subdomain, auth_token, local_port, status, created_at)
        VALUES (?, ?, ?, ?, 'disconnected', ?)
        "#,
    )
    .bind(courier_id)
    .bind(subdomain)
    .bind(auth_token)
    .bind(local_port as i32)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// 从数据库查询隧道信息
pub async fn get_tunnel(pool: &SqlitePool, courier_id: &str) -> Result<Tunnel, CourierError> {
    let row = sqlx::query_as::<_, (String, String, String, i32, String, i64, i64)>(
        r#"
        SELECT id, subdomain, auth_token, local_port, status, created_at, bytes_transferred
        FROM tunnels
        WHERE id = ?
        "#,
    )
    .bind(courier_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    let (id, subdomain, auth_token, local_port, status, created_at, bytes_transferred) =
        row.ok_or_else(|| CourierError::TunnelNotFound(courier_id.to_string()))?;

    let created_at_iso = chrono::DateTime::<Utc>::from_timestamp(created_at, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();

    Ok(Tunnel {
        id,
        subdomain,
        auth_token,
        local_port: local_port as u16,
        status,
        created_at_iso,
        bytes_transferred: bytes_transferred as u64,
        user_id: None,
    })
}

/// 更新隧道状态
pub async fn update_tunnel_status(
    pool: &SqlitePool,
    courier_id: &str,
    status: &str,
) -> Result<(), CourierError> {
    sqlx::query("UPDATE tunnels SET status = ?, last_heartbeat = ? WHERE id = ?")
        .bind(status)
        .bind(Utc::now().timestamp())
        .bind(courier_id)
        .execute(pool)
        .await
        .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// 列出所有隧道
pub async fn list_all_tunnels(pool: &SqlitePool) -> Result<Vec<Tunnel>, CourierError> {
    let rows = sqlx::query_as::<_, (String, String, String, i32, String, i64, i64)>(
        r#"
        SELECT id, subdomain, auth_token, local_port, status, created_at, bytes_transferred
        FROM tunnels
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    let tunnels = rows
        .into_iter()
        .map(|(id, subdomain, auth_token, local_port, status, created_at, bytes_transferred)| {
            let created_at_iso = chrono::DateTime::<Utc>::from_timestamp(created_at, 0)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            Tunnel {
                id,
                subdomain,
                auth_token,
                local_port: local_port as u16,
                status,
                created_at_iso,
                bytes_transferred: bytes_transferred as u64,
                user_id: None,
            }
        })
        .collect();

    Ok(tunnels)
}

/// 检查子域名是否已被使用
/// 
/// # 参数
/// * `pool` - 数据库连接池
/// * `subdomain` - 要检查的子域名
/// 
/// # 返回
/// 如果子域名已被使用返回 Ok(true)，否则返回 Ok(false)
pub async fn is_subdomain_taken(pool: &SqlitePool, subdomain: &str) -> Result<bool, CourierError> {
    let result = sqlx::query_scalar::<_, String>(
        "SELECT id FROM tunnels WHERE subdomain = ? AND status = 'active' LIMIT 1"
    )
    .bind(subdomain)
    .fetch_optional(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(format!("子域名查询失败: {}", e)))?;
    
    Ok(result.is_some())
}

/// 使用事务创建隧道，确保子域名唯一性（Task 5）
/// 
/// # 参数
/// * `pool` - 数据库连接池
/// * `courier_id` - 隧道 ID
/// * `subdomain` - 子域名（必须唯一）
/// * `auth_token` - 认证令牌
/// * `local_port` - 本地端口
/// 
/// # 返回
/// 成功返回 Ok(())，子域名冲突返回 ConflictError
pub async fn create_tunnel_with_unique_subdomain(
    pool: &SqlitePool,
    courier_id: &str,
    subdomain: &str,
    auth_token: &str,
    local_port: u16,
) -> Result<(), CourierError> {
    // 在事务中执行，确保原子性
    let mut tx = pool.begin().await
        .map_err(|e| CourierError::DatabaseError(format!("开始事务失败: {}", e)))?;
    
    // 检查子域名是否已被使用
    let taken = sqlx::query_scalar::<_, String>(
        "SELECT id FROM tunnels WHERE subdomain = ? AND status = 'active' LIMIT 1"
    )
    .bind(subdomain)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| CourierError::DatabaseError(format!("子域名查询失败: {}", e)))?;
    
    if taken.is_some() {
        return Err(CourierError::SubdomainConflict(format!("子域名 {} 已被占用", subdomain)));
    }
    
    // 创建隧道
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO tunnels (id, subdomain, auth_token, local_port, status, created_at, bytes_transferred)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(courier_id)
    .bind(subdomain)
    .bind(auth_token)
    .bind(local_port as i32)
    .bind("active")
    .bind(now)
    .bind(0i64)
    .execute(&mut *tx)
    .await
    .map_err(|e| CourierError::DatabaseError(format!("创建隧道失败: {}", e)))?;
    
    tx.commit().await
        .map_err(|e| CourierError::DatabaseError(format!("提交事务失败: {}", e)))?;
    
    info!("隧道创建成功（唯一子域名）: {} - {}", courier_id, subdomain);
    
    Ok(())
}

/// 通过隧道 ID 查询隧道信息，返回 Option（不存在时返回 None）
pub async fn get_tunnel_by_id(pool: &SqlitePool, courier_id: &str) -> Result<Option<Tunnel>, CourierError> {
    let row = sqlx::query_as::<_, (String, String, String, i32, String, i64, i64)>(
        r#"
        SELECT id, subdomain, auth_token, local_port, status, created_at, bytes_transferred
        FROM tunnels
        WHERE id = ?
        "#,
    )
    .bind(courier_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    Ok(row.map(|(id, subdomain, auth_token, local_port, status, created_at, bytes_transferred)| {
        let created_at_iso = chrono::DateTime::<Utc>::from_timestamp(created_at, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();

        Tunnel {
            id,
            subdomain,
            auth_token,
            local_port: local_port as u16,
            status,
            created_at_iso,
            bytes_transferred: bytes_transferred as u64,
            user_id: None,
        }
    }))
}

/// 删除隧道
pub async fn delete_tunnel(pool: &SqlitePool, courier_id: &str) -> Result<(), CourierError> {
    sqlx::query("DELETE FROM tunnels WHERE id = ?")
        .bind(courier_id)
        .execute(pool)
        .await
        .map_err(|e| CourierError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_initialization() {
        let pool = init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        // 验证表存在
        let result = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tunnels'",
        )
        .fetch_one(&pool)
        .await
        .expect("Failed to query tables");

        assert_eq!(result, 1, "tunnels table should exist");
    }

    #[tokio::test]
    async fn test_create_and_get_tunnel() {
        let pool = init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        let courier_id = "tun_test123";
        let subdomain = "testdomain";
        let auth_token = "test_token";
        let local_port = 3000;

        create_tunnel(&pool, courier_id, subdomain, auth_token, local_port)
            .await
            .expect("Failed to create tunnel");

        let tunnel = get_tunnel(&pool, courier_id)
            .await
            .expect("Failed to get tunnel");

        assert_eq!(tunnel.id, courier_id);
        assert_eq!(tunnel.subdomain, subdomain);
        assert_eq!(tunnel.local_port, local_port);
    }

    #[tokio::test]
    async fn test_subdomain_conflict_detection() {
        let pool = init_database("sqlite::memory:")
            .await
            .expect("Failed to initialize database");

        let subdomain = "conflicted";

        // 创建第一个隧道
        create_tunnel(&pool, "tun_1", subdomain, "token1", 3000)
            .await
            .expect("Failed to create first tunnel");

        // 尝试创建第二个隧道使用同一子域名
        let result = create_tunnel(&pool, "tun_2", subdomain, "token2", 3001).await;

        // 应该失败（但实际上SQLite会返回unique constraint错误）
        assert!(result.is_err(), "Should not allow duplicate subdomain");
    }

    #[tokio::test]
    async fn test_update_tunnel_status() {
        let pool = init_database("sqlite::memory:").await.unwrap();
        create_tunnel_with_unique_subdomain(&pool, "tun_TEST", "testsub", "token", 3000)
            .await.unwrap();
        update_tunnel_status(&pool, "tun_TEST", "disconnected").await.unwrap();
        let tunnel = get_tunnel_by_id(&pool, "tun_TEST").await.unwrap().unwrap();
        assert_eq!(tunnel.status, "disconnected");
    }
}

// 缺失的 Tunnel 结构体定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tunnel {
    pub id: String,
    pub subdomain: String,
    pub auth_token: String,
    pub local_port: u16,
    pub status: String,
    pub created_at_iso: String,
    pub bytes_transferred: u64,
    pub user_id: Option<String>,
}
