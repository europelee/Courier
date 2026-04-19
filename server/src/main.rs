//! 内网穿透工具 - 中转服务器（二进制入口）

use clap::Parser;
use courier_server::{AppState, ServerConfig, build_router, require_admin_password};
use courier_server::websocket::TunnelRegistry;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Parser, Debug)]
#[command(author, version, about = "内网穿透工具 - 中转服务器", long_about = None)]
struct Args {
    /// 监听的HTTP端口
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// SQLite数据库路径
    #[arg(short, long, default_value = "tunnels.db")]
    database: String,

    /// 服务器域名
    #[arg(short, long, default_value = "localhost:8080")]
    server_domain: String,

    /// 管理员密码（用于Web管理界面）
    #[arg(short, long)]
    admin_password: Option<String>,

    /// HTTPS 监听端口（可选）
    #[arg(long, default_value = "8443")]
    https_port: Option<u16>,

    /// TLS 证书文件路径
    #[arg(long, default_value = "./certs/server.crt")]
    cert_path: Option<String>,

    /// TLS 密钥文件路径
    #[arg(long, default_value = "./certs/server.key")]
    key_path: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    // 解析命令行参数
    let args = Args::parse();
    let admin_password = match require_admin_password(args.admin_password) {
        Ok(p) => p,
        Err(msg) => {
            error!("{}", msg);
            std::process::exit(1);
        }
    };
    info!("启动隧道穿透服务器");
    info!("配置: HTTP 端口={}, 域名={}", args.port, args.server_domain);

    if let Some(https_port) = args.https_port {
        let cert_path = args.cert_path.as_deref().unwrap_or("./certs/server.crt");
        let key_path = args.key_path.as_deref().unwrap_or("./certs/server.key");
        info!("HTTPS 支持已配置 (端口={}, 证书={}, 密钥={})", https_port, cert_path, key_path);
    }

    // 初始化数据库
    let db = courier_server::db::init_database(&args.database).await?;
    info!("数据库初始化完成");

    // 创建 TunnelRegistry 并启动 stats 定时广播
    let tunnel_registry = Arc::new(Mutex::new(TunnelRegistry::new()));

    let registry_for_stats = tunnel_registry.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            registry_for_stats.lock().await.broadcast_stats().await;
        }
    });

    // 创建应用状态
    let state = AppState {
        db,
        config: Arc::new(ServerConfig {
            server_domain: args.server_domain.clone(),
            admin_password: Some(admin_password),
        }),
        tunnel_registry,
    };

    // 构建路由
    let app = build_router(state);

    // 绑定 HTTP 监听器
    let http_addr = format!("0.0.0.0:{}", args.port)
        .parse::<std::net::SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(http_addr).await?;

    info!("HTTP 服务器监听 http://{}", http_addr);
    info!("测试 HTTP: curl http://127.0.0.1:{}/health", args.port);

    // 启动 HTTP 服务
    axum::serve(listener, app).await?;

    Ok(())
}
