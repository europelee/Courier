# CLAUDE.md + 开发规则文档 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**目标：** 创建 CLAUDE.md 入口文件及 `.claude/rules/` 下的六个规则文件，让 Claude Code 在每次会话中自动获得项目上下文和行为约束。

**架构：** CLAUDE.md 作为轻量入口（≤150 行），每次会话自动加载；详细规则按主题拆分到 `.claude/rules/` 下六个文件，Claude 在需要时按需读取，不浪费上下文窗口。

**技术栈：** Markdown、Claude Code CLAUDE.md 约定

---

## 文件清单

| 操作 | 路径 | 职责 |
|---|---|---|
| 创建 | `CLAUDE.md` | 入口：项目描述、技术栈、常用命令、规则索引 |
| 创建 | `.claude/rules/01-architecture.md` | 模块职责、数据流、端口 |
| 创建 | `.claude/rules/02-rust-conventions.md` | Rust 编码规范、错误处理、命名、日志 |
| 创建 | `.claude/rules/03-frontend-conventions.md` | Vue3/TypeScript 规范、API 调用约定 |
| 创建 | `.claude/rules/04-git-workflow.md` | 分支命名、提交格式、PR 流程 |
| 创建 | `.claude/rules/05-testing.md` | TDD 流程、测试命令、覆盖要求 |
| 创建 | `.claude/rules/06-constraints.md` | 硬性禁止事项清单 |

---

## Task 1：创建 `.claude/rules/` 目录并写 `01-architecture.md`

**文件：**
- 创建：`.claude/rules/01-architecture.md`

- [ ] **步骤 1：创建目录结构**

```bash
mkdir -p /root/Courier/.claude/rules
```

- [ ] **步骤 2：写入 `01-architecture.md`**

内容如下（完整写入）：

```markdown
# 项目架构

## Workspace 布局

```
Courier/
├── shared/          ← 共享类型（WsMessage、HealthCheckResponse 等）
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
| `handlers.rs` | REST 接口：`POST /api/v1/tunnels`、`GET`、`DELETE` |
| `auth.rs` | JWT Claims 生成/验证，防重放（jti 字段） |
| `db.rs` | SQLite 初始化、隧道 CRUD 查询（sqlx） |
| `websocket.rs` | WebSocket 连接处理、隧道注册、子域名冲突检测 |
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
    config: Arc<ServerConfig>,    // 服务器配置（域名、管理员密码）
}
```

修改 `AppState` 时，`main.rs`、`handlers.rs`、`websocket.rs` 都可能需要同步更新。
```

- [ ] **步骤 3：提交**

```bash
git add .claude/rules/01-architecture.md
git commit -m "docs: 添加架构说明规则文件 01-architecture.md"
```

---

## Task 2：写 `02-rust-conventions.md`

**文件：**
- 创建：`.claude/rules/02-rust-conventions.md`

- [ ] **步骤 1：写入文件**

```markdown
# Rust 编码规范

## 错误处理

**规则：** 库层（errors.rs）用 `thiserror` 定义错误枚举；应用层用 `anyhow::Result`。

```rust
// ✅ 正确：定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("隧道不存在: {0}")]
    NotFound(String),
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
}

// ✅ 正确：应用层传播错误
async fn some_handler() -> anyhow::Result<()> {
    let result = db::query().await?;
    Ok(())
}

// ❌ 禁止：生产代码中使用 unwrap
let value = some_result.unwrap(); // 禁止
```

**禁止在非测试代码中使用：**
- `.unwrap()`
- `.expect("...")`（除非真的是不可能发生的不变量，需注释说明）
- `panic!()`

## 异步规范

- 全部使用 `tokio` 异步运行时
- 禁止 `std::thread::sleep`，改用 `tokio::time::sleep`
- 禁止在 async 上下文中做阻塞 I/O，需用 `tokio::task::spawn_blocking`

```rust
// ✅ 正确
tokio::time::sleep(std::time::Duration::from_secs(1)).await;

