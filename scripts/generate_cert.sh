#!/bin/bash

# 证书生成脚本 - TLS 自签名证书自动化
# 支持参数化配置和过期检查

set -e

# 配置参数
CERT_DIR="${CERT_DIR:-./certs}"
CERT_FILE="${CERT_DIR}/server.crt"
KEY_FILE="${CERT_DIR}/server.key"

# 默认证书参数（支持环境变量覆盖）
CN="${CN:-localhost}"
O="${O:-Courier}"
C="${C:-CN}"
DAYS="${DAYS:-365}"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 创建证书目录
log_info "检查证书目录: ${CERT_DIR}"
mkdir -p "${CERT_DIR}"

# 检查 OpenSSL 是否可用
if ! command -v openssl &> /dev/null; then
    log_error "OpenSSL 未安装，请先安装 OpenSSL"
    exit 1
fi

# 检查证书是否已存在
if [ -f "${CERT_FILE}" ] && [ -f "${KEY_FILE}" ]; then
    log_success "证书已存在"
    
    # 检查证书过期时间
    EXPIRY=$(openssl x509 -in "${CERT_FILE}" -noout -enddate 2>/dev/null | cut -d= -f2 || echo "未知")
    log_info "证书过期时间: ${EXPIRY}"
    
    # 检查证书是否即将过期（少于 30 天）
    EXPIRY_EPOCH=$(date -d "${EXPIRY}" +%s 2>/dev/null || echo "0")
    CURRENT_EPOCH=$(date +%s)
    DAYS_LEFT=$(( ($EXPIRY_EPOCH - $CURRENT_EPOCH) / 86400 ))
    
    if [ "$DAYS_LEFT" -lt 30 ] && [ "$DAYS_LEFT" -gt 0 ]; then
        log_warn "证书即将过期 (${DAYS_LEFT} 天)，建议更新"
    elif [ "$DAYS_LEFT" -le 0 ]; then
        log_warn "证书已过期！建议立即重新生成"
        log_info "删除过期证书..."
        rm -f "${CERT_FILE}" "${KEY_FILE}"
    else
        log_info "证书有效期: ${DAYS_LEFT} 天"
        exit 0
    fi
fi

# 生成新的自签名证书
log_info "生成新的自签名证书..."
log_info "  - CN (Common Name): ${CN}"
log_info "  - O (Organization): ${O}"
log_info "  - C (Country): ${C}"
log_info "  - 有效期: ${DAYS} 天"

openssl req -new -x509 \
    -keyout "${KEY_FILE}" \
    -out "${CERT_FILE}" \
    -days "${DAYS}" \
    -nodes \
    -subj "/CN=${CN}/O=${O}/C=${C}" \
    2>/dev/null

# 设置文件权限（私钥: 600, 证书: 644）
chmod 600 "${KEY_FILE}"
chmod 644 "${CERT_FILE}"

# 验证生成的文件
if [ ! -f "${KEY_FILE}" ] || [ ! -f "${CERT_FILE}" ]; then
    log_error "证书生成失败"
    exit 1
fi

# 显示证书信息
log_success "证书已生成成功"
echo ""
echo "📋 证书详情:"
echo "  - 私钥文件: ${KEY_FILE}"
echo "  - 证书文件: ${CERT_FILE}"
echo "  - 文件权限: $(ls -la "${KEY_FILE}" | awk '{print $1}' | cut -c 2-) (key), $(ls -la "${CERT_FILE}" | awk '{print $1}' | cut -c 2-) (cert)"
echo "  - 有效期: ${DAYS} 天"
echo ""

# 显示证书详细信息
echo "📜 证书信息:"
openssl x509 -in "${CERT_FILE}" -noout -text 2>/dev/null | grep -E "Subject:|Issuer:|Not Before|Not After|Public-Key:" | sed 's/^/  /'

log_info "证书已准备好使用"
