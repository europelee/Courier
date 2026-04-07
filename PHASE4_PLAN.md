# Phase 4 开发计划 - 功能增强（1-2 周）

**项目：** 内网穿透工具 (Courier)  
**阶段：** Phase 4 - 功能增强  
**计划时间：** 12 工作日（1-2 周）  
**计划日期：** 2026-04-02  
**目标：** 完成 Web 界面、HTTPS、监控三大功能  

---

## 🎯 Phase 4 三个核心任务

### Task 1：Web 管理界面（3-5 天）

**目标：** 创建图形化界面管理隧道，完全替代 CLI

**功能清单：**
- ✅ 隧道列表页面
  - 显示所有活跃隧道
  - 显示隧道状态（连接/断开）
  - 显示子域名、本地端口、流量统计
  
- ✅ 创建隧道表单
  - 输入子域名（可选，自动生成）
  - 输入本地端口号
  - 输入认证令牌
  - 选择协议（HTTP/HTTPS）
  
- ✅ 隧道管理操作
  - 创建新隧道
  - 删除隧道
  - 重启隧道
  - 查看隧道详情
  
- ✅ 实时监控
  - 流量统计（总计、当前速率）
  - 连接数统计
  - 响应时间监控
  
- ✅ 日志查看
  - 隧道活动日志
  - 错误日志
  - 系统日志
  - 支持过滤和搜索

**技术栈：**
- 前端：Vue 3 + TypeScript + Tailwind CSS
- 后端：Axum REST API（扩展现有）
- 数据库：SQLite（现有）
- 通信：WebSocket（实时更新）

**交付物：**
- `web/` 目录（前端项目）
- `server/src/api.rs`（REST API 扩展）
- `server/src/ws_api.rs`（WebSocket 实时更新）
- 前端编译产物（HTML + JS + CSS）

---

### Task 2：HTTPS/TLS 支持（2-3 天）

**目标：** 加密所有通信，支持 HTTPS 和 WSS

**功能清单：**
- ✅ 服务端 HTTPS
  - 支持 HTTP 和 HTTPS 双协议
  - 自动 HTTP → HTTPS 重定向
  - 可配置证书路径
  
- ✅ 客户端 WSS 支持
  - 自动识别 wss:// 协议
  - TLS 验证（可选禁用用于测试）
  - 重连机制保留
  
- ✅ 证书管理
  - 加载 PEM 格式证书和密钥
  - 支持自签名证书
  - 证书有效期检查
  - 证书更新无需重启（热加载）
  
- ✅ Let's Encrypt 集成（可选）
  - 自动申请证书
  - 自动续期
  - 支持 DNS-01 和 HTTP-01 验证

**技术栈：**
- TLS 库：rustls + tokio-rustls
- ACME 客户端：acme-lib（可选）
- 证书管理：自编写证书加载模块

**交付物：**
- `server/src/tls.rs`（TLS 配置）
- `server/src/certificate.rs`（证书管理）
- `client/src/wss.rs`（WSS 支持）
- 示例配置文件（证书、密钥）

---

### Task 3：监控仪表板（3-4 天）

**目标：** 实时监控系统状态和性能指标

**功能清单：**
- ✅ Prometheus Metrics 导出
  - `/metrics` 端点
  - 关键指标收集：
    - 隧道数量（活跃/总计）
    - 请求数（按状态码）
    - 流量统计（字节/秒）
    - 响应时间（P50/P95/P99）
    - 错误率
    - 连接创建/销毁率
  
- ✅ Grafana 仪表板
  - 实时数据展示（5s 刷新）
  - 流量图表（过去 1 小时、24 小时）
  - 连接数趋势
  - 错误率告警
  - 性能指标排行
  
- ✅ 性能指标收集
  - 内存使用率
  - CPU 使用率
  - 磁盘 I/O
  - 网络 I/O
  - 数据库查询时间
  
- ✅ 告警系统（可选）
  - 高错误率告警
  - 连接异常告警
  - 资源耗尽告警
  - 邮件/Slack 通知

**技术栈：**
- Metrics 库：prometheus-rs
- 时间序列数据库：Prometheus 服务器
- 可视化：Grafana
- 告警规则：PromQL

**交付物：**
- `server/src/metrics.rs`（指标收集）
- `server/src/prometheus.rs`（Prometheus 端点）
- `monitoring/prometheus.yml`（Prometheus 配置）
- `monitoring/grafana-dashboard.json`（Grafana 配置）
- `docker-compose-monitoring.yml`（完整监控栈）

