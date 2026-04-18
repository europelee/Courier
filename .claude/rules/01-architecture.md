# 项目架构

## Workspace 布局

```
Courier/
├── shared/          ← 共享类型库（lib.rs 导出以下内容）
│   └── src/
│       └── lib.rs       ← 协议类型（WsMessage、RegisterRequest、CourierEstablished）
│                           错误类型（CourierError、ErrorCode）
│                           工具函数（generate_subdomain、validate_subdomain 等）
├── server/          ← 服务端（Axum HTTP + WebSocket 服务器）
│   └── src/
│       ├── main.rs       ← 入口、路由注册、AppState 定义
│       ├── handlers.rs   ← REST API 处理器（隧道 CRUD）
│       ├── auth.rs       ← JWT 生成与验证
│       ├── db.rs         ← SQLite 操作（sqlx）
│       ├── websocket.rs  ← WebSocket 隧道逻辑
│       ├── validation.rs ← 请求参数校验
│       └── errors.rs     ← 统一错误类型（thiserror）
├── client/          ← 客户端（连接服务器，转发本地流量）
│   └── src/
│       ├── main.rs          ← 入口、CLI 参数
│       ├── config.rs        ← 客户端配置
│       ├── tunnel_manager.rs← 隧道连接管理
│       └── proxy.rs         ← 本地代理转发
└── web/             ← 前端管理界面（Vue3 + TypeScript）
    └── src/
        ├── main.ts          ← 应用入口
        ├── App.vue          ← 根组件
        └── api/
            └── tunnelApi.ts ← 所有 API 调用集中在此
```

## 服务端模块职责

| 文件 | 职责 |
|---|---|
| `main.rs` | 启动服务、路由注册、`AppState`（db + config）定义 |
| `handlers.rs` | REST 接口：`POST /api/v1/tunnels`（create）、`GET /api/v1/tunnels`（list）、`GET /api/v1/tunnels/:courier_id`（get_tunnel_status）、`DELETE /api/v1/tunnels/:courier_id`（delete） |
| `auth.rs` | JWT Claims 生成/验证，防重放（jti 字段） |
| `db.rs` | SQLite 初始化、隧道 CRUD 查询（sqlx） |
| `websocket.rs` | WebSocket 连接处理、隧道注册、子域名冲突检测；维护 `WsConnectionManager`（`Arc<Mutex<HashMap>>` 连接状态管理器） |
| `validation.rs` | 请求参数格式校验 |
| `errors.rs` | 统一错误枚举（thiserror），转换为 HTTP 响应 |

## 请求数据流

### HTTP REST 请求
```
客户端请求
  → main.rs 路由匹配
  → handlers.rs（提取参数、调用 db）
  → auth.rs（可选鉴权）
  → db.rs（SQLite 操作）
  → JSON 响应
```

### WebSocket 隧道流
```
client 进程连接 /ws
  → websocket.rs 接收 "register" 消息
  → 生成 courier_id + subdomain
  → 返回 "tunnel_established" 消息
  → 双向转发：公网请求 ↔ 本地服务
```

## 关键端口

| 端口 | 协议 | 用途 |
|---|---|---|
| 8080 | HTTP | 服务端主端口（REST API + WebSocket） |
| 8443 | HTTPS | TLS 端口（需配置证书） |
| 3000 | HTTP | 前端开发服务器（`npm run dev`） |

## AppState 结构

```rust
struct AppState {
    db: SqlitePool,                // SQLite 连接池
    config: Arc<ServerConfig>,    // 含 server_domain: String, admin_password: Option<String>
}
```

修改 `AppState` 时，`main.rs`、`handlers.rs`、`websocket.rs` 都可能需要同步更新。
