# Phase 4 Day 3 Task 3 - 端到端集成测试详细计划

**日期：** 2026-04-03  
**任务：** 完整的端到端集成测试  
**预计耗时：** 1-2 小时  
**预计完成时间：** 14:30-15:30 GMT+8

---

## 🎯 5 个测试阶段 - 傻瓜式步骤

### 阶段 1：后端启动（5 分钟）

**步骤 1.1：清理旧进程**
```bash
pkill -9 courier-server
sleep 1
```

**步骤 1.2：启动后端服务**
```bash
cd /root/.openclaw/workspace-agent_dev/Courier
. "$HOME/.cargo/env"
timeout 600 ./target/release/courier-server --port 8080 --database ":memory:" --server-domain localhost:8080 > /tmp/test_backend.log 2>&1 &
sleep 3
```

**步骤 1.3：验证后端启动**
```bash
curl -s http://127.0.0.1:8080/health | python3 -m json.tool
# 预期：{"status":"ok", "version":"0.1.0", ...}
```

**✅ 阶段 1 验证签名：**
- [ ] 后端进程启动成功
- [ ] /health 端点返回 ok
- [ ] 无错误消息

---

### 阶段 2：前端启动（5 分钟）

**步骤 2.1：确认前端运行（Vite 已后台运行）**
```bash
curl -s http://127.0.0.1:3000/ | grep -o "<title>.*</title>"
# 预期：<title>隧道穿透 - 管理后台</title>
```

**步骤 2.2：验证前端可访问**
```bash
ps aux | grep -E "vite|npm run dev" | grep -v grep
# 预期：显示 node 进程和 vite 进程
```

**✅ 阶段 2 验证签名：**
- [ ] 前端 localhost:3000 可访问
- [ ] HTML 页面正常返回
- [ ] Vite 进程运行中

---

### 阶段 3：API 测试（30 分钟）

#### 测试 3.1：GET /api/v1/tunnels（空列表）

**命令：**
```bash
echo "【测试 3.1】GET /api/v1/tunnels（空列表）"
curl -s http://127.0.0.1:8080/api/v1/tunnels | python3 -m json.tool
```

**预期结果：**
```json
{
    "tunnels": [],
    "total": 0
}
```

**验证：** ✅ / ❌

---

#### 测试 3.2：POST /api/v1/tunnels（创建隧道 1）

**命令：**
```bash
echo "【测试 3.2】POST /api/v1/tunnels（创建隧道 1）"
RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/tunnels \
  -H "Content-Type: application/json" \
  -d '{"auth_token":"token-test-1","local_port":3000,"subdomain":"tunnel-1","protocols":["http"]}')
echo "$RESPONSE" | python3 -m json.tool
TUNNEL_ID_1=$(echo "$RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['tunnel_id'])" 2>/dev/null)
echo "隧道 ID: $TUNNEL_ID_1"
```

**预期结果：**
```json
{
    "tunnel_id": "tun_XXXXXXXX",
    "public_url": "https://tunnel-1.localhost:8080",
    "server_domain": "localhost:8080"
}
```

**验证：** ✅ / ❌

---

#### 测试 3.3：POST /api/v1/tunnels（创建隧道 2）

**命令：**
```bash
echo "【测试 3.3】POST /api/v1/tunnels（创建隧道 2）"
RESPONSE=$(curl -s -X POST http://127.0.0.1:8080/api/v1/tunnels \
  -H "Content-Type: application/json" \
  -d '{"auth_token":"token-test-2","local_port":3001,"subdomain":"tunnel-2","protocols":["https"]}')
echo "$RESPONSE" | python3 -m json.tool
TUNNEL_ID_2=$(echo "$RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['tunnel_id'])" 2>/dev/null)
echo "隧道 ID: $TUNNEL_ID_2"
```

**预期结果：** 隧道创建成功，返回不同的 tunnel_id

**验证：** ✅ / ❌

---

#### 测试 3.4：GET /api/v1/tunnels（验证列表包含 2 个隧道）

**命令：**
```bash
echo "【测试 3.4】GET /api/v1/tunnels（验证列表）"
curl -s http://127.0.0.1:8080/api/v1/tunnels | python3 -m json.tool
```