---

## 📋 执行计划（12 工作日）

### 第一周（Days 1-5）

| Day | 任务 | 目标 | 完成度 |
|-----|------|------|--------|
| 1 | Web 界面架构 + API 设计 | 完成项目结构和 API 规范 | 30% |
| 2 | 前端基础（Vue 3 + 页面） | 隧道列表页面可显示 | 40% |
| 3 | 前端 CRUD 操作 | 创建/删除/编辑隧道功能完成 | 50% |
| 4 | TLS 集成 + 证书管理 | HTTPS 和 WSS 支持完成 | 60% |
| 5 | 集成测试 + Bug 修复 | Web + TLS 联调成功 | 70% |

### 第二周（Days 6-10）

| Day | 任务 | 目标 | 完成度 |
|-----|------|------|--------|
| 6 | Prometheus 指标导出 | /metrics 端点正常 | 75% |
| 7 | Grafana 仪表板配置 | 基础仪表板可显示实时数据 | 80% |
| 8 | 性能指标优化 | 内存、CPU 等系统指标收集 | 85% |
| 9 | 端到端测试 + 文档 | 完整流程测试通过 | 95% |
| 10 | 最终验证 + 发布 | Phase 4 完全验证通过 | 100% |

**总耗时：12 工作日**

---

## 📌 关键里程碑

| 日期 | 里程碑 | 状态 |
|------|--------|------|
| Day 2 | Web 界面基础可运行（30%） | ⏳ |
| Day 4 | HTTPS 集成完成（60%） | ⏳ |
| Day 7 | Grafana 上线（80%） | ⏳ |
| Day 10 | Phase 4 完全验证（100%） | ⏳ |

---

## ✅ 验证标准

### Task 1 验证清单（Web 界面）

```
步骤 1：启动服务器
  curl http://localhost:8080/admin
  ✅ 返回 HTML 页面，Vue 应用加载
  
步骤 2：隧道列表
  页面显示：隧道数量、状态、子域名、端口、流量
  ✅ 无错误，UI 正常

步骤 3：创建隧道
  表单填写：端口=3000，token=test-token
  点击创建
  ✅ 隧道成功创建，列表更新
  
步骤 4：删除隧道
  点击删除按钮
  ✅ 隧道成功删除，列表更新
  
步骤 5：日志查看
  点击日志标签
  ✅ 显示隧道活动日志

验证结果：✅ 通过
```

### Task 2 验证清单（HTTPS/TLS）

```
步骤 1：HTTPS 访问
  curl https://localhost:8443/health
  ✅ 返回 {"status":"ok"}，证书验证成功
  
步骤 2：HTTP 重定向
  curl http://localhost:8080/health
  ✅ 自动重定向到 https://localhost:8443
  
步骤 3：WSS 连接
  客户端连接：wss://127.0.0.1:8443
  ✅ 连接建立，无 TLS 错误
  
步骤 4：证书热加载
  更新 /etc/tunnel-certs/cert.pem
  发送 SIGHUP 信号
  ✅ 新证书自动加载，无重启

验证结果：✅ 通过
```

### Task 3 验证清单（监控仪表板）

```
步骤 1：Metrics 端点
  curl http://localhost:8080/metrics
  ✅ 返回 Prometheus 格式数据
  
步骤 2：Prometheus 抓取
  Prometheus 服务器配置抓取 http://localhost:8080/metrics
  ✅ 数据成功抓取，无错误
  
步骤 3：Grafana 仪表板
  打开 http://localhost:3000（Grafana）
  ✅ 导入仪表板 JSON
  ✅ 显示实时流量、连接、错误率图表
  
步骤 4：性能指标
  创建多个隧道，生成流量
  ✅ 图表实时更新，数据准确
  
步骤 5：告警规则
  触发告警条件（例：错误率 > 5%）
  ✅ 告警规则触发，通知发送

验证结果：✅ 通过
```

---

## 🔧 开发环境设置

### 需要安装的工具

```bash
# Node.js + npm（前端开发）
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo bash -
sudo apt-get install -y nodejs

# Prometheus（监控数据库）
wget https://github.com/prometheus/prometheus/releases/download/v2.45.0/prometheus-2.45.0.linux-amd64.tar.gz
tar xzf prometheus-2.45.0.linux-amd64.tar.gz
sudo mv prometheus-2.45.0.linux-amd64 /opt/prometheus

# Grafana（可视化）
sudo apt-get install -y grafana-server
sudo systemctl start grafana-server
```

