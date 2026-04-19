//! HTTP 请求处理器

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use courier_shared::{
    ApiRegisterRequest, ApiRegisterResponse, CourierError,
};
use uuid::Uuid;

use crate::AppState;

/// 隧道列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ListTunnelsResponse {
    pub tunnels: Vec<crate::db::Tunnel>,
    pub total: usize,
}

/// 获取所有隧道列表
/// 
/// GET /api/v1/tunnels
/// 
/// 返回: ListTunnelsResponse
pub async fn list_tunnels(
    State(state): State<AppState>,
) -> Result<Json<ListTunnelsResponse>, crate::errors::ApiError> {
    let tunnels = crate::db::list_all_tunnels(&state.db)
        .await
        .map_err(|e| crate::errors::ApiError::from(e))?;
    
    let total = tunnels.len();
    
    Ok(Json(ListTunnelsResponse { tunnels, total }))
}

/// 删除隧道
/// 
/// DELETE /api/v1/tunnels/:id
pub async fn delete_tunnel(
    State(state): State<AppState>,
    Path(courier_id): Path<String>,
) -> Result<StatusCode, crate::errors::ApiError> {
    crate::db::delete_tunnel(&state.db, &courier_id)
        .await
        .map_err(|e| crate::errors::ApiError::from(e))?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// 注册新隧道
/// 
/// POST /api/v1/tunnels
/// 
/// 请求体: ApiRegisterRequest
/// 返回: ApiRegisterResponse
pub async fn register_tunnel(
    State(state): State<AppState>,
    Json(req): Json<ApiRegisterRequest>,
) -> Result<impl IntoResponse, crate::errors::ApiError> {
    // 验证认证令牌
    // TODO: 实现实际的token验证逻辑
    if req.auth_token.is_empty() {
        return Err(CourierError::InvalidAuth("Auth token is empty".to_string()).into());
    }

    // 验证本地端口
    if !courier_shared::validate_local_port(req.local_port) {
        return Err(CourierError::InvalidLocalPort(req.local_port).into());
    }

    // 生成或验证子域名
    let subdomain = if req.subdomain.is_empty() {
        courier_shared::generate_subdomain()
    } else {
        if !courier_shared::validate_subdomain(&req.subdomain) {
            return Err(CourierError::InvalidRequest(
                "Invalid subdomain format".to_string(),
            ).into());
        }
        req.subdomain.clone()
    };

    // 检查子域名冲突
    if let Err(_) = crate::db::check_subdomain_conflict(&state.db, &subdomain).await {
        return Err(CourierError::SubdomainConflict(subdomain).into());
    }

    // 生成隧道ID
    let courier_id = format!("tun_{}", Uuid::new_v4().to_string()[..8].to_uppercase());

    // 构造公网URL
    let public_url = format!("https://{}.{}", subdomain, state.config.server_domain);

    // 在数据库中创建隧道记录
    crate::db::create_tunnel(
        &state.db,
        &courier_id,
        &subdomain,
        &req.auth_token,
        req.local_port,
    )
    .await
    .map_err(|e| crate::errors::ApiError::from(e))?;

    // 返回响应
    let response = ApiRegisterResponse {
        courier_id,
        public_url,
        server_domain: state.config.server_domain.clone(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// 获取隧道状态
/// 
/// GET /api/v1/tunnels/:courier_id
pub async fn get_tunnel_status(
    State(state): State<AppState>,
    Path(courier_id): Path<String>,
) -> Result<impl IntoResponse, crate::errors::ApiError> {
    // 从数据库查询隧道状态
    let tunnel = crate::db::get_tunnel(&state.db, &courier_id).await
        .map_err(|e| crate::errors::ApiError::from(e))?;

    let response = serde_json::json!({
        "courier_id": tunnel.id,
        "status": tunnel.status,
        "public_url": format!("https://{}.{}", tunnel.subdomain, state.config.server_domain),
        "connected_at": tunnel.created_at_iso,
        "bytes_transferred": tunnel.bytes_transferred,
    });

    Ok(Json(response))
}

#[derive(Debug, serde::Deserialize)]
pub struct LoginRequest {
    pub password: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: u64,
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, crate::errors::ApiError> {
    let password = req.password.ok_or_else(|| {
        crate::errors::ApiError::ValidationError("password 字段必填".to_string())
    })?;

    let admin_password = state.config.admin_password.as_deref().unwrap_or("");

    if !crate::auth::verify_password(&password, admin_password) {
        return Err(crate::errors::ApiError::Unauthorized("密码错误".to_string()));
    }

    let token = crate::auth::generate_token("admin".to_string(), 24, admin_password)
        .map_err(|e| crate::errors::ApiError::InternalError(e))?;

    Ok(Json(LoginResponse { token, expires_in: 86400 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_validation() {
        // 基础的处理器逻辑测试将在集成测试中进行
    }
}
