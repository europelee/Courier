# 隧道穿透服务 API 文档

**版本：** 1.0.0  
**最后更新：** 2026-04-03  
**基础 URL：** `http://localhost:8080` (HTTP) 或 `https://localhost:8443` (HTTPS)

---

## 📚 API 端点概览

| 方法 | 端点 | 说明 | 优先级 |
|------|------|------|--------|
| GET | `/health` | 系统健康检查 | ⭐⭐⭐ |
| GET | `/api/v1/tunnels` | 获取隧道列表 | ⭐⭐⭐ |
| POST | `/api/v1/tunnels` | 创建新隧道 | ⭐⭐⭐ |
| GET | `/api/v1/tunnels/{tunnel_id}` | 获取隧道详情 | ⭐⭐ |
| DELETE | `/api/v1/tunnels/{tunnel_id}` | 删除隧道 | ⭐⭐⭐ |

---

## 🔧 系统端点

### 1. 健康检查

**端点：** `GET /health`

**说明：** 检查服务器是否正常运行，包含版本和隧道统计信息

**请求示例：**
```bash
curl -X GET http://localhost:8080/health
```

**响应示例（200 OK）：**
```json
{
  "status": "ok",
  "version": "1.0.0",
  "active_tunnels": 5,
  "uptime": 3600
}
```

**错误响应（500）：**
```json
{
  "code": "INTERNAL_ERROR",
  "message": "数据库连接失败"
}
```

**状态码：**
- `200` - 服务器正常
- `500` - 服务器错误

---

## 🌐 隧道管理 API

### 2. 获取隧道列表

**端点：** `GET /api/v1/tunnels`

**说明：** 获取所有已创建的隧道列表，包含隧道的基本信息和状态

**请求参数：** 无

**请求示例：**
```bash
curl -X GET http://localhost:8080/api/v1/tunnels
```

**响应示例（200 OK）：**
```json
{
  "tunnels": [
    {
      "id": "tun_F74FDEEA",
      "subdomain": "my-app",
      "local_port": 8080,
      "status": "connected",
      "created_at_iso": "2026-04-03T09:28:59+00:00",
      "bytes_transferred": 1048576
    },
    {
      "id": "tun_D1186F04",
      "subdomain": "api-server",
      "local_port": 3000,
      "status": "disconnected",
      "created_at_iso": "2026-04-03T10:15:30+00:00",
      "bytes_transferred": 0
    }
  ],
  "total": 2
}
```

**错误响应（500）：**
```json
{
  "code": "DATABASE_ERROR",
  "message": "无法查询隧道列表",
  "details": {
    "reason": "数据库连接超时"
  }
}
```

**状态码：**
- `200` - 成功获取列表
- `500` - 服务器错误

---

### 3. 创建新隧道

**端点：** `POST /api/v1/tunnels`

**说明：** 创建一个新的隧道连接，返回隧道 ID 和公网访问 URL

**请求体：**
```json
{
  "auth_token": "your-secret-token-min-8-chars",
  "local_port": 8080,
  "subdomain": "my-app",
  "protocols": ["http"]
}
```

**请求示例：**
```bash
curl -X POST http://localhost:8080/api/v1/tunnels \
  -H "Content-Type: application/json" \
  -d '{
    "auth_token": "test-token-123",
    "local_port": 8080,
    "subdomain": "my-tunnel",
    "protocols": ["http"]
  }'
```

**响应示例（201 Created）：**
```json
{
  "tunnel_id": "tun_XXXXXXXX",
  "public_url": "https://my-tunnel.localhost:8080",
  "server_domain": "localhost:8080"
}
```

**请求参数说明：**

| 字段 | 类型 | 必填 | 说明 | 范围 |
|------|------|------|------|------|
| auth_token | string | ✅ | 认证令牌（密钥） | 最少 8 字符 |
| local_port | integer | ✅ | 本地服务端口 | 1-65535 |
| subdomain | string | ❌ | 自定义子域名 | 2-32 字符，允许 a-z, 0-9, - |
| protocols | array | ✅ | 支持的协议列表 | http, https, tcp, udp |

**错误响应（400 Bad Request）：**
```json
{
  "code": "INVALID_PORT",
  "message": "端口参数无效",
  "details": {
    "field": "local_port",
    "value": 99999,
    "reason": "端口必须在 1-65535 之间"
  }
}
```

**错误响应（422 Unprocessable Entity）：**
```json
{
  "code": "VALIDATION_ERROR",
  "message": "请求体验证失败",
  "details": {
    "missing_fields": ["auth_token", "local_port"]
  }
}
```

**状态码：**
- `201` - 隧道创建成功
- `400` - 请求参数错误
- `422` - 请求体格式错误
- `500` - 服务器错误

---

### 4. 获取隧道详情

**端点：** `GET /api/v1/tunnels/{tunnel_id}`

**说明：** 获取指定隧道的详细信息，包括连接状态和统计数据

**路径参数：**
- `tunnel_id` - 隧道 ID (格式：tun_XXXXXXXX)

**请求示例：**
```bash
curl -X GET http://localhost:8080/api/v1/tunnels/tun_F74FDEEA
```

**响应示例（200 OK）：**
```json
{
  "id": "tun_F74FDEEA",
  "subdomain": "my-app",
  "local_port": 8080,
  "status": "connected",
  "created_at_iso": "2026-04-03T09:28:59+00:00",
  "bytes_transferred": 1048576,
  "auth_token": "test-token-123",
  "public_url": "https://my-app.localhost:8080",
  "connected_clients": 2
}
```

