# ============================================================================
# 多阶段构建 Dockerfile
# ============================================================================

# ============================================================================
# 阶段 1: 后端编译
# ============================================================================
FROM rust:1.75 AS backend-builder

WORKDIR /build

# 复制源代码
COPY Cargo.toml Cargo.lock ./
COPY server ./server
COPY shared ./shared
COPY client ./client

# 编译后端（release 模式）
RUN cargo build --release --package courier-server

# ============================================================================
# 阶段 2: 前端构建
# ============================================================================
FROM node:22-alpine AS frontend-builder

WORKDIR /build

# 复制前端源代码和配置
COPY web ./

# 安装依赖和构建
RUN npm install && \
    npm run build

# ============================================================================
# 阶段 3: 运行时镜像 - 后端
# ============================================================================
FROM debian:bookworm-slim AS backend

WORKDIR /app

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 复制编译好的后端二进制文件
COPY --from=backend-builder /build/target/release/courier-server /app/courier-server

# 复制证书目录（如果存在）
COPY certs /app/certs 2>/dev/null || true

# 创建数据和日志目录
RUN mkdir -p /app/data /app/logs

# 设置时区
ENV TZ=Asia/Shanghai

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 运行后端
EXPOSE 8080 8443
ENTRYPOINT ["/app/courier-server"]
CMD ["--port", "8080", "--database", "sqlite:/app/data/tunnel.db", "--server-domain", "localhost:8080"]

# ============================================================================
# 阶段 4: 运行时镜像 - 前端 (Nginx)
# ============================================================================
FROM nginx:alpine AS frontend

WORKDIR /app

# 复制 Nginx 配置
COPY nginx.conf /etc/nginx/nginx.conf
COPY default.conf /etc/nginx/conf.d/default.conf

# 复制构建好的前端应用
COPY --from=frontend-builder /build/dist /usr/share/nginx/html

# 复制证书（如果存在）
COPY certs /app/certs 2>/dev/null || true

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD wget --quiet --tries=1 --spider https://localhost:3000/ || exit 1

# 暴露端口
EXPOSE 3000

# ============================================================================
# 最终阶段：完整应用
# ============================================================================
FROM backend AS runtime

# 复制前端应用到后端容器的静态文件目录
COPY --from=frontend /usr/share/nginx/html /app/public

# 暴露所有端口
EXPOSE 8080 8443 3000

# 默认运行后端
CMD ["--port", "8080", "--database", "sqlite:/app/data/tunnel.db", "--server-domain", "localhost:8080"]
