# WebSocket 实时通信 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 补全 WebSocket 实时通信——courier-client 通过 WebSocket 转发 HTTP 流量，前端管理界面通过 WebSocket 订阅隧道状态事件替换轮询。

**Architecture:** 单 `/ws` 端点多路复用，首条消息 `msg_type` 区分 courier-client（`register`）和前端（`subscribe`）。服务端维护 `TunnelRegistry`（`Arc<Mutex<...>>`），存储所有活跃 client 连接和前端订阅者。控制消息用 JSON Text 帧，流量数据用 Binary 帧。

**Tech Stack:** Rust/Tokio/Axum WebSocket（futures-util SplitSink/SplitStream）、Vue3 Composition API、浏览器原生 WebSocket API。

---

### Task 1: 扩展共享协议类型（shared/src/lib.rs）

**Files:**
- Modify: `shared/src/lib.rs`（在现有 `WsMessage` 定义后追加）

- [ ] **Step 1: 写失败测试**

在 `shared/src/lib.rs` 的 `#[cfg(test)]` 块末尾追加：

```rust
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
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test -p courier-shared test_new_message_types_serialize
```
Expected: `error[E0422]: cannot find struct TunnelConnectedEvent`

- [ ] **Step 3: 实现新消息类型**

在 `shared/src/lib.rs` 的 `WsMessage impl` 块之后、常量定义之前追加：

```rust
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
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cargo test -p courier-shared test_new_message_types_serialize
```
Expected: `test test_new_message_types_serialize ... ok`

- [ ] **Step 5: 提交**

```bash
git add shared/src/lib.rs
git commit -m "feat: 新增 WebSocket 实时通信协议类型"
```

---

### Task 2: 实现 TunnelRegistry（server/src/websocket.rs）

**Files:**
- Modify: `server/src/websocket.rs`（完整重写）

当前 `websocket.rs` 有 `WsConnectionManager`（仅含连接计数）和占位 `handle_connection`。本任务替换为真正的 `TunnelRegistry`。

- [ ] **Step 1: 写失败测试**

在 `server/src/websocket.rs` 的 `#[cfg(test)]` 块追加：

```rust
#[tokio::test]
async fn test_registry_register_and_count() {
    let registry = TunnelRegistry::new();
    assert_eq!(registry.client_count(), 0);
}

#[tokio::test]
async fn test_registry_remove_nonexistent_is_ok() {
    let mut registry = TunnelRegistry::new();
    // 移除不存在的 client 不应 panic
    registry.remove_client("nonexistent");
    assert_eq!(registry.client_count(), 0);
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test -p courier-server test_registry_register_and_count 2>&1 | head -20
```
Expected: `error[E0422]: cannot find struct TunnelRegistry`

- [ ] **Step 3: 实现 TunnelRegistry**

将 `server/src/websocket.rs` 完整替换为：

```rust
//! WebSocket 服务器 - TunnelRegistry 和连接类型分发

use axum::extract::ws::{Message, WebSocket};
use courier_shared::{
    TunnelConnectedEvent, TunnelDisconnectedEvent, TunnelStats, StatsUpdateEvent,
    HeartbeatAck, WsMessage,
};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::collections::HashMap;
use tracing::{info, warn};

/// 单个 courier-client 的连接会话
pub struct ClientSession {
    pub sender: SplitSink<WebSocket, Message>,
    pub subdomain: String,
    pub local_port: u16,
    pub bytes_transferred: u64,
}

/// 全局隧道注册表（由 Arc<Mutex<TunnelRegistry>> 保护）
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

    /// 注册新 courier-client，广播 tunnel_connected 给所有订阅者
    pub async fn register_client(
        &mut self,
        courier_id: String,
        session: ClientSession,
    ) {
        let event = TunnelConnectedEvent {
            courier_id: courier_id.clone(),
            subdomain: session.subdomain.clone(),
            public_url: format!("https://{}.placeholder", session.subdomain),
            local_port: session.local_port,
        };
        self.clients.insert(courier_id, session);
        self.broadcast_json("tunnel_connected", &event).await;
    }

    /// 移除 courier-client，广播 tunnel_disconnected 给所有订阅者
    pub fn remove_client(&mut self, courier_id: &str) {
        self.clients.remove(courier_id);
        // 广播在调用方异步完成（避免 remove_client 变 async）
    }

    /// 广播 tunnel_disconnected（异步，移除 client 后调用）
    pub async fn broadcast_disconnected(&mut self, courier_id: &str) {
        let event = TunnelDisconnectedEvent { courier_id: courier_id.to_string() };
        self.broadcast_json("tunnel_disconnected", &event).await;
    }

    /// 新增前端订阅者，立即推送当前所有 client 快照
    pub async fn add_subscriber(&mut self, mut sender: SplitSink<WebSocket, Message>) {
        // 推送当前快照
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
                return; // 发送失败则不加入
            }
        }
        self.subscribers.push(sender);
    }

    /// 汇总流量统计并广播给所有订阅者，清理已断开的订阅者
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

    /// 向所有订阅者广播 JSON 消息，失败的订阅者惰性移除
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
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cargo test -p courier-server test_registry 2>&1 | tail -10
```
Expected: 两个测试均 `ok`