**错误响应（404 Not Found）：**
```json
{
  "code": "NOT_FOUND",
  "message": "隧道不存在",
  "details": {
    "tunnel_id": "tun_INVALID"
  }
}
```

**状态码：**
- `200` - 成功获取详情
- `404` - 隧道不存在
- `500` - 服务器错误

---

### 5. 删除隧道

**端点：** `DELETE /api/v1/tunnels/{tunnel_id}`

**说明：** 删除指定的隧道并断开所有连接

**路径参数：**
- `tunnel_id` - 隧道 ID

**请求示例：**
```bash
curl -X DELETE http://localhost:8080/api/v1/tunnels/tun_F74FDEEA
```

**响应示例（204 No Content）：**
```
（无响应体）
```

**错误响应（404 Not Found）：**
```json
{
  "code": "NOT_FOUND",
  "message": "隧道不存在，无法删除"
}
```

**错误响应（500）：**
```json
{
  "code": "DELETE_ERROR",
  "message": "删除隧道失败",
  "details": {
    "tunnel_id": "tun_F74FDEEA",
    "reason": "无法断开活跃连接"
  }
}
```

**状态码：**
- `204` - 隧道已删除
- `404` - 隧道不存在
- `500` - 服务器错误

---

## 📊 数据模型

### TunnelInfo（隧道信息）
```json
{
  "id": "tun_XXXXXXXX",
  "subdomain": "my-tunnel",
  "local_port": 8080,
  "status": "connected",
  "created_at_iso": "2026-04-03T09:28:59+00:00",
  "bytes_transferred": 1048576
}
```

### TunnelDetail（隧道详情）
```json
{
  "id": "tun_XXXXXXXX",
  "subdomain": "my-tunnel",
  "local_port": 8080,
  "status": "connected",
  "created_at_iso": "2026-04-03T09:28:59+00:00",
  "bytes_transferred": 1048576,
  "auth_token": "test-token-123",
  "public_url": "https://my-tunnel.localhost:8080",
  "connected_clients": 2
}
```

### ErrorResponse（错误响应）
```json
{
  "code": "ERROR_CODE",
  "message": "用户友好的错误消息",
  "details": {
    "field": "value",
    "additional": "information"
  }
}
```

---

## 🔐 安全性

### 认证
- 当前版本使用简单的令牌认证
- 所有创建/修改操作需要有效的 `auth_token`
- 建议使用 HTTPS (端口 8443) 进行生产部署

### 速率限制
- 暂未实现速率限制
- 建议在网关层面添加限制

---

## 🧪 测试用例

### 创建隧道完整流程
```bash
# 1. 创建隧道
curl -X POST http://localhost:8080/api/v1/tunnels \
  -H "Content-Type: application/json" \
  -d '{
    "auth_token": "test-token",
    "local_port": 8080,
    "subdomain": "test-app",
    "protocols": ["http"]
  }'

# 2. 获取隧道列表
curl http://localhost:8080/api/v1/tunnels

# 3. 获取隧道详情
curl http://localhost:8080/api/v1/tunnels/tun_XXXXXXXX

# 4. 删除隧道
curl -X DELETE http://localhost:8080/api/v1/tunnels/tun_XXXXXXXX

# 5. 验证删除
curl http://localhost:8080/api/v1/tunnels
```

---

## 📝 错误码参考

| 错误码 | HTTP 状态码 | 说明 |
|--------|-----------|------|
| VALIDATION_ERROR | 422 | 请求体验证失败 |
| INVALID_PORT | 400 | 端口号无效 |
| INVALID_TOKEN | 400 | 认证令牌无效 |
| NOT_FOUND | 404 | 资源不存在 |
| DATABASE_ERROR | 500 | 数据库操作失败 |
| INTERNAL_ERROR | 500 | 服务器内部错误 |

---

## 🚀 使用示例

### Python
```python
import requests

BASE_URL = "http://localhost:8080"

# 创建隧道
response = requests.post(f"{BASE_URL}/api/v1/tunnels", json={
    "auth_token": "test-token",
    "local_port": 8080,
    "subdomain": "my-app",
    "protocols": ["http"]
})
tunnel = response.json()
tunnel_id = tunnel["tunnel_id"]

# 获取列表
tunnels = requests.get(f"{BASE_URL}/api/v1/tunnels").json()

# 删除隧道
requests.delete(f"{BASE_URL}/api/v1/tunnels/{tunnel_id}")
```

### JavaScript/Node.js
```javascript
const BASE_URL = "http://localhost:8080";

// 创建隧道
const tunnel = await fetch(`${BASE_URL}/api/v1/tunnels`, {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({
    auth_token: "test-token",
    local_port: 8080,
    subdomain: "my-app",
    protocols: ["http"]
  })
}).then(r => r.json());

// 获取列表
const tunnels = await fetch(`${BASE_URL}/api/v1/tunnels`).then(r => r.json());

// 删除隧道
await fetch(`${BASE_URL}/api/v1/tunnels/${tunnel.tunnel_id}`, {
  method: "DELETE"
});
```

---

**文档版本：** 1.0.0  
**最后更新：** 2026-04-03  
**维护者：** Courier 团队

