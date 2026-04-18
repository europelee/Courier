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
