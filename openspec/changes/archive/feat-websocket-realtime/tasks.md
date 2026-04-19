## 1. 共享协议扩展（shared/src/lib.rs）

- [x] 1.1 新增 `SubscribeRequest` 结构体（空 data）
- [x] 1.2 新增 `TunnelConnectedEvent` 结构体（courier_id, subdomain, public_url, local_port）
- [x] 1.3 新增 `TunnelDisconnectedEvent` 结构体（courier_id）
- [x] 1.4 新增 `StatsUpdateEvent` 结构体（tunnels: Vec<TunnelStats>，含 courier_id + bytes_transferred）
- [x] 1.5 新增 `HeartbeatAck` 结构体（status: String）

## 2. 服务端 TunnelRegistry（server/src/websocket.rs）

- [x] 2.1 定义 `ClientSession` 结构体（sender, subdomain, local_port, bytes_transferred）
- [x] 2.2 定义 `TunnelRegistry` 结构体（clients: HashMap, subscribers: Vec）
- [x] 2.3 实现 `TunnelRegistry::register_client`：写入 clients，广播 tunnel_connected 给所有 subscribers
- [x] 2.4 实现 `TunnelRegistry::remove_client`：从 clients 移除，广播 tunnel_disconnected，清理失效 subscribers
- [x] 2.5 实现 `TunnelRegistry::add_subscriber`：写入 subscribers，立即推送当前所有 clients 快照
- [x] 2.6 实现 `TunnelRegistry::broadcast_stats`：汇总所有 clients 的 bytes_transferred，广播 stats_update，清理失效 subscribers

## 3. 服务端 AppState 更新（server/src/main.rs）

- [x] 3.1 AppState 增加 `tunnel_registry: Arc<Mutex<TunnelRegistry>>` 字段
- [x] 3.2 启动时初始化 TunnelRegistry 并注入 AppState
- [x] 3.3 启动独立 tokio task，每 10 秒调用 `tunnel_registry.broadcast_stats()`

## 4. 服务端 handle_socket 重写（server/src/main.rs）

- [x] 4.1 读取首条消息，按 msg_type 分支：`register` → client 流程，`subscribe` → subscriber 流程
- [x] 4.2 client 流程：调用 `TunnelRegistry::register_client`，进入 Binary 帧转发循环
- [x] 4.3 Binary 帧转发：收到 Binary 帧后累加 bytes_transferred，将帧内容转发给对应的本地请求响应通道
- [x] 4.4 subscriber 流程：调用 `TunnelRegistry::add_subscriber`，保持连接直到断开
- [x] 4.5 连接关闭时调用 `TunnelRegistry::remove_client`（client 类型）或从 subscribers 移除（subscriber 类型）
- [x] 4.6 处理 heartbeat：回复 heartbeat_ack

## 5. 前端 WebSocket 封装（web/src/api/tunnelApi.ts）

- [x] 5.1 新增 `connectWebSocket(onEvent)` 函数，连接 `ws://localhost:8080/ws`，发送 subscribe 消息
- [x] 5.2 解析收到的 JSON 消息，按 msg_type 分发：tunnel_connected / tunnel_disconnected / stats_update
- [x] 5.3 实现断线重连（指数退避：1s/2s/4s/8s，上限 30s）
- [x] 5.4 导出 `disconnectWebSocket` 函数用于组件卸载时清理

## 6. 前端 App.vue 更新

- [x] 6.1 移除 `setInterval` 轮询逻辑
- [x] 6.2 `onMounted` 时调用 `connectWebSocket`，注册事件回调
- [x] 6.3 收到 `tunnel_connected` 时将隧道追加到 tunnels 列表，更新 activeTunnels 计数
- [x] 6.4 收到 `tunnel_disconnected` 时从 tunnels 列表移除对应项
- [x] 6.5 收到 `stats_update` 时更新对应隧道的 bytes_transferred 和 totalBytes
- [x] 6.6 `onUnmounted` 时调用 `disconnectWebSocket`
