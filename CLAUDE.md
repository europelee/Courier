# Courier — Claude Code 工作手册

Courier 是一个基于 **Rust + Vue3** 的内网穿透工具（类似 ngrok），支持 HTTP/HTTPS/TCP/UDP 协议代理，提供 Web 管理界面和完整 REST API。

---

## 技术栈

| 层 | 技术 |
|---|---|
| 后端语言 | Rust 1.94+，异步运行时 tokio |
| HTTP 框架 | Axum 0.7 |
| 数据库 | SQLite（sqlx 0.7） |
| 前端 | Vue3 + TypeScript（Composition API） |
| 认证 | JWT（jsonwebtoken） |
| 日志 | tracing + tracing-subscriber |

---

## 常用命令

```bash
# 构建所有 crate
cargo build --workspace

# 运行所有测试
cargo test --workspace

# 运行服务端（开发）
cargo run -p courier-server -- --port 8080 --database :memory: --server-domain localhost:8080

# 运行客户端
cargo run -p courier-client -- --server ws://localhost:8080/ws --local-port 3000

# 前端开发服务器
cd web && npm run dev

# 前端构建
cd web && npm run build

# 代码检查（Rust）
cargo clippy --workspace -- -D warnings

# 格式化（Rust）
cargo fmt --all

# 生成 TLS 证书（开发用）
bash scripts/generate_cert.sh
```

---

## 规则索引

在开始任何任务前，请阅读相关规则文件：

| 文件 | 内容 |
|---|---|
| [`.claude/rules/01-architecture.md`](.claude/rules/01-architecture.md) | Workspace 布局、各模块职责、请求数据流、端口说明 |
| [`.claude/rules/02-rust-conventions.md`](.claude/rules/02-rust-conventions.md) | 错误处理、异步规范、命名约定、日志使用 |
| [`.claude/rules/03-frontend-conventions.md`](.claude/rules/03-frontend-conventions.md) | Vue3 Composition API、TypeScript 严格模式、API 调用约定 |
| [`.claude/rules/04-git-workflow.md`](.claude/rules/04-git-workflow.md) | 分支命名、提交格式（约定式提交）、PR 流程 |
| [`.claude/rules/05-testing.md`](.claude/rules/05-testing.md) | TDD 流程、Rust/前端测试写法、覆盖要求 |
| [`.claude/rules/06-constraints.md`](.claude/rules/06-constraints.md) | **硬性禁止事项**（无例外） |

---

## 快速决策参考

| 情况 | 做法 |
|---|---|
| 要修复 bug | 先读 `05-testing.md`，先写回归测试 |
| 要添加新功能（>50 行） | 先写设计文档，用户确认后再实现 |
| 要提交代码 | 检查 `04-git-workflow.md`，走 PR 流程 |
| 不确定该怎么处理错误 | 读 `02-rust-conventions.md` 错误处理部分 |
| 想重构某段代码 | 先确认用户明确要求了，否则不动 |
