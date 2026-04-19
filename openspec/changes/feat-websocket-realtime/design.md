## Context

当前服务端 `/ws` 端点的 `handle_socket` 仅处理一次性 `register` 握手，之后连接空置。前端通过 `setInterval` 每 5 秒轮询 REST API。courier-client 连接后无法进行流量转发。

已有基础：`WsMessage` 协议类型（shared）、`axum` WebSocket 支持（`futures-util` 已引入）、`TunnelRegistry` 骨架（websocket.rs）。

## Goals / Non-Goals

**Goals:**
- 单 `/ws` 端点同时服务 courier-client（流量转发）和前端（状态订阅）
- courier-client 通过 Binary 帧转发 HTTP 流量，控制消息用 JSON
- 前端通过 WebSocket 订阅实时事件，替换 setInterval 轮询
- 服务端广播 `tunnel_connected`、`tunnel_disconnected`、`stats_update` 给所有前端订阅者

**Non-Goals:**
- TCP/UDP 协议转发（仅 HTTP）
- 前端订阅鉴权（本期不做 admin 密码校验）
- 多服务端实例间的状态同步

## Decisions

### 决策 1：单端点多路复用（`/ws`）
首条消息的 `msg_type` 区分连接类型：`register` = courier-client，`subscribe` = 前端。

**替代方案：** 两个独立端点 `/ws/tunnel` 和 `/ws/admin`。  
**选择单端点的原因：** `TunnelRegistry` 天然共享，无需跨 handler 传递状态；代码量少 ~30%；符合 ngrok 惯例。

### 决策 2：TunnelRegistry 结构
```rust
struct TunnelRegistry {
    clients: HashMap<String, ClientSession>,      // courier_id → session
    subscribers: Vec<SplitSink<WebSocket, Message>>, // 前端订阅者
}

struct ClientSession {
    sender: SplitSink<WebSocket, Message>,
    subdomain: String,
    local_port: u16,
    bytes_transferred: u64,
}
```
用 `Arc<Mutex<TunnelRegistry>>` 注入 `AppState`。选 `Mutex` 而非 `RwLock`：写操作（注册/注销/转发）频率与读相当，RwLock 的写锁竞争会抵消优势。

### 决策 3：流量转发帧格式
- **控制消息**（register、heartbeat、subscribe、事件推送）：JSON Text 帧，复用现有 `WsMessage`
- **流量数据**（HTTP 请求/响应字节）：Binary 帧，原始字节无额外封装

**替代方案：** JSON + Base64 编码数据。  
**选 Binary 的原因：** 延迟更低，无 Base64 膨胀（~33% 体积增加），实现更简单。

### 决策 4：stats_update 广播定时器
服务端启动独立 `tokio::spawn` 任务，每 10 秒遍历 `TunnelRegistry.clients` 汇总 `bytes_transferred` 并广播给所有 subscribers。

### 决策 5：新增 WsMessage 类型（shared/src/lib.rs）
新增：`SubscribeRequest`、`TunnelConnectedEvent`、`TunnelDisconnectedEvent`、`StatsUpdateEvent`。

## Risks / Trade-offs

- **Mutex 锁竞争** → 转发路径持锁时间短（仅写入 bytes_transferred），广播前克隆数据后释放锁，降低阻塞风险
- **subscriber 断线残留** → 广播时发送失败则从列表移除，惰性清理
- **前端 WebSocket 断线重连** → 前端需实现指数退避重连；本期不做服务端 ping/pong 保活
- **Binary 帧大小** → 单帧最大 64KB（与 proxy.rs 缓冲区对齐），大于 64KB 的请求需分帧，本期不实现分帧（HTTP 请求通常远小于 64KB）

## Migration Plan

1. 更新 `shared/src/lib.rs`，新增消息类型（向后兼容，仅新增）
2. 更新 `server/src/main.rs`，AppState 增加 `tunnel_registry`
3. 重写 `server/src/websocket.rs`，实现 TunnelRegistry + 完整 handle_socket
4. 更新 `web/src/api/tunnelApi.ts`，新增 WebSocket 封装
5. 更新 `web/src/App.vue`，替换 setInterval

无需数据库 schema 变更。无需停机部署。

## Open Questions

- 前端 WebSocket 断线后重连间隔策略（留待实现时决定，建议 1s/2s/4s 指数退避，上限 30s）