// ❌ 禁止
std::thread::sleep(std::time::Duration::from_secs(1));
```

## 命名约定

| 类型 | 规范 | 示例 |
|---|---|---|
| 函数/变量 | `snake_case` | `get_tunnel_by_id` |
| 类型/结构体/枚举 | `PascalCase` | `TunnelError`, `AppState` |
| 常量 | `SCREAMING_SNAKE_CASE` | `MAX_TUNNEL_COUNT` |
| 模块文件 | `snake_case` | `tunnel_manager.rs` |

## 日志规范

使用 `tracing` 宏，禁止 `println!` 和 `eprintln!`：

```rust
// ✅ 正确
use tracing::{info, warn, error, debug};
info!("隧道创建成功: courier_id={}", id);
warn!("认证失败: user={}", user_id);
error!("数据库连接错误: {:?}", err);

// ❌ 禁止
println!("隧道创建成功");
eprintln!("错误: {:?}", err);
```

## 模块设计原则

- 每个文件只做一件事（单一职责）
- 文件超过 300 行时，考虑是否需要拆分
- 跨模块只通过 `pub` 函数/类型通信，不暴露内部实现细节
- 共享类型放 `shared/` crate，避免跨 crate 重复定义
```

- [ ] **步骤 2：提交**

```bash
git add .claude/rules/02-rust-conventions.md
git commit -m "docs: 添加 Rust 编码规范规则文件 02-rust-conventions.md"
```

---

## Task 3：写 `03-frontend-conventions.md`

**文件：**
- 创建：`.claude/rules/03-frontend-conventions.md`

- [ ] **步骤 1：写入文件**

```markdown
# 前端编码规范（Vue3 + TypeScript）

## Vue3 组件规范

**只用 Composition API**，禁止 Options API：

```typescript
// ✅ 正确：Composition API
<script setup lang="ts">
import { ref, computed } from 'vue'

const count = ref(0)
const doubled = computed(() => count.value * 2)
</script>

// ❌ 禁止：Options API
export default {
  data() { return { count: 0 } },
  computed: { doubled() { return this.count * 2 } }
}
```

## TypeScript 规范

- 开启严格模式（`strict: true`）
- **禁止 `any` 类型**，用 `unknown` + 类型守卫代替
- 接口名用 `PascalCase`，不加 `I` 前缀

```typescript
// ✅ 正确
interface Tunnel {
  id: string
  subdomain: string
  status: 'active' | 'inactive'
}

// ❌ 禁止
const data: any = response.data
```

## API 调用规范

**所有 API 调用统一在 `src/api/tunnelApi.ts` 中定义**，组件不直接调用 `fetch` 或 `axios`：

```typescript
// ✅ 正确：在 src/api/tunnelApi.ts 中定义
export async function listTunnels(): Promise<Tunnel[]> {
  const response = await fetch('/api/v1/tunnels')
  if (!response.ok) throw new Error(`HTTP ${response.status}`)
  return response.json()
}

// 组件中调用
import { listTunnels } from '@/api/tunnelApi'
const tunnels = await listTunnels()

// ❌ 禁止：组件内直接写 fetch
const resp = await fetch('/api/v1/tunnels') // 禁止写在组件里
```

## 变量声明

- 优先用 `const`，只有确实需要重新赋值时才用 `let`
- 禁止 `var`

## 禁止提交的内容

- `console.log`、`console.error`（调试完毕后必须删除）
- 注释掉的死代码块
- `// TODO` 和 `// FIXME`（提交前必须处理或转为 issue）
```

- [ ] **步骤 2：提交**

```bash
git add .claude/rules/03-frontend-conventions.md
git commit -m "docs: 添加前端编码规范规则文件 03-frontend-conventions.md"
```

---

## Task 4：写 `04-git-workflow.md`

**文件：**
- 创建：`.claude/rules/04-git-workflow.md`

- [ ] **步骤 1：写入文件**

```markdown
# Git 工作流规范

## 分支命名

| 类型 | 格式 | 示例 |
|---|---|---|
| 新功能 | `feat/<简短描述>` | `feat/subdomain-conflict-detection` |
| 修复 | `fix/<简短描述>` | `fix/websocket-reconnect` |
| 日常维护 | `chore/<简短描述>` | `chore/update-dependencies` |
| 文档 | `docs/<简短描述>` | `docs/add-claude-md` |
| 重构 | `refactor/<简短描述>` | `refactor/split-handlers` |

## 提交格式（约定式提交）

格式：`<类型>: <简短描述>`

