//! WebSocket 服务器 - TunnelRegistry 和连接类型分发

use axum::extract::ws::{Message, WebSocket};
use courier_shared::{
    TunnelConnectedEvent, TunnelDisconnectedEvent, TunnelStats, StatsUpdateEvent,
    WsMessage,
};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::collections::HashMap;
use tracing::warn;

pub struct ClientSession {
    pub sender: SplitSink<WebSocket, Message>,
    pub subdomain: String,
    pub local_port: u16,
    pub bytes_transferred: u64,
}

pub struct TunnelRegistry {
    pub clients: HashMap<String, ClientSession>,
    pub subscribers: Vec<SplitSink<WebSocket, Message>>,
}

impl TunnelRegistry {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            subscribers: Vec::new(),
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    pub fn remove_client(&mut self, courier_id: &str) {
        self.clients.remove(courier_id);
    }

    pub async fn broadcast_disconnected(&mut self, courier_id: &str) {
        let event = TunnelDisconnectedEvent { courier_id: courier_id.to_string() };
        self.broadcast_json("tunnel_disconnected", &event).await;
    }

    pub async fn add_subscriber(&mut self, mut sender: SplitSink<WebSocket, Message>) {
        let snapshot: Vec<TunnelConnectedEvent> = self.clients.iter().map(|(id, s)| {
            TunnelConnectedEvent {
                courier_id: id.clone(),
                subdomain: s.subdomain.clone(),
                public_url: format!("https://{}.placeholder", s.subdomain),
                local_port: s.local_port,
            }
        }).collect();

        for evt in snapshot {
            let msg = WsMessage::new("tunnel_connected", serde_json::to_value(&evt).unwrap());
            let text = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(text)).await.is_err() {
                return;
            }
        }
        self.subscribers.push(sender);
    }

    pub async fn broadcast_stats(&mut self) {
        let tunnels: Vec<TunnelStats> = self.clients.iter().map(|(id, s)| {
            TunnelStats {
                courier_id: id.clone(),
                bytes_transferred: s.bytes_transferred,
            }
        }).collect();
        let event = StatsUpdateEvent { tunnels };
        self.broadcast_json("stats_update", &event).await;
    }

    /// 注册 client，先通过 session.sender 发送 established 消息，再广播 tunnel_connected
    pub async fn register_client_raw(
        &mut self,
        courier_id: String,
        mut session: ClientSession,
        established_msg: courier_shared::WsMessage,
    ) {
        let text = serde_json::to_string(&established_msg).unwrap();
        let _ = session.sender.send(Message::Text(text)).await;

        let event = TunnelConnectedEvent {
            courier_id: courier_id.clone(),
            subdomain: session.subdomain.clone(),
            public_url: format!("https://{}.placeholder", session.subdomain),
            local_port: session.local_port,
        };
        self.clients.insert(courier_id, session);
        self.broadcast_json("tunnel_connected", &event).await;
    }

    async fn broadcast_json<T: serde::Serialize>(&mut self, msg_type: &str, data: &T) {
        let msg = WsMessage::new(msg_type, serde_json::to_value(data).unwrap());
        let text = serde_json::to_string(&msg).unwrap();

        let mut failed = vec![];
        for (i, sub) in self.subscribers.iter_mut().enumerate() {
            if sub.send(Message::Text(text.clone())).await.is_err() {
                warn!("subscriber {} disconnected, will remove", i);
                failed.push(i);
            }
        }
        for i in failed.into_iter().rev() {
            self.subscribers.swap_remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_register_and_count() {
        let registry = TunnelRegistry::new();
        assert_eq!(registry.client_count(), 0);
    }

    #[tokio::test]
    async fn test_registry_remove_nonexistent_is_ok() {
        let mut registry = TunnelRegistry::new();
        registry.remove_client("nonexistent");
        assert_eq!(registry.client_count(), 0);
    }
}
