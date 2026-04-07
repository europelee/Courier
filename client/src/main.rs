//! 内网穿透工具 - 客户端（CLI）
//! 
//! 核心职责:
//! - 建立与中转服务器的长连接
//! - 将本地服务流量转发到中转服务器
//! - 处理隧道生命周期管理
//! - 实现断线重连机制

mod config;
mod tunnel_manager;
mod proxy;

use clap::Parser;
use config::ClientConfig;
use std::path::PathBuf;
use tracing::info;

/// ============================================================================
/// CLI 命令行参数
/// ============================================================================

#[derive(Parser, Debug)]
#[command(author, version, about = "内网穿透工具 - 客户端", long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(long)]
    config: Option<PathBuf>,

    /// 本地服务端口
    #[arg(long)]
    local_port: Option<u16>,

    /// 中转服务器地址
    #[arg(long)]
    server: Option<String>,

    /// 认证令牌
    #[arg(long)]
    token: Option<String>,

    /// 期望的子域名（可选）
    #[arg(long)]
    subdomain: Option<String>,

    /// 日志级别
    #[arg(long, default_value = "info")]
    log_level: String,
}

/// ============================================================================
/// 主函数
/// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志系统
    let level: tracing::metadata::Level = args.log_level.parse()?;
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_level(true)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("启动隧道穿透客户端");

    // 加载配置
    let config = if let Some(config_path) = args.config {
        ClientConfig::load_from_file(&config_path)?
    } else {
        // 从命令行参数构造配置
        ClientConfig {
            local_port: args.local_port.ok_or_else(|| {
                anyhow::anyhow!("必须指定本地端口 (--local-port 或配置文件)")
            })?,
            server_address: args.server.unwrap_or_else(|| "ws://localhost:8080".to_string()),
            auth_token: args.token.ok_or_else(|| {
                anyhow::anyhow!("必须指定认证令牌 (--token 或配置文件)")
            })?,
            subdomain: args.subdomain.unwrap_or_default(),
            protocols: vec!["http".to_string()],
        }
    };

    info!("配置加载完成: {:?}", config);

    // 创建隧道管理器
    let manager = tunnel_manager::TunnelManager::new(config).await?;

    // 启动隧道
    manager.run().await?;

    Ok(())
}

/// ============================================================================
/// 模块定义
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = Args::try_parse_from(&[
            "courier-client",
            "--local-port", "3000",
            "--server", "ws://example.com:8080",
            "--token", "mytoken",
        ]).unwrap();

        assert_eq!(args.local_port, Some(3000));
        assert_eq!(args.server, Some("ws://example.com:8080".to_string()));
        assert_eq!(args.token, Some("mytoken".to_string()));
    }
}
