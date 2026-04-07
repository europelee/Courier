# 安全政策 (Security Policy)

感谢你重视本项目的安全。此文档描述了我们的安全报告流程和安全最佳实践。

---

## 📋 目录

1. [安全报告程序](#安全报告程序)
2. [已知安全问题](#已知安全问题)
3. [依赖项安全检查](#依赖项安全检查)
4. [安全最佳实践](#安全最佳实践)
5. [响应 SLA](#响应-sla)
6. [常见安全问题](#常见安全问题)

---

## 🔐 安全报告程序

### ⚠️ 请勿公开披露安全漏洞

如果你发现安全漏洞，**请勿在 public issues 或 pull requests 中披露**。这会对用户造成风险。

### 报告安全问题

**通过电子邮件向我们的安全团队报告：**

📧 **security@Courier.dev**

### 报告内容应包括

```markdown
【问题标题】
简明的漏洞标题

【问题描述】
详细描述漏洞是什么以及它如何被利用

【影响范围】
哪些版本受到影响？
- v1.0.0 ❌
- v1.1.0 ✅（已修复）

【复现步骤】
1. 第一步
2. 第二步
3. 第三步

【日志/截图】
相关的错误日志或截图

【建议的修复】
（可选）你建议如何修复它

【联系信息】
你的名字和邮箱（用于安全团队跟进）
```

### 我们承诺

- ✅ 在收到报告后 24 小时内确认
- ✅ 在 7 天内提供初步评估
- ✅ 在 30 天内发布补丁
- ✅ 如果你同意，会在发布时感谢你的报告
- ✅ 尊重你的报告隐私

### 报告示例

```
主题：隧道穿透 v1.0.0 - SQL 注入漏洞

您好，

我在隧道穿透项目中发现了一个 SQL 注入漏洞。

**漏洞描述：**
在创建隧道时，subdomain 参数没有正确转义，允许 SQL 注入攻击。

**受影响版本：**
- v1.0.0

**复现步骤：**
1. 发送 POST 请求到 /api/v1/tunnels
2. 设置 subdomain 为：test'; DROP TABLE tunnels; --
3. 观察数据库中的数据被删除

**建议的修复：**
使用参数化查询而不是字符串拼接

联系方式：reporter@example.com
```

---

## 🚨 已知安全问题

### v1.0.0

| 问题 | 严重程度 | 状态 | 说明 |
|------|--------|------|------|
| 自签名证书不受信任 | 低 | 已知 | 开发环境正常，生产环境使用正式证书 |
| 内存数据库无加密 | 中 | 已知 | 使用 :memory: 数据库，生产环境配置 SQLite 文件加密 |
| 默认无认证机制 | 高 | 计划修复 | v1.1 将添加用户认证 |
| WebSocket 未加密 | 中 | 已知 | WebSocket 框架已就绪，等待完整集成 |
| API 限流未启用 | 中 | 已知 | 可手动配置 ENABLE_RATE_LIMIT=true |

### 已解决的问题

| 问题 | 版本 | 说明 |
|------|------|------|
| CORS 配置不当 | v1.0.0 | ✅ 已修复 |
| HTTPS 证书加载失败 | v1.0.0 | ✅ 已修复 |
| 错误响应格式不统一 | v1.0.0 | ✅ 已修复 |

---

## 🔍 依赖项安全检查

### 定期检查依赖安全性

#### Rust 依赖检查

```bash
# 检查已知的安全漏洞
cargo audit

# 检查过期的依赖
cargo outdated

# 更新依赖（谨慎操作，先测试）
cargo update

# 运行所有测试以确保兼容性
cargo test --all
```

#### Node.js 依赖检查

```bash
# 检查已知的安全漏洞
npm audit

# 修复自动可修复的漏洞
npm audit fix

# 更新依赖
npm update

# 运行所有测试
npm test
```

### 依赖项清单

#### Rust 依赖（后端）

关键依赖及其安全状态：

```toml
# Web 框架
axum = "0.7"           # ✅ 安全，定期更新
tokio = "1.35"         # ✅ 安全，可信赖

# 数据库
sqlx = "0.7"           # ✅ 安全，支持 SQL 注入防护

# 加密和 TLS
rustls = "0.21"        # ✅ 安全，经过审计
rustls-pemfile = "2"   # ✅ 安全

# 序列化
serde = "1.0"          # ✅ 安全，广泛使用
serde_json = "1.0"     # ✅ 安全
```

#### Node.js 依赖（前端）

关键依赖及其安全状态：

```json
{
  "dependencies": {
    "vue": "^3.3",           // ✅ 安全
    "typescript": "^5.0",    // ✅ 安全
    "axios": "^1.6",         // ✅ 安全
    "vite": "^4.5"           // ✅ 安全
  }
}
```

### 安全更新流程

1. **检查漏洞**
   ```bash
   cargo audit && npm audit
   ```

2. **评估影响**
   - 是否影响生产环境？
   - 升级会破坏兼容性吗？

3. **更新依赖**
   ```bash
   cargo update
   npm update
   ```

4. **运行测试**
   ```bash
   cargo test --all
   npm test
   ```

5. **发布补丁版本**
   ```bash
   git tag v1.0.1
   git push --tags
   ```

---

## 🛡️ 安全最佳实践

### 开发时的安全要求

#### ✅ 输入验证

```rust
// ✅ 正确：验证所有输入
pub fn create_tunnel(req: CreateTunnelRequest) -> Result<Tunnel> {
    // 验证子域名长度
    if req.subdomain.len() < 2 || req.subdomain.len() > 32 {
        return Err(ValidationError::InvalidSubdomain);
    }
    
    // 验证端口范围
    if req.local_port < 1 || req.local_port > 65535 {
        return Err(ValidationError::InvalidPort);
    }
    
    // 验证协议列表
    for protocol in &req.protocols {
        if !["http", "https", "tcp", "udp"].contains(&protocol.as_str()) {
            return Err(ValidationError::InvalidProtocol);
        }
    }
    
    // ... 创建隧道
}

// ❌ 错误：接受未验证的输入
pub fn create_tunnel_bad(req: CreateTunnelRequest) -> Tunnel {
    // 直接使用未验证的输入
    Tunnel {
        subdomain: req.subdomain,  // 可能包含 SQL 注入
        port: req.local_port,      // 可能超出范围
    }
}
```

#### ✅ 错误处理

```rust
// ✅ 正确：处理所有错误情况
pub fn delete_tunnel(id: &str) -> Result<()> {
    let tunnel = db.get_tunnel(id)
        .map_err(|e| {
            error!("Failed to get tunnel: {}", e);
            ApiError::NotFound
        })?;
    
    // 验证授权
    if !is_authorized(&tunnel) {
        return Err(ApiError::Unauthorized);
    }
    
    db.delete_tunnel(id)
        .map_err(|e| {
            error!("Failed to delete tunnel: {}", e);
            ApiError::InternalError
        })?;
    
    Ok(())
}

// ❌ 错误：忽略错误
pub fn delete_tunnel_bad(id: &str) {
    db.delete_tunnel(id).unwrap();  // 可能 panic
}
```

#### ✅ 认证和授权

```rust
// ✅ 正确：检查认证令牌
pub async fn handle_request(auth_header: Option<String>) -> Result<Response> {
    let token = auth_header
        .ok_or(ApiError::Unauthorized)?;
    
    let user = validate_token(&token)
        .map_err(|_| ApiError::Unauthorized)?;
    
    // 继续处理请求
    Ok(Response::ok())
}

// ❌ 错误：跳过认证检查
pub async fn handle_request_bad() -> Response {
    // 没有检查认证
    Response::ok()
}
```

#### ✅ 日志安全

```rust
// ✅ 正确：不记录敏感信息
info!("Tunnel created: {}", tunnel.id);  // 只记录 ID

// ❌ 错误：记录敏感信息
error!("Failed with auth token: {}", auth_token);  // 暴露密钥
error!("Database error: {}", db_password);         // 暴露密码
```

#### ✅ HTTPS/TLS

```rust
// ✅ 正确：强制 HTTPS
pub fn server_config() -> ServerConfig {
    ServerConfig {
        https_only: true,
        certificate_path: "./certs/server.crt",
        key_path: "./certs/server.key",
        // ... 其他配置
    }
}

// ❌ 错误：允许 HTTP
pub fn server_config_bad() -> ServerConfig {
    ServerConfig {
        https_only: false,  // 不安全
        // ... 其他配置
    }
}
```

#### ✅ 数据库安全

```rust
// ✅ 正确：使用参数化查询
let query = sqlx::query_as::<_, Tunnel>(
    "SELECT * FROM tunnels WHERE id = ?"
)
.bind(tunnel_id)
.fetch_one(&db)
.await?;

// ❌ 错误：字符串拼接（SQL 注入风险）
let query = format!("SELECT * FROM tunnels WHERE id = '{}'", tunnel_id);
db.query_raw(&query).await?;
```

### 安全检查清单

部署前检查：

- ✅ 所有输入都已验证
- ✅ 所有错误都已正确处理
- ✅ 没有敏感信息在日志中
- ✅ 使用了 HTTPS/TLS
- ✅ 实现了认证和授权
- ✅ 所有依赖都是最新的（cargo audit, npm audit 无错误）
- ✅ 代码审查完成
- ✅ 安全测试通过
- ✅ 没有硬编码的敏感信息（密钥、密码等）
- ✅ 日志级别设置正确

---

## 📞 响应 SLA

### 安全团队联系方式

| 渠道 | 地址 | 响应时间 |
|------|------|--------|
| **电子邮件** | security@Courier.dev | 24 小时 |
| **PGP 密钥** | [下载](./security/pgp-key.asc) | - |
| **Discord** | [加入服务器](https://discord.gg/Courier) | 48 小时 |

### 响应时间承诺（SLA）

| 严重程度 | 初步反馈 | 补丁发布 | 例子 |
|--------|--------|--------|------|
| **Critical** (9-10) | 4 小时 | 1 天 | 远程代码执行 |
| **High** (7-8) | 12 小时 | 3 天 | SQL 注入 |
| **Medium** (5-6) | 24 小时 | 7 天 | 身份验证绕过 |
| **Low** (3-4) | 48 小时 | 30 天 | 信息泄露 |

### 严重程度评估

我们使用 CVSS v3.1 评分标准：

- **Critical (9.0-10.0)** - 可远程执行代码，直接威胁系统安全
- **High (7.0-8.9)** - 可绕过重要安全措施（认证、授权）
- **Medium (5.0-6.9)** - 有限的访问或需要用户交互
- **Low (3.0-4.9)** - 信息泄露或拒绝服务

---

## ❓ 常见安全问题

### Q: 如何安全地在生产环境中部署？

A: 按照以下步骤：

```bash
# 1. 使用正式的 SSL 证书（不是自签名证书）
cp /path/to/real-cert.crt ./certs/server.crt
cp /path/to/real-key.key ./certs/server.key

# 2. 设置环境变量
export PORT=8080
export SERVER_DOMAIN=your-domain.com
export DATABASE_URL=sqlite:/var/lib/tunnel.db
export JWT_SECRET=$(openssl rand -base64 32)

# 3. 启用限流和认证
export ENABLE_RATE_LIMIT=true
export ENABLE_AUTH=true

# 4. 启动服务
docker-compose up -d

# 5. 验证 HTTPS
curl -I https://your-domain.com
```

### Q: 如何保护敏感数据？

A: 遵循这些实践：

```bash
# 1. 使用环境变量而不是配置文件
export DATABASE_PASSWORD=$(cat /run/secrets/db_password)

# 2. 加密数据库连接
export DATABASE_URL=postgresql://user:pass@host/db?sslmode=require

# 3. 定期备份
mysqldump -u root -p database > backup.sql

# 4. 限制文件权限
chmod 600 /etc/tunnel/config.env
```

### Q: 如何检查项目的安全性？

A: 运行这些检查：

```bash
# 检查依赖漏洞
cargo audit
npm audit

# 检查代码质量
cargo clippy
npm run lint

# 运行安全测试
cargo test --test "*_security_test"
npm run test:security

# 使用 SAST 工具（如果可用）
semgrep --config=p/security-audit
```

### Q: 如何报告我发现的问题？

A: 遵循本文档的"安全报告程序"部分。简单来说：

1. **不要公开披露**
2. **发送电子邮件到 security@Courier.dev**
3. **包括重现步骤和建议的修复**
4. **等待我们的回复**

### Q: 生产环境应该禁用哪些功能？

A: 确保以下配置：

```env
# 禁用调试模式
DEBUG=false

# 使用 HTTPS only
HTTPS_ONLY=true

# 启用认证
ENABLE_AUTH=true

# 启用限流
ENABLE_RATE_LIMIT=true

# 设置强 JWT 密钥
JWT_SECRET=<strong-random-key>

# 使用真实证书
SSL_CERT_PATH=/etc/ssl/certs/real-cert.crt
SSL_KEY_PATH=/etc/ssl/private/real-key.key
```

---

## 📚 安全资源

### 外部指南

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [Rust 安全指南](https://anssi-fr.github.io/rust-guide/)
- [Vue.js 安全最佳实践](https://vuejs.org/guide/best-practices/)

### 工具

- [OWASP ZAP](https://www.zaproxy.org/) - 安全测试工具
- [Snyk](https://snyk.io/) - 依赖漏洞扫描
- [Semgrep](https://semgrep.dev/) - 代码静态分析

---

## 🙏 致谢

感谢所有报告安全问题的研究人员和用户。你们的贡献让这个项目更安全。

---

**最后更新：** 2026-04-04  
**维护者：** Courier 团队

---

## 许可证

本安全政策文档采用 MIT 许可证发布。

