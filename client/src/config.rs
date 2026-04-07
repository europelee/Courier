//! 客户端配置管理

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 客户端配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// 本地服务的端口号
    pub local_port: u16,

    /// 中转服务器地址 (WebSocket URL)
    pub server_address: String,

    /// 认证令牌
    pub auth_token: String,

    /// 期望的子域名（可选，空字符串表示由服务器生成）
    #[serde(default)]
    pub subdomain: String,

    /// 支持的协议列表
    #[serde(default)]
    pub protocols: Vec<String>,
}

impl ClientConfig {
    /// 从TOML配置文件加载配置
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: ClientConfig = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// 保存配置到TOML文件
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// 验证配置的有效性
    pub fn validate(&self) -> Result<()> {
        // 验证本地端口
        if !courier_shared::validate_local_port(self.local_port) {
            anyhow::bail!("Invalid local port: {}", self.local_port);
        }

        // 验证认证令牌
        if self.auth_token.is_empty() {
            anyhow::bail!("Auth token cannot be empty");
        }

        // 验证服务器地址
        if self.server_address.is_empty() {
            anyhow::bail!("Server address cannot be empty");
        }

        // 如果指定了子域名，验证其格式
        if !self.subdomain.is_empty() && !courier_shared::validate_subdomain(&self.subdomain) {
            anyhow::bail!("Invalid subdomain format: {}", self.subdomain);
        }

        Ok(())
    }

    /// 生成示例配置文件内容
    pub fn sample() -> String {
        r#"# 隧道穿透工具 - 客户端配置文件

# 本地服务监听的端口号
local_port = 3000

# 中转服务器的WebSocket地址
server_address = "ws://localhost:8080"

# 客户端认证令牌
auth_token = "your_auth_token_here"

# 期望的子域名（可选，留空则由服务器自动生成）
subdomain = ""

# 支持的协议列表
protocols = ["http", "https"]
"#
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = ClientConfig {
            local_port: 3000,
            server_address: "ws://localhost:8080".to_string(),
            auth_token: "token123".to_string(),
            subdomain: String::new(),
            protocols: vec!["http".to_string()],
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_port() {
        let config = ClientConfig {
            local_port: 0,
            server_address: "ws://localhost:8080".to_string(),
            auth_token: "token123".to_string(),
            subdomain: String::new(),
            protocols: vec!["http".to_string()],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_token() {
        let config = ClientConfig {
            local_port: 3000,
            server_address: "ws://localhost:8080".to_string(),
            auth_token: String::new(),
            subdomain: String::new(),
            protocols: vec!["http".to_string()],
        };

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sample_config_generation() {
        let sample = ClientConfig::sample();
        assert!(sample.contains("local_port"));
        assert!(sample.contains("auth_token"));
    }
}