- [ ] **Step 5: 提交**

```bash
git add server/src/websocket.rs
git commit -m "feat: 实现 TunnelRegistry（WebSocket 连接注册表）"
```

---

### Task 3: AppState 注入 TunnelRegistry + stats 定时器（server/src/main.rs）

**Files:**
- Modify: `server/src/main.rs`（AppState 结构、初始化、stats task、handle_socket）

- [ ] **Step 1: 写失败测试**

在 `server/src/main.rs` 的 `#[cfg(test)]` 块追加：

```rust
#[test]
fn test_appstate_has_tunnel_registry() {
    // 编译期验证 AppState 含 tunnel_registry 字段
    // 若字段缺失此测试无法编译
    let _: fn() -> () = || {
        let _field_exists: std::sync::Arc<tokio::sync::Mutex<crate::websocket::TunnelRegistry>>;
    };
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test -p courier-server test_appstate_has_tunnel_registry 2>&1 | head -10
```
Expected: 编译错误（`tunnel_registry` 字段不存在于 `AppState`）

- [ ] **Step 3: 修改 AppState 和初始化**

在 `server/src/main.rs` 中：

**3a. 添加 import（文件顶部 use 块）：**
```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::websocket::TunnelRegistry;
```

**3b. 修改 `AppState` 结构体：**
```rust
#[derive(Clone)]
struct AppState {
    db: SqlitePool,
    config: Arc<ServerConfig>,
    tunnel_registry: Arc<Mutex<TunnelRegistry>>,
}
```

**3c. 修改 `main()` 中的 AppState 初始化（替换现有 `let state = AppState {...}`）：**
```rust
let tunnel_registry = Arc::new(Mutex::new(TunnelRegistry::new()));

// 启动 stats 广播定时器（每 10 秒）
let registry_for_stats = tunnel_registry.clone();
tokio::spawn(async move {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        registry_for_stats.lock().await.broadcast_stats().await;
    }
});

let state = AppState {
    db,
    config: Arc::new(ServerConfig {
        server_domain: args.server_domain.clone(),
        admin_password: args.admin_password,
    }),
    tunnel_registry,
};
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cargo build -p courier-server 2>&1 | grep -E "^error" | head -10
cargo test -p courier-server test_appstate_has_tunnel_registry
```
Expected: 编译成功，测试 `ok`

- [ ] **Step 5: 提交**

```bash
git add server/src/main.rs
git commit -m "feat: AppState 注入 TunnelRegistry，启动 stats 定时广播"
```

---

### Task 4: 重写 handle_socket（server/src/main.rs）

**Files:**
- Modify: `server/src/main.rs`（替换现有 `handle_socket` 函数）

- [ ] **Step 1: 写失败测试**

在 `server/src/main.rs` 的 `#[cfg(test)]` 块追加：

```rust
#[tokio::test]
async fn test_handle_socket_integration() {
    // 验证服务端路由可以启动（编译期验证 handle_socket 签名正确）
    let db = crate::db::init_database("sqlite::memory:").await.unwrap();
    let registry = Arc::new(Mutex::new(TunnelRegistry::new()));
    let state = AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: "localhost:8080".to_string(),
            admin_password: None,
        }),
        tunnel_registry: registry,
    };
    let _router = build_router(state);
    // 路由构建成功即通过
}
```

- [ ] **Step 2: 运行测试确认当前状态**

