use mcp_helper::deps::InstallInstructions;
use mcp_helper::error::{McpError, Result};

#[path = "common/mod.rs"]
mod common;
use common::assert_error_contains;

#[test]
fn test_missing_dependency_error_display() {
    let instructions = InstallInstructions {
        windows: vec![],
        macos: vec![mcp_helper::deps::InstallMethod {
            name: "Homebrew".to_string(),
            command: "brew install node".to_string(),
            description: Some("Install using Homebrew package manager".to_string()),
        }],
        linux: vec![],
    };

    let error = McpError::missing_dependency("Node.js", Some("v20.0.0".to_string()), instructions);
    let error_string = format!("{error}");

    // Check that the error contains expected content
    assert_error_contains(
        &error_string,
        &[
            "Missing dependency: Node.js",
            "Required version: v20.0.0",
            "How to install:",
        ],
    );
    #[cfg(target_os = "macos")]
    {
        assert_error_contains(&error_string, &["Homebrew", "brew install node"]);
    }
}

#[test]
fn test_version_mismatch_error_display() {
    let instructions = InstallInstructions {
        windows: vec![],
        macos: vec![mcp_helper::deps::InstallMethod {
            name: "Homebrew".to_string(),
            command: "brew upgrade node".to_string(),
            description: None,
        }],
        linux: vec![],
    };

    let error = McpError::version_mismatch("Node.js", "v18.0.0", "v20.0.0", instructions);
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "Version mismatch for: Node.js",
            "Current version: v18.0.0",
            "Required version: v20.0.0",
            "How to upgrade:",
        ],
    );
}

#[test]
fn test_configuration_required_error_display() {
    let error = McpError::configuration_required(
        "slack-server",
        vec!["api_token".to_string(), "workspace_id".to_string()],
        vec![
            ("api_token".to_string(), "Your Slack API token".to_string()),
            (
                "workspace_id".to_string(),
                "Your Slack workspace ID".to_string(),
            ),
        ],
    );
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "Configuration required for: slack-server",
            "Missing fields:",
            "api_token",
            "workspace_id",
            "Field descriptions:",
            "Your Slack API token",
        ],
    );
}

#[test]
fn test_client_not_found_error_display() {
    let error = McpError::client_not_found(
        "Unknown Client",
        vec!["Claude Desktop".to_string(), "Cursor".to_string()],
        "Visit https://claude.ai to download Claude Desktop",
    );
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "MCP client not found: Unknown Client",
            "Available clients:",
            "Claude Desktop",
            "Cursor",
            "Installation guidance:",
            "Visit https://claude.ai",
        ],
    );
}

#[test]
fn test_config_error_display() {
    let error = McpError::config_error(
        "/path/to/config.json",
        "Invalid JSON: expected value at line 3 column 5",
    );
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "Configuration error",
            "Path: /path/to/config.json",
            "Error: Invalid JSON",
        ],
    );
}

#[test]
fn test_server_error_display() {
    let error = McpError::server_error(
        "filesystem-server",
        "Failed to start server: permission denied",
    );
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "Server error: filesystem-server",
            "Failed to start server: permission denied",
        ],
    );
}

#[test]
fn test_io_error_display() {
    use std::io;

    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error = McpError::io_error(
        "reading config",
        Some("/path/to/file".to_string()),
        io_error,
    );
    let error_string = format!("{error}");

    assert_error_contains(
        &error_string,
        &[
            "I/O error during: reading config",
            "Path: /path/to/file",
            "file not found",
        ],
    );
}

#[test]
fn test_error_conversions() {
    // Test From<std::io::Error>
    let io_error = std::io::Error::other("test error");
    let mcp_error: McpError = io_error.into();
    match mcp_error {
        McpError::IoError { .. } => {}
        _ => panic!("Expected IoError variant"),
    }

    // Test From<anyhow::Error>
    let anyhow_error = anyhow::anyhow!("test error");
    let mcp_error: McpError = anyhow_error.into();
    match mcp_error {
        McpError::Other(_) => {}
        _ => panic!("Expected Other variant"),
    }
}

#[test]
fn test_result_type() {
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }

    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");

    fn returns_error() -> Result<String> {
        Err(McpError::server_error("test", "error"))
    }

    let result = returns_error();
    assert!(result.is_err());
}
