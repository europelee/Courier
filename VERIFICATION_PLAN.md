# MVP 轻量化验证计划 - 2026-04-02

## 🎯 验证目标

验证 MVP 核心功能：
1. ✅ **代码完整** - 编译成功，39 个测试全部通过
2. ✅ **二进制可用** - 直接运行，无依赖

---

## 📋 验证步骤（4 步）

### 步骤 1：编译验证（预计 3-5 分钟）
**目标：** 验证发布版编译成功

```bash
cd /root/.openclaw/workspace-agent_dev/Courier
cargo build --release 2>&1 | grep -E "error|Finished|Compiling" | tail -5
```

**成功标准：**
- ✅ 显示 `Finished release`
- ✅ 没有 `error:`

**验证结果：** [ 待执行 ]

---

### 步骤 2：测试验证（预计 2-3 分钟）
**目标：** 验证所有 39 个测试通过

```bash
cd /root/.openclaw/workspace-agent_dev/Courier
cargo test --quiet 2>&1 | tail -3
```

**成功标准：**
- ✅ 显示 `test result: ok.`
- ✅ 测试数 ≥ 39
- ✅ 0 个失败

**验证结果：** [ 待执行 ]

---

### 步骤 3：二进制检查（预计 1 分钟）
**目标：** 验证编译产物存在

```bash
ls -lh /root/.openclaw/workspace-agent_dev/Courier/target/release/tunnel-*
```

**成功标准：**
- ✅ courier-client 存在，大小 > 1MB
- ✅ courier-server 存在，大小 > 1MB

**验证结果：** [ 待执行 ]

---

### 步骤 4：服务端 + 客户端联调（预计 3-5 分钟）
**目标：** 验证服务端和客户端能正常通信

```bash
# 启动服务器（后台运行）
/root/.openclaw/workspace-agent_dev/Courier/target/release/courier-server \
  --host 127.0.0.1 --port 8080 --db-path /tmp/test.db &
SERVER_PID=$!
sleep 2

# 验证服务器健康检查
echo "=== 服务器健康检查 ==="
curl -s http://127.0.0.1:8080/health
echo ""

# 运行客户端（创建隧道）
echo "=== 客户端注册隧道 ==="
/root/.openclaw/workspace-agent_dev/Courier/target/release/courier-client \
  --server-addr 127.0.0.1:8080 \
  --subdomain test-tunnel \
  --local-port 3000 \
  --token test-token &
CLIENT_PID=$!
sleep 2

# 验证隧道注册（查询服务器）
echo "=== 验证隧道是否已注册 ==="
curl -s http://127.0.0.1:8080/tunnel/list

# 清理
kill $SERVER_PID $CLIENT_PID 2>/dev/null
wait 2>/dev/null
```

**成功标准：**
- ✅ 服务器启动成功
- ✅ `/health` 返回 JSON，包含 `"status":"ok"`
- ✅ 客户端连接成功（无连接错误）
- ✅ 隧道信息出现在 `/tunnel/list` 中
- ✅ 正常关闭，无崩溃

**验证结果：** [ 待执行 ]

---

## 📊 验证汇总

**验证前状态：**
```
□ 编译验证
□ 测试验证
□ 二进制检查
□ 二进制运行
```

**验证后预期：**
```
✅ 编译验证       - 发布版成功编译
✅ 测试验证       - 39/39 测试通过
✅ 二进制检查     - 两个可执行文件存在
✅ 二进制直接运行 - 服务启动，健康检查通过
```

---

## ⏱️ 总耗时

- 步骤 1（编译）：3-5 分钟
- 步骤 2（测试）：2-3 分钟
- 步骤 3（检查）：1 分钟
- 步骤 4（运行）：3-5 分钟

**总计：约 9-14 分钟**

---

## 🚨 失败处理

任何步骤失败 → 立即停止 → 报告完整错误日志

---

## ✅ 验证完成标准

所有 4 个步骤全部成功 → **MVP 已验证可用**
