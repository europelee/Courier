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
