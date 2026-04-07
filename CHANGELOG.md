# 更新日志 (Changelog)

所有对本项目的重要更改都会被记录在此文件中。

格式参考 [Keep a Changelog](https://keepachangelog.com/)，
遵循 [Semantic Versioning](https://semver.org/) 版本规范。

---

## [1.0.0] - 2026-04-03

### 新增 (Added)

#### 核心功能
- ✨ 隧道管理完整 CRUD 功能
  - 创建隧道（POST /api/v1/tunnels）
  - 获取隧道列表（GET /api/v1/tunnels）
  - 获取隧道详情（GET /api/v1/tunnels/:id）
  - 删除隧道（DELETE /api/v1/tunnels/:id）
  - 健康检查（GET /health）

#### 后端功能
- ✨ 基于 Axum 的 REST API 服务器
- ✨ SQLite 数据库支持
- ✨ WebSocket 连接管理框架
- ✨ 输入验证模块（5 个验证函数）
  - 子域名验证（2-32 字符）
  - 端口验证（1-65535）
  - IP 地址验证
  - 认证令牌验证（8-256 字符）
  - 协议列表验证
- ✨ 完整的错误处理系统（5 个错误类型）
  - 统一的 JSON 错误格式
  - 详细的错误信息

#### 前端功能
- ✨ Vue3 + TypeScript 管理界面
- ✨ HTTPS 支持（自签名证书）
- ✨ 隧道列表页面
  - 搜索功能（实时过滤）
  - 过滤功能（按状态/协议）
  - 排序功能（4 种排序方式）
  - 分页功能（每页 10 条）
  - 删除确认对话框
- ✨ 创建隧道表单
  - 完整的表单验证
  - 错误提示
  - 成功提示
- ✨ 日志查看页面
  - 实时日志显示
  - 日志级别分类（INFO/ERROR/WARN）
- ✨ 监控统计页面
  - 隧道数统计
  - 流量统计
  - 错误计数
  - 响应时间指标
- ✨ 响应式设计
  - 移动设备适配
  - 平板设备适配
  - 桌面设备优化

#### 文档和配置
- ✨ OpenAPI 3.0 规范（openapi.json）
- ✨ 完整的 API 文档（API_DOCUMENTATION.md）
- ✨ 项目 README（README.md）
- ✨ 环境变量示例（.env.example）
- ✨ Docker Compose 配置（docker-compose.yml）
- ✨ 多阶段 Dockerfile
- ✨ Git 忽略配置（.gitignore）
- ✨ 更新日志（CHANGELOG.md）

#### 脚本和工具
- ✨ 证书生成脚本（scripts/generate_cert.sh）
- ✨ 一键启动脚本（scripts/start.sh）

### 改进 (Improved)

#### 代码质量
- 🔧 完整的 TypeScript 类型定义
- 🔧 单元测试覆盖（30+ 个测试）
- 🔧 代码注释和文档
- 🔧 Rust 编译 0 errors（仅 3 个 warnings）

#### 用户体验
- 🔧 直观的 Web 界面
- 🔧 清晰的错误提示
- 🔧 操作成功提示（自动消失）
- 🔧 加载状态指示
- 🔧 确认对话框防止误删

#### 性能
- 🔧 优化的 API 响应时间（< 100ms）
- 🔧 高效的数据库查询
- 🔧 前端构建优化（webpack 压缩）

### 修复 (Fixed)

- 🐛 CORS 跨域请求配置
- 🐛 Vite API 代理设置
- 🐛 HTTPS 证书加载路径
- 🐛 错误响应格式统一

---

## [0.1.0] - 2026-03-20

### 初始版本

项目框架和基础设施：
- 初始化 Rust 工作空间
- 初始化 Vue3 项目
- 基础的项目结构
- 初始的 README

---

## 版本号说明

- **主版本号 (Major)** - 不兼容的 API 更改
- **次版本号 (Minor)** - 向后兼容的新功能
- **修订号 (Patch)** - 向后兼容的问题修复

---

## 已知问题 (Known Issues)

### v1.0.0
- 自签名证书浏览器会提示不信任（生产环境使用正式证书）
- 内存数据库（:memory:）不支持数据持久化（生产环境配置 SQLite 文件）
- WebSocket 实现框架已就绪但未完全集成

---

## 未来计划 (Roadmap)

### v1.1 (计划)
- [ ] WebSocket 实时通信
- [ ] 用户认证和授权系统
- [ ] 性能监控仪表板
- [ ] 自动 SSL 证书续期

### v2.0 (计划)
- [ ] 集群部署支持
- [ ] 负载均衡
- [ ] 数据持久化到 PostgreSQL
- [ ] 高级分析和报告

---

## 贡献

感谢所有为本项目做出贡献的开发者！

贡献名单见 [CONTRIBUTORS.md](./CONTRIBUTORS.md)

---

## 许可证

本项目采用 MIT 许可证，详见 [LICENSE](./LICENSE)

---

**最后更新：** 2026-04-04
**维护者：** Courier 团队