**预期结果：**
```json
{
    "tunnels": [
        {"id": "tun_...", "subdomain": "tunnel-2", "local_port": 3001, ...},
        {"id": "tun_...", "subdomain": "tunnel-1", "local_port": 3000, ...}
    ],
    "total": 2
}
```

**验证：** ✅ / ❌

---

#### 测试 3.5：DELETE /api/v1/tunnels/:id（删除隧道 1）

**命令：**
```bash
echo "【测试 3.5】DELETE /api/v1/tunnels/$TUNNEL_ID_1"
curl -s -X DELETE http://127.0.0.1:8080/api/v1/tunnels/$TUNNEL_ID_1 -w "\nHTTP Status: %{http_code}\n"
```

**预期结果：** HTTP 204 No Content

**验证：** ✅ / ❌

---

#### 测试 3.6：GET /api/v1/tunnels（验证只剩 1 个隧道）

**命令：**
```bash
echo "【测试 3.6】GET /api/v1/tunnels（验证删除后）"
curl -s http://127.0.0.1:8080/api/v1/tunnels | python3 -m json.tool
```

**预期结果：** 只有 tunnel-2，total = 1

**验证：** ✅ / ❌

---

**✅ 阶段 3 验证签名：**
- [ ] 3.1 空列表测试通过
- [ ] 3.2 创建隧道 1 通过
- [ ] 3.3 创建隧道 2 通过
- [ ] 3.4 列表包含 2 个隧道
- [ ] 3.5 删除隧道通过
- [ ] 3.6 列表更新正确

---

### 阶段 4：前端界面验证（15 分钟）

**步骤 4.1：前端页面访问**
```bash
echo "【步骤 4.1】前端页面可访问"
curl -s http://127.0.0.1:3000/ | grep -c "隧道管理"
# 预期：1（表示找到了"隧道管理"文字）
```

**步骤 4.2：前端 API 集成验证**
```bash
echo "【步骤 4.2】前端已集成 API（检查 JS 代码）"
curl -s http://127.0.0.1:3000/ | grep -c "localhost:8080"
# 预期：出现后端地址引用
```

**✅ 阶段 4 验证签名：**
- [ ] 前端页面正常显示
- [ ] 页面包含"隧道管理"文字
- [ ] API 代理配置正确

---

### 阶段 5：最终清理和报告（10 分钟）

**步骤 5.1：清理测试隧道**
```bash
echo "【步骤 5.1】清理剩余隧道"
curl -s -X DELETE http://127.0.0.1:8080/api/v1/tunnels/$TUNNEL_ID_2 -w "\nHTTP Status: %{http_code}\n"
```

**步骤 5.2：最终验证空列表**
```bash
echo "【步骤 5.2】验证所有隧道已清除"
curl -s http://127.0.0.1:8080/api/v1/tunnels | python3 -m json.tool
# 预期：{"tunnels": [], "total": 0}
```

**✅ 阶段 5 验证签名：**
- [ ] 清理完成
- [ ] 列表为空
- [ ] 测试结束

---

## 📊 总体测试结果汇总

| 阶段 | 任务 | 结果 |
|------|------|------|
| 1 | 后端启动 | ✅ / ❌ |
| 2 | 前端启动 | ✅ / ❌ |
| 3.1 | GET 空列表 | ✅ / ❌ |
| 3.2 | POST 创建 1 | ✅ / ❌ |
| 3.3 | POST 创建 2 | ✅ / ❌ |
| 3.4 | GET 验证列表 | ✅ / ❌ |
| 3.5 | DELETE 删除 | ✅ / ❌ |
| 3.6 | GET 验证删除 | ✅ / ❌ |
| 4 | 前端验证 | ✅ / ❌ |
| 5 | 清理报告 | ✅ / ❌ |

---

## 🎯 成功标准

**全部通过** = Day 3 Task 3 PASS

```
✅ 所有 5 个 API 操作通过
✅ 前后端通信正常
✅ 数据持久性正确
✅ 错误处理正常
✅ 完整日志记录
```

---

## 📝 记录要求

**所有命令和结果记录到：** `DAY3_TASK3_TEST_LOG.md`

格式：
```
【测试时间】2026-04-03 HH:MM:SS
【阶段 X】...
【命令】...
【结果】...
【验证】✅ / ❌
```

---

**现在开始执行！** 🚀