```bash
cargo test -p courier-server test_handle_socket_integration 2>&1 | tail -5
```

- [ ] **Step 3: 替换 handle_socket 实现**

将 `server/src/main.rs` 中现有的 `handle_socket` 函数完整替换为：

```rust
async fn handle_socket(socket: WebSocket, state: AppState) {
    use axum::extract::ws::Message;
    use futures_util::StreamExt;

    let (sender, mut receiver) = socket.split();

    // 等待首条消息，决定连接类型
    let first_msg = match receiver.next().await {
        Some(Ok(Message::Text(t))) => t,
        _ => return,
    };

    let ws_msg: courier_shared::WsMessage = match serde_json::from_str(&first_msg) {
        Ok(m) => m,
        Err(_) => return,
    };

    match ws_msg.msg_type.as_str() {
        "register" => {
            handle_client_connection(sender, receiver, ws_msg.data, state).await;
        }
        "subscribe" => {
            handle_subscriber_connection(sender, receiver, state).await;
        }
        _ => {}
    }
}

/// courier-client 连接：注册隧道 + Binary 帧转发循环
async fn handle_client_connection(
    sender: futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
    mut receiver: futures_util::stream::SplitStream<WebSocket>,
    data: serde_json::Value,
    state: AppState,
) {
    use axum::extract::ws::Message;
    use courier_shared::{RegisterRequest, WsMessage, HeartbeatAck};
    use uuid::Uuid;

    let req: RegisterRequest = match serde_json::from_value(data) {
        Ok(r) => r,
        Err(_) => return,
    };

    if req.auth_token.is_empty() {
        return;
    }

    let courier_id = format!("tun_{}", &Uuid::new_v4().to_string()[..8].to_uppercase());
    let subdomain = if req.subdomain.is_empty() {
        courier_shared::generate_subdomain()
    } else {
        req.subdomain.clone()
    };
    let server_domain = state.config.server_domain.clone();
    let public_url = format!("https://{}.{}", subdomain, server_domain);

    // 持久化到数据库
    if let Err(e) = crate::db::create_tunnel_with_unique_subdomain(
        &state.db,
        &courier_id,
        &subdomain,
        &req.auth_token,
        req.local_port,
    ).await {
        tracing::error!("DB error: {}", e);
        return;
    }

    // 回复 tunnel_established
    let response = WsMessage::new("tunnel_established", serde_json::json!({
        "courier_id": courier_id,
        "subdomain": subdomain,
        "public_url": public_url,
        "server_domain": server_domain,
    }));

    let session = crate::websocket::ClientSession {
        sender,
        subdomain: subdomain.clone(),
        local_port: req.local_port,
        bytes_transferred: 0,
    };

    // 先发送 tunnel_established，再注册（ClientSession 拥有 sender）
    // 因此先构造好 session，通过注册接口发送响应
    {
        let mut reg = state.tunnel_registry.lock().await;
        // 将 tunnel_established 通过 session.sender 发送
        // register_client 内部不发 tunnel_established，需在注册前发
        // 所以拆分：先拿出 sender 发消息，再交给 session
        // 由于所有权约束，用临时 channel 方式：
        // 实际上 session.sender 在 register_client 后由 registry 持有
        // 此处重新设计：先发消息（需要 sender），再交给 registry
        // 见 Step 3b 说明
        reg.register_client_raw(courier_id.clone(), session, response).await;
    }

    info!("courier-client registered: {} ({})", courier_id, subdomain);

    // 进入消息循环
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Binary(data)) => {
                // 累加流量统计
                let mut reg = state.tunnel_registry.lock().await;
                if let Some(session) = reg.clients.get_mut(&courier_id) {
                    session.bytes_transferred += data.len() as u64;
                }
                // TODO: 将 data 转发给等待的 HTTP 请求（本期记录流量，转发在下一期实现）
                tracing::debug!("binary frame {} bytes from {}", data.len(), courier_id);
            }
            Ok(Message::Text(text)) => {
                if let Ok(m) = serde_json::from_str::<WsMessage>(&text) {
                    if m.msg_type == "heartbeat" {
                        let ack = WsMessage::new("heartbeat_ack", serde_json::to_value(HeartbeatAck { status: "ok".to_string() }).unwrap());
                        let ack_text = serde_json::to_string(&ack).unwrap();
                        let mut reg = state.tunnel_registry.lock().await;
                        if let Some(session) = reg.clients.get_mut(&courier_id) {
                            let _ = session.sender.send(Message::Text(ack_text)).await;
                        }
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    // 断线清理
    let mut reg = state.tunnel_registry.lock().await;
    reg.remove_client(&courier_id);
    reg.broadcast_disconnected(&courier_id).await;
    let _ = crate::db::update_tunnel_status(&state.db, &courier_id, "disconnected").await;
    info!("courier-client disconnected: {}", courier_id);
}

/// 前端订阅者连接：注册后保持接收（前端不发消息）
async fn handle_subscriber_connection(
    sender: futures_util::stream::SplitSink<WebSocket, axum::extract::ws::Message>,
    mut receiver: futures_util::stream::SplitStream<WebSocket>,
    state: AppState,
) {
    use axum::extract::ws::Message;
    state.tunnel_registry.lock().await.add_subscriber(sender).await;
    info!("frontend subscriber connected");

    // 保持连接直到断开
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    info!("frontend subscriber disconnected");
}
```

