//! 隧道穿透工具 - 共享协议定义模块
//! 
//! 本模块定义了客户端和服务器之间的通信协议、错误类型和数据结构

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// ============================================================================
/// 错误类型定义
/// ============================================================================

/// 统一错误码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    // 客户端错误 (4xxx)
    InvalidAuth = 4001,
    TunnelNotFound = 4002,
    SubdomainConflict = 4003,
    LocalPortInvalid = 4004,
    InvalidRequest = 4005,
    
    // 服务端错误 (5xxx)
    DatabaseError = 5001,
    TlsCertificateError = 5002,
    WebSocketError = 5003,
    ProxyError = 5004,
    InternalError = 5005,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}

/// 隧道穿透工具的错误类型
#[derive(Error, Debug)]
pub enum CourierError {
    #[error("认证失败: {0}")]
    InvalidAuth(String),
    
    #[error("隧道不存在: {0}")]
    TunnelNotFound(String),
    
    #[error("子域名冲突: {0}")]
    SubdomainConflict(String),
    
    #[error("无效的本地端口: {0}")]
    InvalidLocalPort(u16),
    
    #[error("数据库错误: {0}")]
    DatabaseError(String),
    
    #[error("TLS证书错误: {0}")]
    TlsCertificateError(String),
    
    #[error("WebSocket错误: {0}")]
    WebSocketError(String),
    
    #[error("代理错误: {0}")]
    ProxyError(String),
    
    #[error("内部错误: {0}")]
    InternalError(String),
    
    #[error("无效的请求: {0}")]
    InvalidRequest(String),
}

impl CourierError {
    /// 获取对应的错误码
    pub fn code(&self) -> ErrorCode {
        match self {
            CourierError::InvalidAuth(_) => ErrorCode::InvalidAuth,
            CourierError::TunnelNotFound(_) => ErrorCode::TunnelNotFound,
            CourierError::SubdomainConflict(_) => ErrorCode::SubdomainConflict,
            CourierError::InvalidLocalPort(_) => ErrorCode::LocalPortInvalid,
            CourierError::DatabaseError(_) => ErrorCode::DatabaseError,
            CourierError::TlsCertificateError(_) => ErrorCode::TlsCertificateError,
            CourierError::WebSocketError(_) => ErrorCode::WebSocketError,
            CourierError::ProxyError(_) => ErrorCode::ProxyError,
            CourierError::InternalError(_) => ErrorCode::InternalError,
            CourierError::InvalidRequest(_) => ErrorCode::InvalidRequest,
        }
    }
}

/// 方便的Result类型别名
pub type Result<T> = std::result::Result<T, CourierError>;

/// ============================================================================
/// 协议消息定义 - JSON 序列化的数据结构
/// ============================================================================

/// 客户端注册请求
/// 
/// 客户端通过此消息向服务器请求创建隧道
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterRequest {
    /// 客户端认证令牌
    pub auth_token: String,
    
    /// 本地服务监听的端口号
    pub local_port: u16,
    
    /// 支持的协议列表 (如 "http", "https")
    pub protocols: Vec<String>,
    
    /// 期望的子域名，空字符串则自动生成
    #[serde(default)]
    pub subdomain: String,
}

/// 隧道建立成功响应
/// 
/// 服务器在客户端成功注册后返回此消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierEstablished {
    /// 隧道的唯一标识符
    pub courier_id: String,
    
    /// 公网访问URL
    pub public_url: String,
    
    /// 服务器域名
    pub server_domain: String,
    
    /// 分配的子域名
    pub subdomain: String,
}

/// 心跳请求消息
/// 
/// 客户端定期发送此消息保持连接活跃
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    /// 隧道ID
    pub courier_id: String,
    
    /// 心跳时间戳 (Unix时间戳，秒)
    pub timestamp: u64,
}

/// 心跳响应消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatResponse {
    /// 响应状态
    pub status: String,
}

/// 隧道注册HTTP API请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRegisterRequest {
    pub auth_token: String,
    pub local_port: u16,
    pub protocols: Vec<String>,
    #[serde(default)]
    pub subdomain: String,
}

/// 隧道注册HTTP API响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRegisterResponse {
    pub courier_id: String,
    pub public_url: String,
    pub server_domain: String,
}

/// 隧道状态查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierStatusResponse {
    pub courier_id: String,
    pub status: String,  // 'active' | 'disconnected'
    pub public_url: String,
    pub connected_at: String,
    pub bytes_transferred: u64,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub active_tunnels: usize,
    pub uptime: u64,
}

/// ============================================================================
/// WebSocket 消息封装
/// ============================================================================

/// WebSocket消息的统一外壳
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    /// 消息类型: "register", "heartbeat", "tunnel_established" 等
    pub msg_type: String,
    
    /// 消息体的JSON数据
    pub data: serde_json::Value,
}

