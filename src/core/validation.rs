//! Pure validation logic for server names and configurations
//!
//! This module contains validation functions that can be tested in isolation
//! without requiring I/O operations or external dependencies.

use crate::server::ServerType;

/// Validates server name format and security constraints
pub fn validate_server_name(server_name: &str) -> Result<(), String> {
    if server_name.is_empty() {
        return Err("Server name cannot be empty".to_string());
    }

    if server_name.len() > 256 {
        return Err("Server name is too long (max 256 characters)".to_string());
    }

    // Check for potentially malicious patterns
    if server_name.contains("..") {
        return Err("Server name contains potentially unsafe path traversal patterns".to_string());
    }

    Ok(())
}

/// Validates NPM package name format
pub fn validate_npm_package_name(package_name: &str) -> Result<(), String> {
    if package_name.is_empty() {
        return Err("NPM package name cannot be empty".to_string());
    }

    // Basic NPM package name validation
    if package_name.starts_with('.') || package_name.starts_with('_') {
        return Err("NPM package name cannot start with '.' or '_'".to_string());
    }

    if package_name.len() > 214 {
        return Err("NPM package name is too long (max 214 characters)".to_string());
    }

    // Check for scoped packages
    if package_name.starts_with('@') && !package_name.contains('/') {
        return Err("Scoped NPM package must contain '/' after scope".to_string());
    }

    Ok(())
}

/// Validates Docker image name format
pub fn validate_docker_image_name(image_name: &str) -> Result<(), String> {
    if image_name.is_empty() {
        return Err("Docker image name cannot be empty".to_string());
    }

    // Docker image names must be lowercase (excluding tag after ':')
    let image_part = if let Some(colon_pos) = image_name.find(':') {
        &image_name[..colon_pos]
    } else {
        image_name
    };

    if image_part.chars().any(|c| c.is_uppercase()) {
        return Err("Docker image names must be lowercase".to_string());
    }

    // Check for registry prefix
    if image_name.contains("://") {
        return Err("Docker image name should not contain protocol".to_string());
    }

    Ok(())
}

/// Validates URL format for binary downloads
pub fn validate_binary_url(url: &str) -> Result<(), String> {
    if url.is_empty() {
        return Err("Binary URL cannot be empty".to_string());
    }

    if !url.starts_with("https://") {
        return Err("Binary URL must use HTTPS for security".to_string());
    }

    // Check for common malicious patterns
    if url.contains("localhost") || url.contains("127.0.0.1") || url.contains("0.0.0.0") {
        return Err("Binary URL cannot point to localhost".to_string());
    }

    Ok(())
}

/// Validates server type-specific constraints
pub fn validate_server_type_constraints(server_type: &ServerType) -> Result<(), String> {
    match server_type {
        ServerType::Npm {
            package,
            version: _,
        } => validate_npm_package_name(package),
        ServerType::Docker { image, tag: _ } => validate_docker_image_name(image),
        ServerType::Binary { url, checksum: _ } => validate_binary_url(url),
        ServerType::Python {
            package: _,
            version: _,
        } => {
            // Python validation could be added here
            Ok(())
        }
    }
}

/// Checks if a server name indicates a potentially risky installation
pub fn assess_server_risk_level(server_name: &str) -> RiskLevel {
    // Check for binary downloads from unknown sources first (highest priority)
    if server_name.starts_with("https://") {
        if server_name.contains("github.com") || server_name.contains("releases.hashicorp.com") {
            return RiskLevel::Medium; // Known safe sources
        } else {
            return RiskLevel::High; // Unknown sources
        }
    }

    // Check for development/test packages
    if server_name.contains("test")
        || server_name.contains("dev")
        || server_name.contains("example")
    {
        return RiskLevel::Medium;
    }

    // Check for unofficial packages
    if !server_name.starts_with("@modelcontextprotocol/") && !server_name.starts_with("@anthropic/")
    {
        return RiskLevel::Medium;
    }

    RiskLevel::Low
}

#[derive(Debug, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_server_name_success() {
        assert!(validate_server_name("valid-server-name").is_ok());
        assert!(validate_server_name("@scope/package").is_ok());
    }

    #[test]
    fn test_validate_server_name_empty() {
        let result = validate_server_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_validate_server_name_too_long() {
        let long_name = "a".repeat(257);
        let result = validate_server_name(&long_name);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));
    }

    #[test]
    fn test_validate_server_name_path_traversal() {
        let result = validate_server_name("../malicious-path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("path traversal"));
    }

    #[test]
    fn test_validate_npm_package_name_success() {
        assert!(validate_npm_package_name("lodash").is_ok());
        assert!(validate_npm_package_name("@types/node").is_ok());
    }

    #[test]
    fn test_validate_npm_package_name_invalid_start() {
        assert!(validate_npm_package_name(".hidden").is_err());
        assert!(validate_npm_package_name("_private").is_err());
    }

    #[test]
    fn test_validate_npm_package_name_scoped_invalid() {
        let result = validate_npm_package_name("@scope");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must contain '/'"));
    }

    #[test]
    fn test_validate_docker_image_name_success() {
        assert!(validate_docker_image_name("nginx").is_ok());
        assert!(validate_docker_image_name("user/repo").is_ok());
    }

    #[test]
    fn test_validate_docker_image_name_uppercase() {
        let result = validate_docker_image_name("Nginx");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase"));
    }

    #[test]
    fn test_validate_binary_url_success() {
        assert!(
            validate_binary_url("https://github.com/user/repo/releases/download/v1.0/binary")
                .is_ok()
        );
    }

    #[test]
    fn test_validate_binary_url_insecure() {
        let result = validate_binary_url("http://example.com/binary");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("HTTPS"));
    }

    #[test]
    fn test_validate_binary_url_localhost() {
        let result = validate_binary_url("https://localhost/binary");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("localhost"));
    }

    #[test]
    fn test_assess_server_risk_level() {
        assert_eq!(
            assess_server_risk_level("@modelcontextprotocol/server-filesystem"),
            RiskLevel::Low
        );
        assert_eq!(assess_server_risk_level("test-package"), RiskLevel::Medium);
        assert_eq!(
            assess_server_risk_level("random-package"),
            RiskLevel::Medium
        );
        assert_eq!(
            assess_server_risk_level("https://sketchy-site.com/binary"),
            RiskLevel::High
        );
    }

    #[test]
    fn test_validate_server_type_constraints() {
        let npm_type = ServerType::Npm {
            package: "lodash".to_string(),
            version: None,
        };
        assert!(validate_server_type_constraints(&npm_type).is_ok());

        let invalid_npm_type = ServerType::Npm {
            package: ".invalid".to_string(),
            version: None,
        };
        assert!(validate_server_type_constraints(&invalid_npm_type).is_err());
    }
}