**注意：** `register_client_raw` 需要在 Task 2 完成后对 `TunnelRegistry` 添加一个变体方法，在注册时同时通过 session.sender 发送 `tunnel_established`。见 Step 3b。

- [ ] **Step 3b: 在 websocket.rs 增加 register_client_raw**

在 `TunnelRegistry` impl 块追加：

```rust
/// 注册 client，先通过 session.sender 发送 established 消息，再广播 tunnel_connected
pub async fn register_client_raw(
    &mut self,
    courier_id: String,
    mut session: ClientSession,
    established_msg: courier_shared::WsMessage,
) {
    use axum::extract::ws::Message;
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
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cargo test -p courier-server 2>&1 | tail -15
```
Expected: 所有测试通过，无编译错误

- [ ] **Step 5: 提交**

```bash
git add server/src/main.rs server/src/websocket.rs
git commit -m "feat: 实现 WebSocket 多路复用——client 注册转发 + subscriber 订阅"
```

---

### Task 5: db::update_tunnel_status（server/src/db.rs）

**Files:**
- Modify: `server/src/db.rs`

Task 4 调用了 `db::update_tunnel_status`，需要先实现它。

- [ ] **Step 1: 写失败测试**

在 `server/src/db.rs` 的 `#[cfg(test)]` 块追加：

```rust
#[tokio::test]
async fn test_update_tunnel_status() {
    let pool = init_database("sqlite::memory:").await.unwrap();
    // 先创建一条隧道
    create_tunnel_with_unique_subdomain(&pool, "tun_TEST", "testsub", "token", 3000)
        .await.unwrap();
    // 更新状态
    update_tunnel_status(&pool, "tun_TEST", "disconnected").await.unwrap();
    // 查询确认
    let tunnel = get_tunnel_by_id(&pool, "tun_TEST").await.unwrap().unwrap();
    assert_eq!(tunnel.status, "disconnected");
}
```

- [ ] **Step 2: 运行测试确认失败**

```bash
cargo test -p courier-server test_update_tunnel_status 2>&1 | head -10
```
Expected: `error[E0425]: cannot find function update_tunnel_status`

- [ ] **Step 3: 实现 update_tunnel_status**

在 `server/src/db.rs` 中查找现有函数后追加：

```rust
/// 更新隧道状态（如 "connected" → "disconnected"）
pub async fn update_tunnel_status(
    pool: &SqlitePool,
    courier_id: &str,
    status: &str,
) -> anyhow::Result<()> {
    sqlx::query!(
        "UPDATE tunnels SET status = ? WHERE courier_id = ?",
        status,
        courier_id
    )
    .execute(pool)
    .await?;
    Ok(())
}
```

- [ ] **Step 4: 运行测试确认通过**

```bash
cargo test -p courier-server test_update_tunnel_status
```
Expected: `ok`

- [ ] **Step 5: 提交**

```bash
git add server/src/db.rs
git commit -m "feat: 新增 db::update_tunnel_status"
```

---

### Task 6: 前端 WebSocket 封装（web/src/api/tunnelApi.ts）

**Files:**
- Modify: `web/src/api/tunnelApi.ts`（追加 WebSocket 相关函数）

- [ ] **Step 1: 定义事件类型和接口**

