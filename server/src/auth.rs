//! 认证模块 - JWT 令牌验证
//! 
//! 提供安全的 JWT 令牌生成和验证功能，包括：
//! - 令牌签名和验证
//! - 过期时间检查
//! - 令牌 ID（防重放攻击）

use axum::{
    extract::Request,
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use subtle::ConstantTimeEq;
use tracing::{error, info, warn};
use crate::errors::ApiError;
use crate::AppState;

/// JWT 声明（Claims）
/// 
/// 包含用户身份信息和令牌元数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// 用户 ID (subject)
    pub sub: String,
    
    /// 过期时间 (Unix 时间戳，秒)
    pub exp: u64,
    
    /// 签发时间 (Unix 时间戳，秒)
    pub iat: u64,
    
    /// 令牌 ID（用于防重放攻击）
    pub jti: String,
}

impl Claims {
    /// 创建新的 JWT 声明
    /// 
    /// # 参数
    /// * `user_id` - 用户 ID
    /// * `expires_in_hours` - 过期时间（小时）
    /// 
    /// # 返回
    /// 新的 Claims 对象
    pub fn new(user_id: String, expires_in_hours: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let exp = now + (expires_in_hours * 3600);
        
        // 简单的令牌 ID 生成（实际应使用 UUID）
        let jti = format!("jti_{}", now);
        
        Self {
            sub: user_id,
            exp,
            iat: now,
            jti,
        }
    }
}

/// 验证认证令牌
/// 
/// 检查令牌的签名和过期时间
/// 
/// # 参数
/// * `token` - JWT 令牌字符串
/// * `secret` - 签名密钥
/// 
/// # 返回
/// 成功时返回 Claims，失败时返回错误信息
pub fn validate_auth_token(token: &str, secret: &str) -> Result<Claims, String> {
    // 检查令牌是否为空
    if token.is_empty() {
        error!("Empty auth token");
        return Err("Empty auth token".to_string());
    }
    
    // 解码并验证令牌签名
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ).map_err(|e| {
        warn!("Token decode failed: {}", e);
        format!("Invalid token signature: {}", e)
    })?;
    
    // 验证过期时间
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    if token_data.claims.exp < now {
        warn!("Token expired at {}, current time: {}", token_data.claims.exp, now);
        return Err("Token expired".to_string());
    }
    
    info!(
        "Token validated for user: {}, expires at: {} (in {} seconds)",
        token_data.claims.sub,
        token_data.claims.exp,
        token_data.claims.exp - now
    );
    
    Ok(token_data.claims)
}

/// 生成新的 JWT 令牌
/// 
/// 用于测试和初始化
/// 
/// # 参数
/// * `user_id` - 用户 ID
/// * `expires_in_hours` - 过期时间（小时）
/// * `secret` - 签名密钥
/// 
/// # 返回
/// JWT 令牌字符串
pub fn generate_token(user_id: String, expires_in_hours: u64, secret: &str) -> Result<String, String> {
    let claims = Claims::new(user_id, expires_in_hours);
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ).map_err(|e| {
        error!("Token generation failed: {}", e);
        format!("Token generation failed: {}", e)
    })
}

/// 以常量时间比较密码，防止计时攻击
///
/// # 参数
/// * `input` - 用户输入的密码
/// * `expected` - 预期密码
///
/// # 返回
/// 密码匹配时返回 true，否则返回 false
pub fn verify_password(input: &str, expected: &str) -> bool {
    if input.is_empty() {
        return false;
    }
    input.as_bytes().ct_eq(expected.as_bytes()).into()
}

/// JWT 认证中间件
///
/// 从 Authorization: Bearer <token> 头中提取并验证令牌
///
/// # 返回
/// 验证通过则调用下一个处理器，否则返回 401 Unauthorized
pub async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let secret = state.config.admin_password.as_deref().unwrap_or("");

    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    match auth_header {
        None => Err(ApiError::Unauthorized("缺少 Authorization header".to_string())),
        Some(token) => {
            validate_auth_token(&token, secret)
                .map_err(|e| ApiError::Unauthorized(e))?;
            Ok(next.run(req).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_secret_key_123";

    /// 测试有效令牌
    #[test]
    fn test_valid_token() {
        let token = generate_token("user_123".to_string(), 1, TEST_SECRET).unwrap();
        let claims = validate_auth_token(&token, TEST_SECRET).unwrap();
        assert_eq!(claims.sub, "user_123");
    }

    /// 测试无效令牌
    #[test]
    fn test_invalid_token() {
        let result = validate_auth_token("invalid_token", TEST_SECRET);
        assert!(result.is_err());
        let err_msg = result.unwrap_err();
        assert!(err_msg.contains("Invalid token signature") || 
                err_msg.contains("Token decode failed"));
    }

    /// 测试错误的密钥
    #[test]
    fn test_wrong_secret() {
        let token = generate_token("user_123".to_string(), 1, TEST_SECRET).unwrap();
        let result = validate_auth_token(&token, "wrong_secret");
        assert!(result.is_err());
    }

    /// 测试过期令牌
    #[test]
    fn test_expired_token() {
        // 创建一个立即过期的令牌（0 小时）
        let claims = Claims::new("user_123".to_string(), 0);
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(TEST_SECRET.as_ref()),
        ).unwrap();
        
        // 等待一秒，确保令牌已过期
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        let result = validate_auth_token(&token, TEST_SECRET);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Token expired"));
    }

    /// 测试令牌声明字段
    #[test]
    fn test_token_claims() {
        let token = generate_token("user_456".to_string(), 24, TEST_SECRET).unwrap();
        let claims = validate_auth_token(&token, TEST_SECRET).unwrap();
        
        assert_eq!(claims.sub, "user_456");
        assert!(claims.exp > claims.iat);
        assert!(!claims.jti.is_empty());
    }

    /// 测试空令牌
    #[test]
    fn test_empty_token() {
        let result = validate_auth_token("", TEST_SECRET);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Empty auth token");
    }

    #[test]
    fn test_verify_password_constant_time() {
        assert!(verify_password("secret", "secret"));
        assert!(!verify_password("secret", "wrong"));
        assert!(!verify_password("", "secret"));
    }

    /// 测试令牌过期时间计算
    #[test]
    fn test_token_expiry_calculation() {
        let token = generate_token("user_789".to_string(), 24, TEST_SECRET).unwrap();
        let claims = validate_auth_token(&token, TEST_SECRET).unwrap();
        
        // 检查过期时间大约是当前时间 + 24 小时
        let expected_expiry_range = 24 * 3600;
        let actual_expiry = claims.exp - claims.iat;
        
        // 允许 1 秒的误差
        assert!((actual_expiry as i64 - expected_expiry_range as i64).abs() <= 1);
    }
}
