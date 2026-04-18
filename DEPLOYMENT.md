# 部署指南

## 目录

1. [快速开始](#快速开始)
2. [Docker 部署](#docker-部署)
3. [二进制部署](#二进制部署)
4. [Systemd 服务](#systemd-服务)
5. [配置说明](#配置说明)
6. [监控与日志](#监控与日志)

---

## 快速开始

### Docker Compose（推荐）

```bash
# 克隆项目
git clone https://github.com/europelee/Courier.git
cd Courier

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f courier-server

# 停止服务
docker-compose down
```

验证服务状态：
```bash
curl http://localhost:8080/health
```

---

## Docker 部署

### 构建镜像

```bash
docker build -t Courier:latest .
```

### 运行服务器

```bash
docker run -d \
  --name courier-server \
  -p 8080:8080 \
  -v tunnel_data:/data \
  -e SERVER_DOMAIN=your-domain.com:8080 \
  Courier:latest
```

### 运行客户端

```bash
docker run -d \
  --name courier-client \
  -e SERVER_ADDRESS=ws://courier-server:8080 \
  -e LOCAL_PORT=3000 \
  -e AUTH_TOKEN=your_token_here \
  Courier:latest courier-client \
    --local-port 3000 \
    --server ws://courier-server:8080 \
    --token your_token_here
```

---

## 二进制部署

### 编译发布版本

```bash
cargo build --release
```

### 创建应用目录

```bash
sudo mkdir -p /opt/Courier
sudo mkdir -p /var/lib/Courier
sudo mkdir -p /var/log/Courier
```

### 复制二进制文件

```bash
sudo cp target/release/courier-server /opt/Courier/
sudo cp target/release/courier-client /opt/Courier/
sudo chmod +x /opt/Courier/tunnel-*
```

### 创建用户

```bash
sudo useradd -r -s /bin/false tunnel
sudo chown -R tunnel:tunnel /var/lib/Courier
sudo chown -R tunnel:tunnel /var/log/Courier
```

---

## Systemd 服务

### 安装服务

```bash
sudo cp courier-server.service /etc/systemd/system/
sudo systemctl daemon-reload
```

### 启动服务

```bash
# 启动
sudo systemctl start courier-server

# 启用自启动
sudo systemctl enable courier-server

# 查看状态
sudo systemctl status courier-server

# 查看日志
sudo journalctl -u courier-server -f
```

### 停止服务

```bash
sudo systemctl stop courier-server
```

---

## 配置说明

### 服务器配置

```bash
courier-server \
  --port 8080 \
  --database /var/lib/Courier/tunnels.db \
  --server-domain your-domain.com:8080 \
  --admin-password secure_password
```

**参数说明：**
- `--port` - HTTP API 监听端口（默认：8080）
- `--database` - SQLite 数据库路径（默认：tunnels.db）
- `--server-domain` - 公网服务器域名（默认：localhost:8080）
- `--admin-password` - 管理界面密码（可选）

### 客户端配置

**命令行参数：**
```bash
courier-client \
  --local-port 3000 \
  --server ws://courier-server:8080 \
  --token your_auth_token \
  --subdomain your-subdomain \
  --log-level info
```

**配置文件 (config.toml)：**
```toml
[server]
url = "ws://courier-server:8080"
port = 8080
verify_tls = false

[client]
local_port = 3000
protocols = ["http", "https"]
auth_token = "your_auth_token"

[tunnel]
max_retries = 10
initial_backoff_ms = 1000
max_backoff_ms = 60000
heartbeat_interval_s = 30
```

---

## 监控与日志

### 查看日志

**Systemd 日志：**
```bash
sudo journalctl -u courier-server -n 100 -f
```

**Docker 日志：**
```bash
docker logs -f courier-server
```

**应用日志文件：**
```bash
tail -f /var/log/Courier/courier-server.log
```

### 日志级别

支持的日志级别：debug, info, warn, error

设置日志级别：
```bash
RUST_LOG=debug /opt/Courier/courier-server
```

### 健康检查

```bash
# HTTP 健康检查
curl -v http://localhost:8080/health

# 预期响应
{
  "status": "ok",
  "version": "0.1.0",
  "active_tunnels": 5,
  "uptime": 3600
}
```

### 监控指标

**活跃隧道数：**
```bash
curl http://localhost:8080/health | grep active_tunnels
```

**性能监控：**
- CPU 使用率
- 内存使用率
- 网络 I/O
- 数据库连接数

---

## 故障排查

### 服务无法启动

**检查日志：**
```bash
sudo journalctl -u courier-server -n 50
```

**常见问题：**
1. 端口被占用 → 更改 --port 参数
2. 数据库权限 → 检查文件夹权限
3. 域名解析失败 → 验证 --server-domain 参数

### 客户端无法连接

**诊断步骤：**
```bash
# 1. 测试网络连接
ping courier-server

# 2. 测试端口连接
nc -zv courier-server 8080

# 3. 验证认证令牌
curl -H "Authorization: Bearer YOUR_TOKEN" http://courier-server:8080/health
```

### 性能问题

**优化建议：**
1. 增加数据库连接池大小
2. 启用压缩
3. 调整心跳间隔
4. 增加系统文件描述符限制

---

## 生产环境检查清单

- [ ] 配置 HTTPS/TLS 证书
- [ ] 设置强密码
- [ ] 配置防火墙规则
- [ ] 启用日志审计
- [ ] 配置备份策略
- [ ] 设置监控告警
- [ ] 制定灾难恢复计划
- [ ] 定期安全审计
