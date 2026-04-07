// 输入验证模块

use regex::Regex;

/// 隧道名称验证（2-32 字符）
pub fn validate_subdomain(subdomain: &str) -> Result<(), String> {
    if subdomain.len() < 2 {
        return Err("子域名至少需要 2 个字符".to_string());
    }
    if subdomain.len() > 32 {
        return Err("子域名最多 32 个字符".to_string());
    }
    if !Regex::new(r"^[a-z0-9-]+$").unwrap().is_match(subdomain) {
        return Err("子域名只允许使用小写字母、数字和连字符".to_string());
    }
    if subdomain.starts_with('-') || subdomain.ends_with('-') {
        return Err("子域名不能以连字符开头或结尾".to_string());
    }
    Ok(())
}

/// 端口验证（1-65535）
pub fn validate_port(port: u16) -> Result<(), String> {
    if port == 0 {
        return Err("端口号必须大于 0".to_string());
    }
    if port > 65535 {
        return Err("端口号不能超过 65535".to_string());
    }
    Ok(())
}

/// IP 地址验证
pub fn validate_ip_address(ip: &str) -> Result<(), String> {
    let ip_regex = Regex::new(
        r"^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$"
    ).unwrap();
    
    if !ip_regex.is_match(ip) {
        return Err("无效的 IP 地址格式".to_string());
    }
    Ok(())
}

/// 认证令牌验证（至少 8 个字符）
pub fn validate_auth_token(token: &str) -> Result<(), String> {
    if token.len() < 8 {
        return Err("认证令牌至少需要 8 个字符".to_string());
    }
    if token.len() > 256 {
        return Err("认证令牌最多 256 个字符".to_string());
    }
    Ok(())
}

/// 协议列表验证
pub fn validate_protocols(protocols: &[String]) -> Result<(), String> {
    if protocols.is_empty() {
        return Err("至少需要指定一个协议".to_string());
    }
    
    let valid_protocols = vec!["http", "https", "tcp", "udp"];
    for protocol in protocols {
        if !valid_protocols.contains(&protocol.as_str()) {
            return Err(format!(
                "无效的协议 '{}', 允许的协议: {}",
                protocol,
                valid_protocols.join(", ")
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_subdomain() {
        assert!(validate_subdomain("my-tunnel").is_ok());
        assert!(validate_subdomain("api-server").is_ok());
        assert!(validate_subdomain("app123").is_ok());
    }

    #[test]
    fn test_invalid_subdomain_too_short() {
        assert!(validate_subdomain("a").is_err());
    }

    #[test]
    fn test_invalid_subdomain_too_long() {
        let long = "a".repeat(33);
        assert!(validate_subdomain(&long).is_err());
    }

    #[test]
    fn test_invalid_subdomain_chars() {
        assert!(validate_subdomain("my_tunnel").is_err());
        assert!(validate_subdomain("MY-TUNNEL").is_err());
        assert!(validate_subdomain("-tunnel").is_err());
        assert!(validate_subdomain("tunnel-").is_err());
    }

    #[test]
    fn test_valid_port() {
        assert!(validate_port(80).is_ok());
        assert!(validate_port(8080).is_ok());
        assert!(validate_port(3000).is_ok());
        assert!(validate_port(65535).is_ok());
    }

    #[test]
    fn test_invalid_port_zero() {
        assert!(validate_port(0).is_err());
    }

    #[test]
    fn test_invalid_port_too_high() {
        assert!(validate_port(65535).is_ok()); // 65535 is max valid port
        // Can't test 65536 as literal because it overflows u16
    }

    #[test]
    fn test_valid_ip() {
        assert!(validate_ip_address("127.0.0.1").is_ok());
        assert!(validate_ip_address("192.168.1.1").is_ok());
        assert!(validate_ip_address("10.0.0.1").is_ok());
    }

    #[test]
    fn test_invalid_ip() {
        assert!(validate_ip_address("256.1.1.1").is_err());
        assert!(validate_ip_address("192.168.1").is_err());
        assert!(validate_ip_address("invalid").is_err());
    }

    #[test]
    fn test_valid_auth_token() {
        assert!(validate_auth_token("test-token-123").is_ok());
        assert!(validate_auth_token(&"a".repeat(256)).is_ok());
    }

    #[test]
    fn test_invalid_auth_token() {
        assert!(validate_auth_token("short").is_err());
        assert!(validate_auth_token(&"a".repeat(257)).is_err());
    }

    #[test]
    fn test_valid_protocols() {
        assert!(validate_protocols(&vec!["http".to_string()]).is_ok());
        assert!(validate_protocols(&vec!["https".to_string(), "tcp".to_string()]).is_ok());
    }

    #[test]
    fn test_invalid_protocols_empty() {
        assert!(validate_protocols(&vec![]).is_err());
    }

    #[test]
    fn test_invalid_protocols_unknown() {
        assert!(validate_protocols(&vec!["ftp".to_string()]).is_err());
    }
}
