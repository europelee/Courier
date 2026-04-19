Status: done

## Why

当前所有 REST API 端点（隧道列表、创建、删除）和 WebSocket 订阅均无需任何认证即可访问，任何人都能操作隧道数据。需要加入管理员登录机制，保护 Web 管理界面及其后端 API。

## What Changes

- 新增 `POST /api/v1/auth/login` 登录接口，接受管理员密码，返回 JWT token
- 为所有隧道管理 REST API 和 WebSocket 订阅连接添加 JWT 鉴权保护
- 前端新增登录页，未登录时拦截跳转，登录后将 token 存入 localStorage 并注入请求 header
- `admin_password` CLI 参数由可选改为启动必需（未设置时拒绝启动）
- `admin_password` 同时作为 JWT 签名密钥

## Capabilities

### New Capabilities

- `admin-login`: 管理员用密码换取 JWT token 的登录流程
- `api-auth-guard`: 受保护 API 路由的 JWT 鉴权中间件
- `frontend-auth`: 前端登录页、token 存储与 API 请求鉴权头注入

### Modified Capabilities

## Impact

- `server/src/main.rs`：路由分组，受保护路由加中间件；`admin_password` 改为必填
- `server/src/auth.rs`：新增鉴权中间件函数
- `server/src/handlers.rs`：新增 `login` handler
- `web/src/App.vue`：新增登录视图，未登录时隐藏管理界面
- `web/src/api/tunnelApi.ts`：API 调用统一注入 `Authorization: Bearer` header，WebSocket 连接带 `?token=` query 参数
