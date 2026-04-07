# 编译和运行指南

## 环境要求

- **Rust**: 1.75.0 或更高（已验证）
- **Cargo**: 1.75.0 或更高
- **SQLite3**: 3.x
- **操作系统**: Linux / macOS / Windows

## 快速开始

### 1. 进入项目目录

```bash
cd /root/.openclaw/workspace-agent_dev/Courier
```

### 2. 编译项目

#### 检查代码（不生成二进制）
```bash
cargo check --all
```

#### 编译Debug版本（快速编译）
```bash
cargo build --all
```

#### 编译Release版本（优化、较慢）
```bash
cargo build --all --release
```

### 3. 运行测试

#### 运行所有测试
```bash
cargo test --all
```

#### 查看测试详细输出
```bash
cargo test --all -- --nocapture
```

#### 按模块运行测试

**共享协议模块**
```bash
cargo test --lib -p courier-shared
```

**服务器模块**
```bash
cargo test --lib -p courier-server
```

**客户端模块**
```bash
cargo test --lib -p courier-client
```

### 4. 启动服务

#### 启动服务器

```bash
# Debug版本
cargo run -p courier-server -- \
  --port 8080 \
  --server-domain localhost:8080 \
  --database ./tunnels.db

# Release版本（推荐）
cargo run --release -p courier-server -- \
  --port 8080 \
  --server-domain localhost:8080 \
  --database ./tunnels.db
```

输出示例：
```
2026-04-02T00:00:00.000Z INFO tunnel_server: 启动隧道穿透服务器
2026-04-02T00:00:00.001Z INFO tunnel_server: 配置: 端口=8080, 域名=localhost:8080
2026-04-02T00:00:00.002Z INFO tunnel_server: 数据库初始化完成
2026-04-02T00:00:00.003Z INFO tunnel_server: 服务器监听 http://0.0.0.0:8080
```

#### 启动客户端

创建配置文件 `client.toml`：
```toml
local_port = 3000
server_address = "ws://localhost:8080"
auth_token = "test_token_123"
subdomain = ""
protocols = ["http"]
```

然后启动：
```bash
cargo run -p courier-client -- --config ./client.toml
```

或使用命令行参数：
```bash
cargo run -p courier-client -- \
  --local-port 3000 \
  --server ws://localhost:8080 \
  --token test_token_123
```

## 故障排查

### 编译错误：找不到依赖

**解决方案**：
```bash
cargo update
cargo clean
cargo build --all
```

### 端口被占用

如果8080端口已被使用：
```bash
# 找出占用端口的进程
lsof -i :8080

# 使用其他端口启动
cargo run -p courier-server -- --port 9090
```

### SQLite错误

确保数据库目录存在并有写权限：
```bash
mkdir -p ./data
chmod 755 ./data
```

### 日志级别调整

```bash
# 设置日志级别为debug
RUST_LOG=debug cargo run -p courier-server

# 设置为trace（最详细）
RUST_LOG=trace cargo run -p courier-server
```

## 测试报告

### 运行完整测试并生成报告

```bash
#!/bin/bash
cd /root/.openclaw/workspace-agent_dev/Courier

echo "=== 开始测试 ==="
cargo test --all -- --nocapture 2>&1 | tee test_results.log

echo ""
echo "=== 测试统计 ==="
grep -E "test result:|^test " test_results.log
```

### 检查代码覆盖率（可选）

使用 `tarpaulin` 工具：
```bash
# 安装tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --all --out Html --output-dir coverage
```

## 性能测试

### 健康检查端点性能

```bash
# 需要安装wrk
wrk -t4 -c100 -d10s http://localhost:8080/health
```

### 隧道注册API性能

```bash
wrk -t4 -c50 -d10s -c50 -s post.lua http://localhost:8080/api/v1/tunnels
```

其中 `post.lua` 内容：
```lua
request = function()
   wrk.method = "POST"
   wrk.headers["Content-Type"] = "application/json"
   wrk.body = '{"auth_token":"test","local_port":3000,"protocols":["http"]}'
   return wrk.format(nil)
end
```

## 清理构建产物

```bash
# 删除编译结果
cargo clean

# 删除测试日志
rm -f test_results.log

# 删除数据库
rm -f tunnels.db
```

## 常用命令速查

| 命令 | 说明 |
|------|------|
| `cargo check` | 快速检查代码 |
| `cargo build` | 编译Debug版本 |
| `cargo build --release` | 编译Release版本 |
| `cargo test` | 运行所有测试 |
| `cargo run` | 运行主程序 |
| `cargo doc --open` | 生成并打开文档 |
| `cargo fmt` | 格式化代码 |
| `cargo clippy` | 代码质量分析 |
| `cargo clean` | 清理编译产物 |

## 文档生成

```bash
# 生成Rust文档
cargo doc --all --no-deps --open
```

## 下一步

1. ✅ Phase 1 编译验证
2. ⏳ Phase 2：WebSocket实现
3. ⏳ Phase 3：流量转发
4. ⏳ Phase 4：TLS证书管理
5. ⏳ Phase 5：Web管理界面
