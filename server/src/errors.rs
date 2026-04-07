//! 错误处理模块

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;
use courier_shared::CourierError;

/// 自定义错误类型
#[derive(Debug)]
pub enum ApiError {
    /// 验证错误（400）
    ValidationError(String),
    /// 字段验证错误
    FieldValidationError { field: String, reason: String },
    /// 未找到（404）
    NotFound(String),
    /// 数据库错误（500）
    DatabaseError(String),
    /// 内部服务器错误（500）
    InternalError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::ValidationError(msg) => write!(f, "验证错误：{}", msg),
            ApiError::FieldValidationError { field, reason } => {
                write!(f, "字段 '{}' 验证失败：{}", field, reason)
            }
            ApiError::NotFound(msg) => write!(f, "未找到：{}", msg),
            ApiError::DatabaseError(msg) => write!(f, "数据库错误：{}", msg),
            ApiError::InternalError(msg) => write!(f, "服务器错误：{}", msg),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            ApiError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg.clone(),
                json!({}),
            ),
            ApiError::FieldValidationError { field, reason } => (
                StatusCode::BAD_REQUEST,
                "FIELD_VALIDATION_ERROR",
                format!("字段 '{}' 验证失败", field),
                json!({ "field": field, "reason": reason }),
            ),
            ApiError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg.clone(),
                json!({}),
            ),
            ApiError::DatabaseError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                msg.clone(),
                json!({ "reason": "数据库连接或查询失败" }),
            ),
            ApiError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg.clone(),
                json!({}),
            ),
        };

        let body = json!({
            "code": code,
            "message": message,
            "details": details,
        });

        (status, Json(body)).into_response()
    }
}

// 实现 From trait，方便错误转换
impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        ApiError::InternalError(format!("JSON 序列化错误：{}", err))
    }
}

impl From<String> for ApiError {
    fn from(err: String) -> Self {
        ApiError::InternalError(err)
    }
}

impl From<CourierError> for ApiError {
    fn from(err: CourierError) -> Self {
        match err {
            CourierError::InvalidAuth(msg) => ApiError::ValidationError(msg),
            CourierError::TunnelNotFound(msg) => ApiError::NotFound(msg),
            CourierError::SubdomainConflict(msg) => ApiError::ValidationError(msg),
            CourierError::InvalidLocalPort(port) => {
                ApiError::ValidationError(format!("无效的本地端口: {}", port))
            },
            CourierError::DatabaseError(msg) => ApiError::DatabaseError(msg),
            _ => ApiError::InternalError(err.to_string()),
        }
    }
}
