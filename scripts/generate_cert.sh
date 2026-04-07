#!/bin/bash

CERT_DIR="./certs"
CERT_FILE="$CERT_DIR/server.crt"
KEY_FILE="$CERT_DIR/server.key"

mkdir -p $CERT_DIR

if [ -f "$CERT_FILE" ]; then
    echo "✅ 证书已存在"
    EXPIRY=$(openssl x509 -in "$CERT_FILE" -noout -enddate | cut -d= -f2)
    echo "📅 有效期至：$EXPIRY"
else
    echo "🔄 生成新证书..."
    openssl genrsa -out $KEY_FILE 4096
    openssl req -new -x509 -key $KEY_FILE -out $CERT_FILE -days 365 -subj "/CN=localhost/O=Tunnel-Penetrator/C=CN"
    echo "✅ 证书已生成"
fi