impl WsMessage {
    /// 创建新的WebSocket消息
    pub fn new(msg_type: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            msg_type: msg_type.into(),
            data,
        }
    }
}

/// 前端订阅请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {}

/// 隧道上线事件（服务端 → 前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelConnectedEvent {
    pub courier_id: String,
    pub subdomain: String,
    pub public_url: String,
    pub local_port: u16,
}

/// 隧道下线事件（服务端 → 前端）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelDisconnectedEvent {
    pub courier_id: String,
}

/// 单条隧道流量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelStats {
    pub courier_id: String,
    pub bytes_transferred: u64,
}

/// 流量统计广播事件（服务端 → 前端，每 10 秒）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsUpdateEvent {
    pub tunnels: Vec<TunnelStats>,
}

/// 心跳确认响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatAck {
    pub status: String,
}

/// ============================================================================
/// 常量定义
/// ============================================================================

/// 心跳间隔（秒）
pub const HEARTBEAT_INTERVAL: u64 = 30;

/// 心跳超时时间（秒）
pub const HEARTBEAT_TIMEOUT: u64 = 90;

/// 最大重试次数
pub const MAX_RETRIES: u32 = 10;

/// 初始重试延迟（毫秒）
pub const BASE_RETRY_DELAY_MS: u64 = 1000;

/// 最大重试延迟（毫秒）
pub const MAX_RETRY_DELAY_MS: u64 = 60000;

/// 子域名长度
pub const SUBDOMAIN_LENGTH: usize = 6;

/// 服务器监听的HTTP端口
pub const DEFAULT_SERVER_PORT: u16 = 8080;

/// 服务器监听的HTTPS端口
pub const DEFAULT_HTTPS_PORT: u16 = 443;

/// ============================================================================
/// 辅助函数
/// ============================================================================

/// 生成随机子域名
pub fn generate_subdomain() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..SUBDOMAIN_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 验证子域名格式
pub fn validate_subdomain(subdomain: &str) -> bool {
    // 检查长度
    if subdomain.is_empty() || subdomain.len() > 63 {
        return false;
    }
    
    // 检查字符（只允许字母数字和连字符，但不能以连字符开头或结尾）
    subdomain.chars().all(|c| c.is_alphanumeric() || c == '-')
        && !subdomain.starts_with('-')
        && !subdomain.ends_with('-')
}

/// 验证本地端口号
pub fn validate_local_port(port: u16) -> bool {
    port > 0  // u16 max is 65535, so <= 65535 is always true
}

/// ============================================================================
/// 单元测试
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subdomain_validation() {
        assert!(validate_subdomain("mysubdomain"));
        assert!(validate_subdomain("test-123"));
        assert!(!validate_subdomain("-invalid"));
        assert!(!validate_subdomain("invalid-"));
        assert!(!validate_subdomain(""));
    }
    
    #[test]
    fn test_port_validation() {
        assert!(validate_local_port(8080));
        assert!(validate_local_port(3000));
        assert!(!validate_local_port(0));
        assert!(validate_local_port(65535));
    }
    
    #[test]
    fn test_register_request_serialization() {
        let req = RegisterRequest {
            auth_token: "test_token".to_string(),
            local_port: 3000,
            protocols: vec!["http".to_string()],
            subdomain: String::new(),
        };
        
        let json = serde_json::to_string(&req).unwrap();
        let deserialized: RegisterRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.auth_token, "test_token");
        assert_eq!(deserialized.local_port, 3000);
    }
    
    #[test]
    fn test_tunnel_established_serialization() {
        let resp = CourierEstablished {
            courier_id: "tun_123".to_string(),
            public_url: "https://xyz789.example.com".to_string(),
            server_domain: "example.com".to_string(),
            subdomain: "xyz789".to_string(),
        };
        
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: CourierEstablished = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.courier_id, "tun_123");
        assert_eq!(deserialized.subdomain, "xyz789");
    }
    
    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::InvalidAuth.to_string(), "4001");
        assert_eq!(ErrorCode::DatabaseError.to_string(), "5001");
    }

    #[test]
    fn test_new_message_types_serialize() {
        let evt = TunnelConnectedEvent {
            courier_id: "tun_ABC".to_string(),
            subdomain: "abc".to_string(),
            public_url: "https://abc.example.com".to_string(),
            local_port: 3000,
        };
        let json = serde_json::to_string(&evt).unwrap();
        assert!(json.contains("tun_ABC"));

        let disc = TunnelDisconnectedEvent { courier_id: "tun_ABC".to_string() };
        let json2 = serde_json::to_string(&disc).unwrap();
        assert!(json2.contains("tun_ABC"));

        let stats = StatsUpdateEvent {
            tunnels: vec![TunnelStats { courier_id: "tun_ABC".to_string(), bytes_transferred: 1024 }],
        };
        let json3 = serde_json::to_string(&stats).unwrap();
        assert!(json3.contains("1024"));
    }
}