```
feat: 新增子域名冲突检测
fix: 修复 WebSocket 断线重连问题
chore: 更新 tokio 到 1.36
docs: 添加 API 文档
refactor: 拆分 handlers.rs 模块
test: 添加隧道创建集成测试
```

**规则：**
- 描述用中文，简洁（≤50 字符）
- 不加句号
- 使用现在时（"添加"而非"添加了"）

## PR 流程

1. 从 `main` 创建功能分支
2. 开发完成后提交 PR
3. PR 描述说明：做了什么、为什么、如何测试
4. 所有测试通过后才能合并
5. 合并后删除功能分支

## 硬性禁止

- **禁止** `git push origin main`（直接推送主分支）
- **禁止** `git push --force`（强推任何分支）
- **禁止** 在 commit 中包含 `.env`、证书文件、密钥
- **禁止** 跳过 pre-commit hook（`--no-verify`）

## 有用的命令

```bash
# 查看当前状态
git status
git log --oneline -10

# 创建功能分支
git checkout -b feat/my-feature

# 暂存指定文件（不用 git add .）
git add src/specific_file.rs

# 查看差异
git diff HEAD
```
```

- [ ] **步骤 2：提交**

```bash
git add .claude/rules/04-git-workflow.md
git commit -m "docs: 添加 Git 工作流规范规则文件 04-git-workflow.md"
```

---

## Task 5：写 `05-testing.md`

**文件：**
- 创建：`.claude/rules/05-testing.md`

- [ ] **步骤 1：写入文件**

```markdown
# 测试规范

## TDD 流程（必须遵守）

每个功能或修复都按此顺序进行：

1. **写失败的测试** — 描述预期行为
2. **运行测试，确认失败** — 验证测试有效
3. **写最少代码让测试通过** — 不多写
4. **运行测试，确认通过**
5. **提交**
6. **重构（如有必要）**，再次确认测试通过

## Rust 测试

### 单元测试

放在被测文件底部的 `#[cfg(test)]` 模块：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tunnel_creation() {
        let config = ServerConfig {
            server_domain: "example.com".to_string(),
            admin_password: None,
        };
        assert_eq!(config.server_domain, "example.com");
    }

    #[tokio::test]
    async fn test_async_operation() {
        let result = some_async_fn().await;
        assert!(result.is_ok());
    }
}
```

### 集成测试

放在 `tests/` 目录下（与 `src/` 平级）：

```rust
// tests/tunnel_integration_test.rs
#[tokio::test]
async fn test_register_tunnel_endpoint() {
    // 启动测试服务器
    // 发送 POST /api/v1/tunnels
    // 断言响应状态码和 JSON 结构
}
```

### 运行测试

```bash
# 运行所有测试
cargo test --workspace

# 运行特定测试
cargo test test_tunnel_creation

# 带输出运行（调试用）
cargo test -- --nocapture
```

## 前端测试

使用 Vue Test Utils：

```typescript
// tests/TunnelList.spec.ts
import { mount } from '@vue/test-utils'
import TunnelList from '@/components/TunnelList.vue'

test('renders tunnel list', () => {
  const wrapper = mount(TunnelList, {
    props: { tunnels: [{ id: '1', subdomain: 'test' }] }
  })
  expect(wrapper.text()).toContain('test')
})
```

## 覆盖要求

| 场景 | 要求 |
|---|---|
| 新功能 | 必须有对应测试（happy path + 至少一个错误场景） |
| Bug 修复 | 必须有回归测试（复现 bug 的测试，修复后通过） |
| 重构 | 不得减少测试覆盖，重构前后测试结果相同 |

## 禁止事项

- 测试失败时，不得将任务标记为完成
- 不得跳过测试（`#[ignore]` 需注明原因和 issue 编号）
- 不得用 mock 替代真实的数据库操作（集成测试必须用真实 SQLite）
```

- [ ] **步骤 2：提交**

```bash
git add .claude/rules/05-testing.md
git commit -m "docs: 添加测试规范规则文件 05-testing.md"
```

---

## Task 6：写 `06-constraints.md`

**文件：**
- 创建：`.claude/rules/06-constraints.md`

- [ ] **步骤 1：写入文件**

```markdown
# 硬性约束（禁止事项）

以下规则无例外，不得以"这次情况特殊"为由跳过。

