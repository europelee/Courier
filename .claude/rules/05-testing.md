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
