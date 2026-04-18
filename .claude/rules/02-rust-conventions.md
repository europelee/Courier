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