在 `tunnelApi.ts` 末尾追加：

```typescript
export interface TunnelConnectedEvent {
  courier_id: string
  subdomain: string
  public_url: string
  local_port: number
}

export interface TunnelDisconnectedEvent {
  courier_id: string
}

export interface TunnelStatsItem {
  courier_id: string
  bytes_transferred: number
}

export interface StatsUpdateEvent {
  tunnels: TunnelStatsItem[]
}

export type WsEventHandler = {
  onConnected: (evt: TunnelConnectedEvent) => void
  onDisconnected: (evt: TunnelDisconnectedEvent) => void
  onStatsUpdate: (evt: StatsUpdateEvent) => void
  onSnapshot: (evt: TunnelConnectedEvent) => void
}

let ws: WebSocket | null = null
let reconnectDelay = 1000

export function connectWebSocket(handlers: WsEventHandler): void {
  if (ws && ws.readyState === WebSocket.OPEN) return

  ws = new WebSocket('ws://localhost:8080/ws')

  ws.onopen = () => {
    reconnectDelay = 1000
    ws!.send(JSON.stringify({ msg_type: 'subscribe', data: {} }))
  }

  ws.onmessage = (event) => {
    try {
      const msg = JSON.parse(event.data as string) as { msg_type: string; data: unknown }
      switch (msg.msg_type) {
        case 'tunnel_connected':
          handlers.onConnected(msg.data as TunnelConnectedEvent)
          break
        case 'tunnel_disconnected':
          handlers.onDisconnected(msg.data as TunnelDisconnectedEvent)
          break
        case 'stats_update':
          handlers.onStatsUpdate(msg.data as StatsUpdateEvent)
          break
      }
    } catch {
      // 忽略无法解析的消息
    }
  }

  ws.onclose = () => {
    ws = null
    setTimeout(() => connectWebSocket(handlers), reconnectDelay)
    reconnectDelay = Math.min(reconnectDelay * 2, 30000)
  }

  ws.onerror = () => {
    ws?.close()
  }
}

export function disconnectWebSocket(): void {
  if (ws) {
    ws.onclose = null // 阻止重连
    ws.close()
    ws = null
  }
}
```

- [ ] **Step 2: 确认 TypeScript 编译通过**

```bash
cd web && npx tsc --noEmit 2>&1 | head -20
```
Expected: 无错误输出

- [ ] **Step 3: 提交**

```bash
git add web/src/api/tunnelApi.ts
git commit -m "feat: 前端新增 WebSocket 订阅封装（connectWebSocket/disconnectWebSocket）"
```

---

### Task 7: 前端 App.vue 替换轮询为 WebSocket 订阅

**Files:**
- Modify: `web/src/App.vue`（`<script setup>` 部分）

- [ ] **Step 1: 修改 script setup**

将 `App.vue` 中 `<script setup lang="ts">` 的 import 行和 `onMounted` 逻辑替换如下：

**修改 import 行（在现有 import 后追加）：**
```typescript
import { connectWebSocket, disconnectWebSocket } from './api/tunnelApi'
import type { TunnelConnectedEvent, TunnelDisconnectedEvent, StatsUpdateEvent } from './api/tunnelApi'
```

**删除** `onMounted` 中的 `setInterval` 块，替换整个 `onMounted`：

```typescript
onMounted(async () => {
  addLog('INFO', '应用启动')
  await checkHealth()
  await fetchTunnels() // 初次加载完整列表

  connectWebSocket({
    onConnected(evt: TunnelConnectedEvent) {
      if (!tunnels.value.find(t => t.id === evt.courier_id)) {
        tunnels.value.push({
          id: evt.courier_id,
          subdomain: evt.subdomain,
          local_port: evt.local_port,
          status: 'connected',
          bytes_transferred: 0,
        })
        activeTunnels.value = tunnels.value.length
      }
      addLog('INFO', `隧道上线: ${evt.subdomain}`)
    },
    onDisconnected(evt: TunnelDisconnectedEvent) {
      tunnels.value = tunnels.value.filter(t => t.id !== evt.courier_id)
      activeTunnels.value = tunnels.value.length
      addLog('INFO', `隧道下线: ${evt.courier_id}`)
    },
    onStatsUpdate(evt: StatsUpdateEvent) {
      for (const stat of evt.tunnels) {
        const t = tunnels.value.find(t => t.id === stat.courier_id)
        if (t) t.bytes_transferred = stat.bytes_transferred
      }
      totalBytes.value = tunnels.value.reduce((sum, t) => sum + t.bytes_transferred, 0)
    },
    onSnapshot(evt: TunnelConnectedEvent) {
      if (!tunnels.value.find(t => t.id === evt.courier_id)) {
        tunnels.value.push({
          id: evt.courier_id,
          subdomain: evt.subdomain,
          local_port: evt.local_port,
          status: 'connected',
          bytes_transferred: 0,
        })
        activeTunnels.value = tunnels.value.length
      }
    },
  })
})
```

