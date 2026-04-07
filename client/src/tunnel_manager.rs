//! 隧道管理器 - 核心业务逻辑

use crate::config::ClientConfig;
use anyhow::Result;
use std::time::Duration;
use tokio_tungstenite::tungstenite::Message;
use futures::stream::{StreamExt, SplitSink};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use courier_shared::{
    MAX_RETRIES, BASE_RETRY_DELAY_MS, MAX_RETRY_DELAY_MS, RegisterRequest,
};

/// 隧道管理器
/// 
/// 负责管理与服务器的连接、流量转发和生命周期管理
pub struct TunnelManager {
    config: ClientConfig,
    retry_count: u32,
}

impl TunnelManager {
    /// 创建新的隧道管理器
    pub async fn new(config: ClientConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config,
            retry_count: 0,
        })
    }

    /// 启动隧道（主运行循环）
    pub async fn run(&self) -> Result<()> {
        info!("隧道管理器启动");

        loop {
            match self.run_once().await {
                Ok(_) => {
                    // 成功运行完成（通常不会返回）
                    info!("隧道运行完成");
                }
                Err(e) => {
                    error!("隧道运行出错: {}", e);
                    self.handle_error().await?;
                }
            }
        }
    }

    /// 单次运行周期（Task 6 - WebSocket 通信循环）
    async fn run_once(&self) -> Result<()> {
        info!("🔄 WebSocket 通信循环启动");
        
        // 1. 连接到服务器
        info!("📡 正在连接到服务器: {}", self.config.server_address);
        let ws_stream = self.connect_to_server().await?;
        let (mut sender, mut receiver) = ws_stream.split();
        let sender: Arc<Mutex<_>> = Arc::new(Mutex::new(sender));
        info!("✅ WebSocket 连接成功");

        // 2. 发送注册请求
        let register_req = RegisterRequest {
            auth_token: self.config.auth_token.clone(),
            local_port: self.config.local_port,
            protocols: self.config.protocols.clone(),
            subdomain: self.config.subdomain.clone(),
        };

        info!("📝 发送注册请求");
        let msg = courier_shared::WsMessage::new(
            "register",
            serde_json::to_value(&register_req)?
        );
        {
            let mut s = sender.lock().await;
            s.send(Message::Text(serde_json::to_string(&msg)?)).await?;
        }
        info!("✅ 注册请求已发送");

        // 3. 等待隧道建立响应
        info!("⏳ 等待隧道建立响应");
        let tunnel_established = loop {
            match receiver.next().await {
                Some(Ok(Message::Text(text))) => {
                    let ws_msg: courier_shared::WsMessage = serde_json::from_str(&text)?;
                    
                    if ws_msg.msg_type == "tunnel_established" {
                        let tunnel: courier_shared::CourierEstablished = 
                            serde_json::from_value(ws_msg.data)?;
                        info!(
                            "✅ 隧道建立成功: {} - {}",
                            tunnel.courier_id, tunnel.public_url
                        );
                        break tunnel;
                    } else {
                        warn!("收到未预期的消息类型: {}", ws_msg.msg_type);
                    }
                },
                Some(Ok(_)) => {
                    warn!("收到非文本消息");
                },
                Some(Err(e)) => {
                    error!("WebSocket 错误: {}", e);
                    return Err(anyhow::anyhow!("WebSocket error: {}", e));
                },
                None => {
                    return Err(anyhow::anyhow!("服务器关闭了连接"));
                }
            }
        };

        // 4. 启动心跳任务
        info!("💓 启动心跳任务");
        let courier_id = tunnel_established.courier_id.clone();
        let sender_hb = Arc::clone(&sender);
        
        // 启动心跳任务（异步后台任务）
        let heartbeat_handle = tokio::spawn(async move {
            const HEARTBEAT_INTERVAL: u64 = 30;
            loop {
                tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
                let hb = courier_shared::HeartbeatRequest {
                    courier_id: courier_id.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                let msg = courier_shared::WsMessage::new(
                    "heartbeat",
                    serde_json::json!(hb)
                );
                if let Ok(mut lock) = sender_hb.lock().await.send(
                    Message::Text(serde_json::to_string(&msg).unwrap_or_default())
                ).await {
                    info!("💓 心跳已发送: {}", courier_id);
                }
            }
        });

        // 5. 启动流量转发
        info!("🔀 启动流量转发 (本地端口 {})", self.config.local_port);
        let local_proxy = crate::proxy::LocalProxy::new(
            self.config.local_port,
            self.config.server_address.clone(),
        );
        
        // 代理会一直运行，直到隧道关闭
        match local_proxy.start().await {
            Ok(_) => {
                info!("✅ 流量转发正常完成");
            },
            Err(e) => {
                error!("❌ 流量转发错误: {}", e);
                heartbeat_handle.abort();
                return Err(e);
            }
        }

        // 代理关闭时，中止心跳
        heartbeat_handle.abort();
        info!("⏹️  WebSocket 通信循环结束");

        Ok(())
    }

    /// 启动心跳任务（Task 6）
    async fn start_heartbeat(
        &self,
        courier_id: String,
        sender: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    ) -> Result<()> {
        const HEARTBEAT_INTERVAL: u64 = 30;  // 30 秒心跳
        
        info!("💓 心跳任务启动: {}", courier_id);
        
        loop {
            tokio::time::sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
            
            let hb = courier_shared::HeartbeatRequest {
                courier_id: courier_id.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            
            let msg = courier_shared::WsMessage::new(
                "heartbeat",
                serde_json::to_value(&hb)?
            );
            
            match sender.lock().await.send(
                Message::Text(serde_json::to_string(&msg)?)
            ).await {
                Ok(_) => {
                    info!("💓 心跳已发送: {}", courier_id);
                },
                Err(e) => {
                    error!("❌ 心跳发送失败: {}", e);
                    return Err(anyhow::anyhow!("Heartbeat send failed: {}", e));
                }
            }
        }
    }

    /// 连接到WebSocket服务器
    async fn connect_to_server(&self) -> Result<tokio_tungstenite::WebSocketStream<
        tokio::net::TcpStream,
    >> {
        // 将ws://改为localhost的标准地址
        let addr = self.config.server_address.replace("ws://", "").replace("wss://", "");
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .map_err(|e| anyhow::anyhow!("TCP连接失败: {}", e))?;
        
        let ws_stream = tokio_tungstenite::client_async(&self.config.server_address, stream)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket连接失败: {}", e))?
            .0;

        Ok(ws_stream)
    }

    /// 处理错误，执行断线重连
    async fn handle_error(&self) -> Result<()> {
        if self.retry_count >= MAX_RETRIES {
            anyhow::bail!("达到最大重试次数，放弃连接");
        }

        // 计算指数退避延迟时间
        let delay_ms = Self::calculate_backoff_delay(self.retry_count);
        warn!(
            "将在 {}ms 后重试 (重试次数: {}/{})",
            delay_ms, self.retry_count, MAX_RETRIES
        );

        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        Ok(())
    }

    /// 计算指数退避延迟时间
    fn calculate_backoff_delay(attempt: u32) -> u64 {
        let base = BASE_RETRY_DELAY_MS as u64;
        let backoff = 2_u64.saturating_pow(attempt);
        let delay = base.saturating_mul(backoff);
        delay.min(MAX_RETRY_DELAY_MS as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        assert_eq!(TunnelManager::calculate_backoff_delay(0), 1000);
        assert_eq!(TunnelManager::calculate_backoff_delay(1), 2000);
        assert_eq!(TunnelManager::calculate_backoff_delay(2), 4000);
        assert_eq!(TunnelManager::calculate_backoff_delay(3), 8000);
        assert_eq!(TunnelManager::calculate_backoff_delay(4), 16000);
        assert_eq!(TunnelManager::calculate_backoff_delay(5), 32000);
        assert_eq!(TunnelManager::calculate_backoff_delay(6), 60000); // 超过最大值60000
    }

    #[tokio::test]
    async fn test_courier_manager_creation() {
        let config = ClientConfig {
            local_port: 3000,
            server_address: "ws://localhost:8080".to_string(),
            auth_token: "test_token".to_string(),
            subdomain: String::new(),
            protocols: vec!["http".to_string()],
        };

        let manager = TunnelManager::new(config).await;
        assert!(manager.is_ok());
    }
}