### Rust 依赖更新

```toml
# server/Cargo.toml 需要添加
[dependencies]
prometheus = "0.13"
tokio-rustls = "0.24"
rustls = "0.21"
rustls-pemfile = "1.0"
tokio-tungstenite = "0.20"  # WebSocket over TLS
```

---

## 📊 资源分配

| 任务 | 时间 | 人力 |
|------|------|------|
| **Task 1: Web 界面** | 3-5 天 | 1 前端 + 1 后端 |
| **Task 2: HTTPS/TLS** | 2-3 天 | 1 后端 |
| **Task 3: 监控** | 3-4 天 | 1 后端 + 1 DevOps |
| **集成 + 测试** | 2 天 | 1 QA + 1 后端 |

---

## 🎯 成功指标

| 指标 | 目标 | 验证方法 |
|------|------|---------|
| **Web 界面响应时间** | < 200ms | Apache Bench |
| **HTTPS 性能开销** | < 5% | TLS benchmark |
| **隧道管理延迟** | < 100ms | 监控指标 |
| **监控数据新鲜度** | < 10s | Grafana 刷新率 |
| **可用性** | > 99.9% | Uptime 监控 |

---

## 📝 每日汇报格式

**Morning（每天 08:00）：**
```
Day X 早晨汇报

【昨日完成】
- ✅ 任务 1：完成度 XX%
- ✅ 任务 2：完成度 XX%

【今日计划】
- 📝 任务 A：目标 XX
- 📝 任务 B：目标 XX

【遇到的问题】
- 问题 1：状态（已解决/进行中/待解决）
- 问题 2：状态

【预计今晚完成度】
- XX%
```

**Evening（每天 18:00）：**
```
Day X 晚间汇报

【今日实际完成】
- ✅ 任务 A：完成度 XX%
- ✅ 任务 B：完成度 XX%

【遇到的问题和解决】
- 问题：描述
- 解决：方案

【代码提交】
- Commit 1：描述
- Commit 2：描述

【明天重点】
- 优先级 1：任务描述
- 优先级 2：任务描述

【预计完成度】
- XX%
```

---

## 🚀 立即开始

**第一步：确认计划**
- ✅ 阅读本文档
- ✅ 确认 3 个核心任务
- ✅ 确认时间表（12 天）

**第二步：环境准备**
```bash
# 1. 安装前端工具
node --version  # 需要 v16+
npm --version   # 需要 v7+

# 2. 安装监控工具
# Prometheus + Grafana（可选，Day 6 才需要）

# 3. 更新 Rust 依赖
cd /root/.openclaw/workspace-agent_dev/Courier
cargo update
```

**第三步：开始 Task 1（Web 界面）**
```bash
# Day 1 早晨任务
mkdir -p web
cd web
npm init -y  # 初始化 Vue 3 项目
# 或使用 Vite
npm create vite@latest . -- --template vue
```

---

## 📌 不能做的事情

❌ 跳过任何验证步骤  
❌ 不进行测试就认为完成  
❌ 隐瞒问题或进展缓慢  
❌ 修改时间表（需架构师批准）  
❌ 功能不完整就标记为"完成"

---

## 📌 必须做的事情

✅ 每天早晚汇报进度  
✅ 遇到问题立即反馈  
✅ 代码必须有单元测试  
✅ 新功能必须通过验证清单  
✅ 每个 Task 完成后生成文档

---

## 🎯 最终目标

**Phase 4 完成时：**
1. Web 管理界面全功能可用
2. HTTPS/TLS 安全通信完成
3. Prometheus + Grafana 监控上线
4. 所有验证清单通过
5. 完整文档和部署指南
6. 可部署到生产环境

---

**计划生成日期：** 2026-04-02 20:09 GMT+8  
**计划负责人：** 开发工程师（执行）+ 架构师（监督）  
**计划状态：** ⏳ 待开始（Day 1）

---

## 快速导航

- 📖 查看此计划：`cat PHASE4_PLAN.md`
- 🚀 开始 Task 1：`cd web && npm create vite@latest ...`
- 📊 查看进度：每日汇报
- ✅ 验证完成：执行验证清单
