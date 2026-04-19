**Status: done**

## Why

当前前端通过每 5 秒轮询 REST API 获取隧道状态，courier-client 的 WebSocket 连接仅完成一次性握手后空置，无法实现流量转发。需要补全 WebSocket 实时通信，使隧道流量转发可用，并让管理界面实时感知隧道状态变化。

## What Changes

- **新增** `TunnelRegistry`：服务端全局注册表，管理所有活跃 client 连接和前端订阅者
- **新增** courier-client 通过 WebSocket 进行流量转发（控制消息用 JSON，数据帧用 Binary）
- **新增** 前端通过 WebSocket 订阅隧道状态事件（上线/下线/流量统计），替换现有 setInterval 轮询
- **修改** `handle_socket`：根据首条消息类型区分 client 连接（`register`）和前端订阅（`subscribe`）
- **新增** 服务端向前端广播事件：`tunnel_connected`、`tunnel_disconnected`、`stats_update`
- **新增** 共享协议消息类型：`subscribe`、`tunnel_connected`、`tunnel_disconnected`、`stats_update`

## Capabilities

### New Capabilities

- `tunnel-traffic-forwarding`: courier-client 通过 WebSocket 与服务端保持长连接，公网请求通过 Binary 帧转发到本地服务
- `frontend-realtime-status`: 前端通过 WebSocket 订阅隧道状态，服务端主动推送上线/下线/流量统计事件

### Modified Capabilities

（无已有 specs，无需 delta）

## Impact

- `server/src/websocket.rs`：新增 `TunnelRegistry` 结构，重写 `handle_socket` 逻辑
- `server/src/main.rs`：`AppState` 增加 `tunnel_registry` 字段
- `shared/src/lib.rs`：新增 4 个 WsMessage 消息类型
- `web/src/api/tunnelApi.ts`：新增 WebSocket 客户端封装
- `web/src/App.vue`：替换 setInterval 轮询为 WebSocket 订阅
- 新增依赖：无（复用现有 `futures-util`、`tokio`、`axum` ws 支持）
