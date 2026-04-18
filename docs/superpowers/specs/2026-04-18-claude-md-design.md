# 设计文档：CLAUDE.md + 开发规则文档体系

**日期：** 2026-04-18  
**项目：** Courier（内网穿透隧道管理系统）  
**目标：** 添加 CLAUDE.md 及结构化规则文件，指导 Claude Code 在本项目中的工作方式

---

## 背景

Courier 是一个基于 Rust + Vue3 的内网穿透工具（类似 ngrok），技术栈如下：
- **后端：** Rust workspace（shared/server/client crates）、Axum、SQLite、WebSocket
- **前端：** Vue3 + TypeScript（精简结构：App.vue、main.ts、src/api/）
- **现状：** 项目中尚无 CLAUDE.md 或 .claude/ 目录

这些文档的受众是 **Claude Code（AI 助手）**，不是人类开发者。目标是减少每次会话重复说明上下文，并强制执行一致的工作行为。

---

## 文件结构

```
CLAUDE.md                          ← 入口文件（不超过 150 行）
.claude/
  rules/
    01-architecture.md             ← 模块职责、数据流
    02-rust-conventions.md         ← Rust 编码规范
    03-frontend-conventions.md     ← Vue3/TypeScript 规范
    04-git-workflow.md             ← 分支、提交、PR 流程
    05-testing.md                  ← TDD 要求、测试策略
    06-constraints.md              ← 硬性禁止事项清单
```

---

## CLAUDE.md — 入口文件

包含以下内容：
1. 一句话项目描述
2. 技术栈速览（Rust 1.94+、Axum 0.7、SQLite、Vue3、TypeScript）
3. 常用命令（构建、测试、启动、代码检查）
4. 规则索引（指向 .claude/rules/ 下各文件，附一行说明）

保持在 150 行以内——每次会话都会加载，必须保持轻量。

---

## 规则文件说明

### 01-architecture.md
- Workspace 布局：`shared/`、`server/`、`client/`、`web/`
- 服务端各模块职责：auth.rs（JWT 认证）、db.rs（SQLite）、handlers.rs（REST 接口）、websocket.rs（隧道）、validation.rs、errors.rs
- HTTP 请求流：请求 → handlers.rs → auth.rs → db.rs → 响应
- WebSocket 隧道流：客户端连接 → websocket.rs → 转发到本地服务
- 关键端口：8080（HTTP）、8443（HTTPS）、3000（前端开发）

### 02-rust-conventions.md
- 错误处理：库层用 `thiserror`，应用层用 `anyhow`
- **禁止：** 非测试代码中使用 `.unwrap()`，改用 `?` 或显式处理
- 异步：统一使用 `tokio`，禁止 `std::thread::sleep`
- 命名：函数/变量用 `snake_case`，类型用 `PascalCase`，常量用 `SCREAMING_SNAKE_CASE`
- 日志：使用 `tracing` 宏（`info!`、`warn!`、`error!`），禁止 `println!`
- 模块保持单一职责，一个文件只做一件事

### 03-frontend-conventions.md
- 只用 Vue3 Composition API，禁止 Options API
- TypeScript 严格模式，禁止 `any` 类型
- 所有 API 调用统一走 `src/api/` 目录，禁止在组件内直接写 fetch
- 优先使用 `const`，非必要不用 `let`

### 04-git-workflow.md
- 分支命名：`feat/<名称>`、`fix/<名称>`、`chore/<名称>`
- 提交格式：约定式提交（`feat: 新增 X`、`fix: 修复 Y`、`chore: 更新 Z`）
- **禁止：** 直接 `git push origin main`，所有改动必须走 PR
- PR 合并前必须通过所有测试

### 05-testing.md
- TDD 流程：先写失败的测试 → 实现代码 → 测试通过 → 重构
- Rust：使用 `cargo test --workspace`；集成测试放在 `tests/` 目录
- 前端：使用 Vue Test Utils 做组件测试
- 每个新功能必须有测试覆盖
- 每个 bugfix 必须有回归测试
- 测试失败时，不得将任务标记为完成

### 06-constraints.md（硬性禁止事项）
- **禁止随意重构** — 只修改被要求修改的代码，不"顺手优化"周边代码
- **禁止跳过测试** — 任何实现都必须先写测试
- **禁止直接推送 main** — 所有修改走 PR 流程
- **禁止无设计文档实现大功能** — 超过 50 行新代码的功能必须先写设计文档
- **禁止在生产代码中使用 `.unwrap()`** — 测试文件除外
- **禁止提交包含 `console.log` 的前端代码**
- **禁止静默添加依赖** — 新增 Cargo.toml 或 package.json 依赖时必须明确说明

---

## 决策记录

| 决策 | 选择 | 原因 |
|---|---|---|
| 受众 | Claude Code（AI） | 人类文档已存在（CONTRIBUTING.md、README.md） |
| 结构 | CLAUDE.md + .claude/rules/ | CLAUDE.md 保持轻量，规则文件按需加载 |
| 语言 | 中文 | 项目文档体系使用中文 |
| 规则文件命名 | 数字前缀（01-、02-…） | 保证顺序一致，便于引用 |

---

## 范围外事项

- 不修改已有的 CONTRIBUTING.md / README.md / SECURITY.md
- 不添加 CI/CD 配置
- 不添加 lint 配置文件（clippy.toml、eslint config）
