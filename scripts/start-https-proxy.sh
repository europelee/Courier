#!/bin/bash

# 启动 HTTPS 后端服务器
# 这个脚本会启动一个支持 HTTPS 8443 的代理

echo "🔒 启动 HTTPS 反向代理服务器..."

# 检查证书
if [ ! -f "./certs/server.crt" ] || [ ! -f "./certs/server.key" ]; then
    echo "❌ 证书文件不存在"
    exit 1
fi

# 使用 stunnel 或 nginx 作为 HTTPS 代理
# 如果没有 stunnel，我们用 Node.js 简单代理

cat > /tmp/https-proxy.js << 'EOF'
const https = require('https');
const http = require('http');
const fs = require('fs');

const options = {
  key: fs.readFileSync('./certs/server.key'),
  cert: fs.readFileSync('./certs/server.crt')
};

https.createServer(options, (req, res) => {
  // 转发请求到 HTTP 后端
  const proxyReq = http.request(
    {
      hostname: 'localhost',
      port: 8080,
      path: req.url,
      method: req.method,
      headers: req.headers
    },
    (proxyRes) => {
      res.writeHead(proxyRes.statusCode, proxyRes.headers);
      proxyRes.pipe(res);
    }
  );

  proxyReq.on('error', (e) => {
    console.error('Proxy error:', e);
    res.writeHead(502);
    res.end('Bad Gateway');
  });

  req.pipe(proxyReq);
}).listen(8443, () => {
  console.log('🔒 HTTPS 代理服务器监听在 https://0.0.0.0:8443');
});
EOF

node /tmp/https-proxy.js &
PROXY_PID=$!

echo "✅ HTTPS 代理启动 (PID: $PROXY_PID)"
sleep 2

echo "📍 HTTPS 代理：https://localhost:8443"
echo "📍 HTTP 后端：http://localhost:8080"