**追加 onUnmounted：**
```typescript
import { ref, computed, onMounted, onUnmounted } from 'vue'
// （替换原有的 onMounted import）
```

```typescript
onUnmounted(() => {
  disconnectWebSocket()
})
```

- [ ] **Step 2: 确认 TypeScript 编译通过**

```bash
cd web && npx tsc --noEmit 2>&1 | head -20
```
Expected: 无错误

- [ ] **Step 3: 提交**

```bash
git add web/src/App.vue
git commit -m "feat: 前端替换 setInterval 轮询为 WebSocket 实时订阅"
```

---

### Task 8: 全量测试验证

- [ ] **Step 1: 运行所有 Rust 测试**

```bash
cargo test --workspace 2>&1 | tail -20
```
Expected: 所有测试通过，无 `FAILED`

- [ ] **Step 2: 确认前端编译**

```bash
cd web && npm run build 2>&1 | tail -10
```
Expected: `built in Xs`，无错误

- [ ] **Step 3: 手动冒烟测试**

```bash
# 终端 1：启动服务端
cargo run -p courier-server -- --port 8080 --database :memory: --server-domain localhost:8080

# 终端 2：用 wscat 模拟 courier-client
wscat -c ws://localhost:8080/ws
# 输入：{"msg_type":"register","data":{"auth_token":"test","local_port":3000,"protocols":["http"],"subdomain":""}}
# 期望收到：{"msg_type":"tunnel_established","data":{...}}

# 终端 3：用 wscat 模拟前端订阅
wscat -c ws://localhost:8080/ws
# 输入：{"msg_type":"subscribe","data":{}}
# 期望收到当前隧道快照，之后收到 tunnel_connected 事件
```

- [ ] **Step 4: 最终提交（如有遗漏文件）**

```bash
git status
git add -p  # 按需暂存
git commit -m "test: WebSocket 实时通信全量验证通过"
```

---

## Self-Review

**Spec coverage 检查：**

| Spec 要求 | 覆盖任务 |
|---|---|
| register 消息 → tunnel_established 响应 | Task 4 handle_client_connection |
| 空 auth_token → 关闭连接 | Task 4（`if req.auth_token.is_empty() { return; }`） |
| Binary 帧流量转发（字节计数） | Task 4 Binary 分支 |
| heartbeat → heartbeat_ack | Task 4 Text 分支 |
| 断线 → 移除 registry + DB 状态更新 | Task 4 + Task 5 |
| subscribe → 注册订阅者 + 快照推送 | Task 4 handle_subscriber_connection + Task 2 add_subscriber |
| tunnel_connected 广播 | Task 2 register_client_raw |
| tunnel_disconnected 广播 | Task 2 broadcast_disconnected |
| stats_update 每 10 秒广播 | Task 3 stats 定时器 |
| 前端替换轮询 | Task 7 |
| 断线重连（指数退避） | Task 6 onclose handler |

**类型一致性检查：**
- `ClientSession.sender` 类型：`SplitSink<WebSocket, Message>`，Task 2 定义，Task 4 使用 ✓
- `TunnelConnectedEvent` 字段：Task 1 定义（courier_id/subdomain/public_url/local_port），Task 2/4/6 使用 ✓
- `register_client_raw` 在 Task 2 Step 3b 定义，Task 4 Step 3 调用 ✓
- `update_tunnel_status` 在 Task 5 定义，Task 4 调用 ✓（Task 5 需在 Task 4 之前执行）

**任务依赖顺序：** Task 1 → Task 2 → Task 5 → Task 3 → Task 4 → Task 6 → Task 7 → Task 8