---

## 禁止随意重构

**只修改被明确要求修改的代码。**

- 不得"顺手"修改周边代码的命名、结构、格式
- 不得在修复 bug 时同时重构不相关的函数
- 不得在添加功能时清理"看起来不好"的老代码

> 原因：未经要求的改动扩大了 diff 范围，增加了 review 负担，可能引入新 bug。

---

## 禁止跳过测试

**任何实现都必须先写测试（TDD）。**

- 不得先写实现再补测试
- 不得以"这个改动太小"为由省略测试
- 不得提交测试失败的代码

> 原因：测试是行为的规格说明，先写测试才能确保实现正确。

---

## 禁止直接推送 main

**所有代码改动必须通过 PR。**

- 禁止 `git push origin main`
- 禁止 `git push origin master`
- 禁止 `git push --force` 任何分支

> 原因：直接推送绕过了代码审查，可能破坏其他人的工作。

---

## 禁止无设计文档实现大功能

**超过 50 行新代码的功能，必须先写设计文档。**

- 设计文档路径：`docs/superpowers/specs/YYYY-MM-DD-<功能名>.md`
- 用户确认设计文档后，才能开始实现
- 小修小补（修复 bug、调整配置、更新文档）不需要设计文档

> 原因：大功能如果方向错了，事后修改成本极高。

---

## 禁止在生产代码中使用 `.unwrap()`

**非测试代码中禁止 `.unwrap()` 和无理由的 `.expect()`。**

```rust
// ❌ 禁止
let value = result.unwrap();
let value = result.expect("should work");

// ✅ 正确
let value = result?;
let value = result.map_err(|e| TunnelError::Database(e))?;
```

测试代码（`#[cfg(test)]` 块内）可以使用 `.unwrap()`。

---

## 禁止提交调试代码

以下内容不得出现在提交的代码中：

- 前端：`console.log`、`console.error`、`console.warn`
- Rust：`dbg!(...)`、`println!`（用 `tracing` 替代）
- 任何 `// TODO`、`// FIXME`（提交前必须处理或转为 issue）

---

## 禁止静默添加依赖

**新增任何依赖时，必须在对话中明确说明：**

- 依赖名称和版本
- 为什么需要这个依赖（不能用已有依赖解决吗？）
- 修改了哪个 `Cargo.toml` 或 `package.json`

> 原因：依赖增加了供应链风险和构建时间，需要明确决策。
```

- [ ] **步骤 2：提交**

```bash
git add .claude/rules/06-constraints.md
git commit -m "docs: 添加硬性约束规则文件 06-constraints.md"
```

---

## Task 7：写 `CLAUDE.md` 入口文件

**文件：**
- 创建：`CLAUDE.md`

- [ ] **步骤 1：写入 `CLAUDE.md`**

```markdown
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
```

- [ ] **步骤 2：验证文件行数不超过 150 行**

```bash
wc -l CLAUDE.md
```

预期输出：行数 ≤ 150

- [ ] **步骤 3：提交**

```bash
git add CLAUDE.md
git commit -m "docs: 添加 CLAUDE.md 入口文件"
```

---

## Task 8：最终验证

- [ ] **步骤 1：确认所有文件存在**

```bash
ls -la CLAUDE.md .claude/rules/
```

预期输出：
```
CLAUDE.md
.claude/rules/01-architecture.md
.claude/rules/02-rust-conventions.md
.claude/rules/03-frontend-conventions.md
.claude/rules/04-git-workflow.md
.claude/rules/05-testing.md
.claude/rules/06-constraints.md
```

- [ ] **步骤 2：确认 CLAUDE.md 行数**

```bash
wc -l CLAUDE.md
```

预期：≤ 150 行

- [ ] **步骤 3：确认提交历史**

```bash
git log --oneline -8
```

预期：能看到 Task 1-7 的提交记录

- [ ] **步骤 4：确认 .gitignore 不会忽略 .claude/ 目录**

```bash
git check-ignore -v .claude/rules/01-architecture.md
```

预期：无输出（表示不被忽略）。若有输出，需修改 `.gitignore`。
```

- [ ] **步骤 5：全部完成后做一次汇总提交（如有遗漏文件）**

```bash
git status
# 若有未追踪文件，补充 add 和 commit
```
