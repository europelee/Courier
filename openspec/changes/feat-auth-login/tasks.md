## 1. 后端：登录接口

- [x] 1.1 在 `server/src/handlers.rs` 新增 `login` handler：接收 `{"password": "..."}` 请求体，与 `state.config.admin_password` 做常数时间比较，验证通过后调用 `auth::generate_token` 返回 JWT
- [x] 1.2 在 `server/src/main.rs` 的 `build_router` 中注册 `POST /api/v1/auth/login` 路由（无需鉴权）
- [x] 1.3 `admin_password` 改为启动必填：在 `main()` 中检查，未设置时 `error!` 并 `process::exit(1)`

## 2. 后端：鉴权中间件

- [x] 2.1 在 `server/src/auth.rs` 新增 `auth_middleware`：从 `Authorization: Bearer` 提取 token，调用 `validate_auth_token`，失败返回 401
- [x] 2.2 在 `build_router` 中将受保护路由（`/api/v1/tunnels` 全部端点）单独分组，加 `.layer(middleware::from_fn_with_state(state.clone(), auth_middleware))`
- [x] 2.3 在 `handle_subscriber_connection` 中从 WebSocket 握手 query 参数 `?token=` 提取并验证 JWT，无效时发送错误消息并关闭连接

## 3. 后端：测试

- [x] 3.1 `login` handler 单元/集成测试：密码正确返回 200+token、密码错误返回 401、缺少字段返回 400
- [x] 3.2 鉴权中间件测试：无 token 返回 401、过期 token 返回 401、有效 token 通过、`/health` 无需 token
- [x] 3.3 WebSocket 订阅鉴权测试：有效 token 连接成功、无 token 连接被拒绝
- [x] 3.4 未设置 admin_password 时启动逻辑测试

## 4. 前端：API 层

- [x] 4.1 在 `web/src/api/tunnelApi.ts` 新增 `login(password: string): Promise<{token: string, expires_in: number}>` 函数
- [x] 4.2 修改所有受保护 API 调用（`getTunnels`、`createTunnel`、`deleteTunnel`），从 localStorage 读取 token 并注入 `Authorization: Bearer` header
- [x] 4.3 修改 `connectWebSocket`，连接时在 URL 附加 `?token=<token>` query 参数

## 5. 前端：登录页与鉴权流程

- [x] 5.1 在 `web/src/App.vue` 新增响应式变量 `isLoggedIn`（检查 localStorage 中的 `auth_token`）
- [x] 5.2 新增登录视图：密码输入框 + 提交按钮，调用 `login()`，成功存 token 并切换到管理界面，失败显示错误提示
- [x] 5.3 在 header 区域新增登出按钮，点击清除 localStorage token 并切换到登录视图
- [x] 5.4 应用启动时根据 `isLoggedIn` 决定显示登录页还是管理界面（`v-if` 控制）
