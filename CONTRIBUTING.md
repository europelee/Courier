# 贡献指南 (Contributing Guide)

感谢你对本项目的关注！本指南将帮助你快速上手项目开发。

---

## 📋 目录

1. [开发环境设置](#开发环境设置)
2. [编码规范](#编码规范)
3. [Pull Request 流程](#pull-request-流程)
4. [问题报告](#问题报告)
5. [代码审查](#代码审查)
6. [许可证](#许可证)

---

## 🔧 开发环境设置

### 前置要求

- **Rust** 1.94.1+ - [安装指南](https://rustup.rs/)
- **Node.js** v22+ - [下载地址](https://nodejs.org/)
- **Git** - [安装指南](https://git-scm.com/)
- **OpenSSL** - 用于证书生成

### Step 1：克隆项目

```bash
# 克隆仓库
git clone https://github.com/europelee/Courier.git
cd Courier

# 添加上游远程（用于同步最新代码）
git remote add upstream https://github.com/europelee/Courier.git
```

### Step 2：安装依赖

```bash
# 后端依赖（Rust）
cd server
cargo check
cd ..

# 前端依赖（Node.js）
cd web
npm install
cd ..
```

### Step 3：生成证书和启动服务

```bash
# 生成自签名证书
bash scripts/generate_cert.sh

# 启动所有服务（后端 + 前端）
bash scripts/start.sh

# 或分别启动：
# 后端
./target/release/courier-server --port 8080 --database ":memory:" --server-domain localhost:8080

# 前端（在另一个终端）
cd web && npm run dev
```

### Step 4：验证环境

```bash
# 检查后端健康状态
curl http://localhost:8080/health

# 访问前端
open https://localhost:3000

# 运行测试
cargo test --all
cd web && npm test
```

---

## 📝 编码规范

### Rust 代码规范

#### 命名规范
```rust
// ✅ 正确
pub fn create_tunnel(config: TunnelConfig) -> Result<Tunnel> { }
struct TunnelManager { }
const MAX_CONNECTIONS: u32 = 1000;
let tunnel_id = "tun_abc123";

// ❌ 错误
pub fn CreateTunnel(config: TunnelConfig) { }  // PascalCase for function
struct tunnel_manager { }  // snake_case for struct
```

#### 代码风格
```bash
# 自动格式化
cargo fmt

# 代码检查
cargo clippy -- -D warnings

# 运行测试
cargo test
```

#### 注释规范
```rust
/// 公共函数必须有文档注释
/// 
/// # Arguments
/// * `name` - 隧道名称
/// 
/// # Returns
/// * `Result<Tunnel>` - 创建的隧道或错误
pub fn create_tunnel(name: &str) -> Result<Tunnel> {
    // 实现代码
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_create_tunnel() {
        // 测试代码
    }
}
```

### TypeScript/Vue 代码规范

#### 命名规范
```typescript
// ✅ 正确
interface TunnelInfo {
  id: string;
  name: string;
}

const getTunnels = async () => { }

const MAX_RETRIES = 3;

// ❌ 错误
interface tunnel_info { }  // PascalCase for interfaces
const get_tunnels = () => { }  // camelCase for functions
```

#### 代码风格
```bash
# 格式化
npm run format

# 代码检查
npm run lint

# 类型检查
npm run type-check
```

#### 注释规范
```typescript
/**
 * 获取隧道列表
 * @returns Promise<Tunnel[]> 隧道数组
 */
export const getTunnels = async (): Promise<Tunnel[]> => {
  // 实现代码
};
```

### 通用规范

- ✅ 所有代码必须有注释
- ✅ 变量名要清晰和有意义
- ✅ 函数长度控制在 50 行以内
- ✅ 避免深层嵌套（最多 3 层）
- ✅ 使用模块化设计

---

## 🔀 Pull Request 流程

### 1. Fork 项目

访问 [项目主页](https://github.com/europelee/Courier)，点击 Fork 按钮。

### 2. 创建功能分支

```bash
# 从 main 分支创建新分支
git checkout -b feature/your-feature-name

# 或修复 bug
git checkout -b fix/issue-number

# 或改进文档
git checkout -b docs/improvement-description
```

### 3. 提交代码

```bash
# 查看修改
git status

# 暂存修改
git add .

# 按照 Conventional Commits 规范提交
git commit -m "feat: add tunnel search functionality"
git commit -m "fix: resolve port validation bug"
git commit -m "docs: update API documentation"
git commit -m "test: add unit tests for validation module"
git commit -m "refactor: simplify error handling logic"
```

#### Conventional Commits 规范

```
type(scope): subject

feat:      新功能
fix:       bug 修复
docs:      文档更新
style:     代码风格（不影响功能）
refactor:  代码重构
test:      添加/修改测试
chore:     依赖更新、构建变更等

示例：
✅ feat(api): add tunnel creation endpoint
✅ fix(validation): improve port range validation
✅ docs(readme): update installation instructions
❌ update code
❌ fix bug
```

### 4. 推送分支

```bash
# 推送到你的 fork
git push origin feature/your-feature-name
```

### 5. 创建 Pull Request

1. 访问 GitHub 仓库主页
2. 点击 "Compare & pull request" 按钮
3. 填写 PR 描述（见下方模板）
4. 提交 PR

#### PR 描述模板

```markdown
## 描述
简明扼要地描述你的更改。

## 关联 Issue
修复 #123（如果适用）

## 变更类型
- [ ] Bug 修复
- [ ] 新功能
- [ ] 破坏性变更
- [ ] 文档更新

## 测试
描述你如何测试这些更改：
- [ ] 添加了单元测试
- [ ] 添加了集成测试
- [ ] 手动测试（描述步骤）

## 检查清单
- [ ] 我已遵循项目代码风格
- [ ] 我已进行了自我代码审查
- [ ] 我已添加了必要的注释
- [ ] 我已更新了相关文档
- [ ] 我的更改没有引入新的警告
- [ ] 我已添加了测试来证明修复/功能有效
- [ ] 新的/现有的单元测试通过
```

### 6. 代码审查

我们的维护者将审查你的代码。可能会要求：
- 修复代码风格问题
- 补充缺失的测试
- 更新文档
- 解决冲突

### 7. 合并

一旦 PR 获得批准并通过所有检查，我们将合并它。

---

## 🐛 问题报告

### 报告 Bug

访问 [Issues 页面](https://github.com/europelee/Courier/issues)，点击 "New Issue"，选择 "Bug report"。

#### Bug 报告模板

```markdown
## 描述
简明描述这个 bug。

## 复现步骤
1. 第一步...
2. 第二步...
3. 第三步...

## 预期行为
应该发生什么

## 实际行为
实际发生了什么

## 环境信息
- OS: [e.g. macOS 12.1]
- Rust version: [e.g. 1.94.1]
- Node version: [e.g. 22.0.0]
- Browser: [e.g. Chrome 120]

## 日志/截图
添加相关的日志输出或截图

## 额外信息
任何其他有用的信息
```

### 报告安全问题

⚠️ **不要在 public issue 中报告安全问题**

请发送邮件到 `security@Courier.dev`。

### 提交功能请求

选择 "Feature request" 模板，描述：
- 你想要什么功能
- 为什么需要它
- 如何使用它

---

## ✅ 代码审查

### 审查标准

审查者将检查以下方面：

#### 功能性 (Functionality)
- ✅ 代码按预期工作
- ✅ 没有逻辑错误
- ✅ 处理了边界情况
- ✅ 错误处理完整

#### 代码质量 (Code Quality)
- ✅ 遵循编码规范
- ✅ 代码清晰易懂
- ✅ 适当的注释
- ✅ 没有代码重复

#### 测试 (Testing)
- ✅ 有充分的单元测试
- ✅ 测试覆盖边界情况
- ✅ 测试通过率 100%
- ✅ 添加了集成测试

#### 文档 (Documentation)
- ✅ 文档已更新
- ✅ API 变更已记录
- ✅ README 已更新
- ✅ CHANGELOG 已更新

#### 性能 (Performance)
- ✅ 没有性能下降
- ✅ 没有内存泄漏
- ✅ 算法复杂度合理

#### 安全 (Security)
- ✅ 没有安全漏洞
- ✅ 输入验证完整
- ✅ 没有硬编码的敏感信息

### 审查反馈

我们会提供建设性的反馈。如果有异议：
1. 在 PR 评论中解释你的观点
2. 提供额外的背景信息
3. 保持专业和尊重的态度

---

## 📚 开发资源

### 文档
- [API 文档](./API_DOCUMENTATION.md)
- [项目 README](./README.md)
- [项目总结](./PROJECT_SUMMARY.md)

### 工具
- [Rust 官方文档](https://doc.rust-lang.org/)
- [Vue3 官方文档](https://vuejs.org/)
- [TypeScript 官方文档](https://www.typescriptlang.org/)

### 社区
- [GitHub Discussions](https://github.com/europelee/Courier/discussions)
- [Discord 社区](https://discord.gg/Courier)

---

## 📜 许可证

通过贡献代码，你同意你的代码将在 MIT 许可证下发布。

---

## 🙏 致谢

感谢所有贡献者！你们的努力让这个项目变得更好。

贡献者名单见 [CONTRIBUTORS.md](./CONTRIBUTORS.md)

---

**最后更新：** 2026-04-04  
**维护者：** Courier 团队

---

## 常见问题 (FAQ)

**Q: 我是初学者，可以贡献吗？**  
A: 当然可以！我们欢迎所有级别的贡献者。从简单的文档改进开始。

**Q: 有什么"好的第一个贡献"吗？**  
A: 查看标记为 `good-first-issue` 的 issues。

**Q: 需要多长时间才能得到 PR 审查？**  
A: 通常在 1-3 天内。

**Q: 代码被拒绝了怎么办？**  
A: 这很正常！我们会解释原因，你可以修改后重新提交。

**Q: 可以同时处理多个 PR 吗？**  
A: 可以，但建议先完成一个再开始下一个。
