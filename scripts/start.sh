#!/bin/bash

echo "🚀 启动隧道穿透服务..."

# 检查/生成证书
bash scripts/generate_cert.sh

# 编译后端
echo ""
echo "🔨 编译后端..."
cd server
cargo build --release --quiet 2>/dev/null
COMPILE_STATUS=$?
cd ..

if [ $COMPILE_STATUS -eq 0 ]; then
    echo "✅ 编译成功"
else
    echo "❌ 编译失败"
    exit 1
fi

# 启动后端
echo ""
echo "🚀 启动后端服务..."
pkill -9 courier-server 2>/dev/null
sleep 1
./target/release/courier-server --port 8080 --database ":memory:" --server-domain localhost:8080 > /tmp/courier-server.log 2>&1 &
BACKEND_PID=$!

sleep 2

# 检查后端是否启动成功
if ps -p $BACKEND_PID > /dev/null; then
    echo "✅ 后端已启动 (PID: $BACKEND_PID)"
else
    echo "❌ 后端启动失败"
    exit 1
fi

echo ""
echo "✅ 所有服务已启动"
echo "📍 前端：http://localhost:3000"
echo "📍 后端 HTTP：http://localhost:8080"
echo "📍 后端 HTTPS：https://localhost:8443（自签名证书）"
echo ""
echo "💡 日志：tail -f /tmp/courier-server.log"
