//! Test fixtures and data builders for consistent test data
//!
//! This module provides builders and helpers for creating test data
//! that is used across multiple test modules.

use crate::client::ServerConfig;
use crate::deps::{InstallInstructions, InstallMethod};
use std::collections::HashMap;

/// Creates a sample server configuration for testing
pub fn sample_server_config() -> ServerConfig {
    ServerConfig {
        command: "npx".to_string(),
        args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
        env: HashMap::from([(
            "MCP_ALLOWED_PATHS".to_string(),
            "/tmp,/home/user".to_string(),
        )]),
    }
}

/// Creates a minimal server configuration
pub fn minimal_server_config() -> ServerConfig {
    ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    }
}

/// Creates sample install instructions for testing
pub fn sample_install_instructions() -> InstallInstructions {
    InstallInstructions {
        windows: vec![
            InstallMethod {
                name: "winget".to_string(),
                command: "winget install Node.js".to_string(),
                description: Some("Windows Package Manager".to_string()),
            },
            InstallMethod {
                name: "download".to_string(),
                command: "https://nodejs.org/download".to_string(),
                description: Some("Direct download".to_string()),
            },
        ],
        macos: vec![InstallMethod {
            name: "brew".to_string(),
            command: "brew install node".to_string(),
            description: Some("Homebrew package manager".to_string()),
        }],
        linux: vec![
            InstallMethod {
                name: "apt".to_string(),
                command: "sudo apt install nodejs".to_string(),
                description: Some("Debian/Ubuntu".to_string()),
            },
            InstallMethod {
                name: "dnf".to_string(),
                command: "sudo dnf install nodejs".to_string(),
                description: Some("Fedora/RHEL".to_string()),
            },
        ],
    }
}

/// Creates a HashMap of multiple server configurations
pub fn multiple_server_configs() -> HashMap<String, ServerConfig> {
    let mut configs = HashMap::new();

    configs.insert(
        "filesystem".to_string(),
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
            env: HashMap::from([(
                "MCP_ALLOWED_PATHS".to_string(),
                "/home/user/documents".to_string(),
            )]),
        },
    );

    configs.insert(
        "github".to_string(),
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server-github".to_string()],
            env: HashMap::from([("GITHUB_TOKEN".to_string(), "ghp_test_token".to_string())]),
        },
    );

    configs.insert(
        "custom".to_string(),
        ServerConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "custom_server".to_string()],
            env: HashMap::new(),
        },
    );

    configs
}

/// Configuration field descriptions for testing
pub fn sample_field_descriptions() -> Vec<(String, String)> {
    vec![
        (
            "api_key".to_string(),
            "API key for authentication".to_string(),
        ),
        ("base_url".to_string(), "Base URL for the API".to_string()),
        (
            "timeout".to_string(),
            "Request timeout in seconds".to_string(),
        ),
    ]
}

/// Creates test environment variables
pub fn test_env_vars() -> HashMap<String, String> {
    HashMap::from([
        ("TEST_VAR_1".to_string(), "value1".to_string()),
        ("TEST_VAR_2".to_string(), "value2".to_string()),
        ("PATH".to_string(), "/usr/bin:/usr/local/bin".to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_server_config() {
        let config = sample_server_config();
        assert_eq!(config.command, "npx");
        assert_eq!(config.args.len(), 1);
        assert!(!config.env.is_empty());
    }

    #[test]
    fn test_minimal_server_config() {
        let config = minimal_server_config();
        assert_eq!(config.command, "node");
        assert!(config.env.is_empty());
    }

    #[test]
    fn test_sample_install_instructions() {
        let instructions = sample_install_instructions();
        assert!(!instructions.windows.is_empty());
        assert!(!instructions.macos.is_empty());
        assert!(!instructions.linux.is_empty());
    }

    #[test]
    fn test_multiple_server_configs() {
        let configs = multiple_server_configs();
        assert_eq!(configs.len(), 3);
        assert!(configs.contains_key("filesystem"));
        assert!(configs.contains_key("github"));
        assert!(configs.contains_key("custom"));
    }
}
