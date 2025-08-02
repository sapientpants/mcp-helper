//! Comprehensive tests for the error handling system in error.rs

use mcp_helper::deps::{InstallInstructions, InstallMethod};
use mcp_helper::error::{McpError, Result};
use std::error::Error;
use std::io;

/// Test helper to strip ANSI color codes from output for easier assertion
fn strip_ansi_codes(s: &str) -> String {
    // Simple ANSI code removal without regex dependency
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' && chars.peek() == Some(&'[') {
            // Skip escape sequence
            chars.next(); // Skip '['
            for ch in chars.by_ref() {
                if ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Test helper to create sample install instructions
fn sample_install_instructions() -> InstallInstructions {
    InstallInstructions {
        windows: vec![InstallMethod {
            name: "winget".to_string(),
            command: "winget install OpenJS.NodeJS".to_string(),
            description: Some("Windows Package Manager".to_string()),
        }],
        macos: vec![InstallMethod {
            name: "homebrew".to_string(),
            command: "brew install node".to_string(),
            description: Some("Homebrew package manager".to_string()),
        }],
        linux: vec![InstallMethod {
            name: "apt".to_string(),
            command: "sudo apt-get install nodejs".to_string(),
            description: Some("Debian/Ubuntu package manager".to_string()),
        }],
    }
}

#[test]
fn test_missing_dependency_creation() {
    let error = McpError::missing_dependency(
        "Node.js",
        Some("18.0.0".to_string()),
        sample_install_instructions(),
    );

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Node.js");
            assert_eq!(required_version, Some("18.0.0".to_string()));
        }
        _ => panic!("Expected MissingDependency variant"),
    }
}

#[test]
fn test_missing_dependency_without_version() {
    let error = McpError::missing_dependency("Docker", None, sample_install_instructions());

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Docker");
            assert_eq!(required_version, None);
        }
        _ => panic!("Expected MissingDependency variant"),
    }
}

#[test]
fn test_version_mismatch_creation() {
    let error =
        McpError::version_mismatch("Python", "3.8.0", "3.10.0", sample_install_instructions());

    match error {
        McpError::VersionMismatch {
            dependency,
            current_version,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Python");
            assert_eq!(current_version, "3.8.0");
            assert_eq!(required_version, "3.10.0");
        }
        _ => panic!("Expected VersionMismatch variant"),
    }
}

#[test]
fn test_configuration_required_creation() {
    let error = McpError::configuration_required(
        "slack-server",
        vec!["api_token".to_string(), "workspace_id".to_string()],
        vec![
            ("api_token".to_string(), "Slack API token".to_string()),
            ("workspace_id".to_string(), "Slack workspace ID".to_string()),
        ],
    );

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "slack-server");
            assert_eq!(missing_fields.len(), 2);
            assert_eq!(missing_fields[0], "api_token");
            assert_eq!(missing_fields[1], "workspace_id");
            assert_eq!(field_descriptions.len(), 2);
            assert_eq!(field_descriptions[0].0, "api_token");
            assert_eq!(field_descriptions[0].1, "Slack API token");
        }
        _ => panic!("Expected ConfigurationRequired variant"),
    }
}

#[test]
fn test_client_not_found_creation() {
    let error = McpError::client_not_found(
        "VSCode",
        vec!["Claude Desktop".to_string(), "Cursor".to_string()],
        "Install VSCode from https://code.visualstudio.com",
    );

    match error {
        McpError::ClientNotFound {
            client_name,
            available_clients,
            install_guidance,
        } => {
            assert_eq!(client_name, "VSCode");
            assert_eq!(available_clients.len(), 2);
            assert_eq!(available_clients[0], "Claude Desktop");
            assert!(install_guidance.contains("code.visualstudio.com"));
        }
        _ => panic!("Expected ClientNotFound variant"),
    }
}

#[test]
fn test_config_error_creation() {
    let error = McpError::config_error(
        "/path/to/config.json",
        "Invalid JSON: expected value at line 5",
    );

    match error {
        McpError::ConfigError { path, message } => {
            assert_eq!(path, "/path/to/config.json");
            assert_eq!(message, "Invalid JSON: expected value at line 5");
        }
        _ => panic!("Expected ConfigError variant"),
    }
}

#[test]
fn test_server_error_creation() {
    let error = McpError::server_error(
        "filesystem-server",
        "Failed to bind to port 3000: address already in use",
    );

    match error {
        McpError::ServerError {
            server_name,
            message,
        } => {
            assert_eq!(server_name, "filesystem-server");
            assert!(message.contains("port 3000"));
        }
        _ => panic!("Expected ServerError variant"),
    }
}

#[test]
fn test_io_error_creation() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let error = McpError::io_error(
        "reading configuration",
        Some("/etc/mcp/config.json".to_string()),
        io_err,
    );

    match error {
        McpError::IoError {
            operation,
            path,
            source,
        } => {
            assert_eq!(operation, "reading configuration");
            assert_eq!(path, Some("/etc/mcp/config.json".to_string()));
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_io_error_without_path() {
    let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "Unexpected EOF");
    let error = McpError::io_error("network operation", None, io_err);

    match error {
        McpError::IoError {
            operation,
            path,
            source,
        } => {
            assert_eq!(operation, "network operation");
            assert_eq!(path, None);
            assert_eq!(source.kind(), io::ErrorKind::UnexpectedEof);
        }
        _ => panic!("Expected IoError variant"),
    }
}

#[test]
fn test_display_missing_dependency() {
    let error = McpError::missing_dependency(
        "Node.js",
        Some("18.0.0".to_string()),
        sample_install_instructions(),
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Missing dependency: Node.js"));
    assert!(stripped.contains("Required version: 18.0.0"));
    assert!(stripped.contains("How to install:"));

    // Platform-specific check
    #[cfg(target_os = "windows")]
    assert!(stripped.contains("winget install"));
    #[cfg(target_os = "macos")]
    assert!(stripped.contains("brew install"));
    #[cfg(target_os = "linux")]
    assert!(stripped.contains("apt-get install"));
}

#[test]
fn test_display_version_mismatch() {
    let error =
        McpError::version_mismatch("Python", "3.8.0", "3.10.0", sample_install_instructions());

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Version mismatch for: Python"));
    assert!(stripped.contains("Current version: 3.8.0"));
    assert!(stripped.contains("Required version: 3.10.0"));
    assert!(stripped.contains("How to upgrade:"));
}

#[test]
fn test_display_configuration_required() {
    let error = McpError::configuration_required(
        "slack-server",
        vec!["api_token".to_string(), "workspace_id".to_string()],
        vec![
            ("api_token".to_string(), "Slack API token".to_string()),
            ("workspace_id".to_string(), "Slack workspace ID".to_string()),
        ],
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Configuration required for: slack-server"));
    assert!(stripped.contains("Missing fields:"));
    assert!(stripped.contains("api_token"));
    assert!(stripped.contains("workspace_id"));
    assert!(stripped.contains("Field descriptions:"));
    assert!(stripped.contains("Slack API token"));
    assert!(stripped.contains("Slack workspace ID"));
}

#[test]
fn test_display_client_not_found() {
    let error = McpError::client_not_found(
        "VSCode",
        vec!["Claude Desktop".to_string(), "Cursor".to_string()],
        "Install VSCode from https://code.visualstudio.com",
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("MCP client not found: VSCode"));
    assert!(stripped.contains("Available clients:"));
    assert!(stripped.contains("Claude Desktop"));
    assert!(stripped.contains("Cursor"));
    assert!(stripped.contains("Installation guidance:"));
    assert!(stripped.contains("https://code.visualstudio.com"));
}

#[test]
fn test_display_config_error() {
    let error = McpError::config_error(
        "/path/to/config.json",
        "Invalid JSON: expected value at line 5",
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Configuration error"));
    assert!(stripped.contains("Path: /path/to/config.json"));
    assert!(stripped.contains("Error: Invalid JSON"));
}

#[test]
fn test_display_server_error() {
    let error = McpError::server_error(
        "filesystem-server",
        "Failed to bind to port 3000: address already in use",
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Server error: filesystem-server"));
    assert!(stripped.contains("Failed to bind to port 3000"));
}

#[test]
fn test_display_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let error = McpError::io_error(
        "reading configuration",
        Some("/etc/mcp/config.json".to_string()),
        io_err,
    );

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("I/O error during: reading configuration"));
    assert!(stripped.contains("Path: /etc/mcp/config.json"));
    assert!(stripped.contains("Access denied"));
}

#[test]
fn test_display_other_error() {
    let anyhow_err = anyhow::anyhow!("Custom error message");
    let error = McpError::Other(anyhow_err);

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Custom error message"));
}

#[test]
fn test_error_source_chain() {
    // Test IoError source
    let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = McpError::io_error("opening file", None, io_err);

    assert!(error.source().is_some());
    let source = error.source().unwrap();
    assert_eq!(source.to_string(), "File not found");

    // Test Other error source
    let anyhow_err = anyhow::anyhow!("Database connection failed");
    let error = McpError::Other(anyhow_err);

    assert!(error.source().is_some());
    let source = error.source().unwrap();
    assert_eq!(source.to_string(), "Database connection failed");

    // Test variants without source
    let error = McpError::missing_dependency("Test", None, InstallInstructions::default());
    assert!(error.source().is_none());
}

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::AlreadyExists, "File already exists");
    let error: McpError = io_err.into();

    match error {
        McpError::IoError {
            operation,
            path,
            source,
        } => {
            assert_eq!(operation, "unknown");
            assert_eq!(path, None);
            assert_eq!(source.kind(), io::ErrorKind::AlreadyExists);
        }
        _ => panic!("Expected IoError variant from io::Error conversion"),
    }
}

#[test]
fn test_from_anyhow_error() {
    let anyhow_err = anyhow::anyhow!("Generic error");
    let error: McpError = anyhow_err.into();

    match error {
        McpError::Other(err) => {
            assert_eq!(err.to_string(), "Generic error");
        }
        _ => panic!("Expected Other variant from anyhow::Error conversion"),
    }
}

#[test]
fn test_from_dialoguer_error() {
    use std::io;
    // Create a dialoguer error through io error
    let io_err = io::Error::new(io::ErrorKind::Interrupted, "User cancelled");
    let dialog_err = dialoguer::Error::IO(io_err);
    let error: McpError = dialog_err.into();

    match error {
        McpError::Other(err) => {
            assert!(err.to_string().contains("Dialog error"));
        }
        _ => panic!("Expected Other variant from dialoguer::Error conversion"),
    }
}

#[test]
fn test_result_type_alias() {
    fn sample_function() -> Result<String> {
        Ok("Success".to_string())
    }

    fn failing_function() -> Result<String> {
        Err(McpError::server_error("test", "Failed"))
    }

    assert!(sample_function().is_ok());
    assert_eq!(sample_function().unwrap(), "Success");

    assert!(failing_function().is_err());
    match failing_function().unwrap_err() {
        McpError::ServerError { server_name, .. } => {
            assert_eq!(server_name, "test");
        }
        _ => panic!("Expected ServerError"),
    }
}

#[test]
fn test_colored_output_contains_ansi() {
    // Force colored output
    colored::control::set_override(true);

    let error = McpError::missing_dependency("Test", None, InstallInstructions::default());
    let display = format!("{error}");

    // Should contain ANSI escape codes
    assert!(display.contains('\x1B'));

    // Reset
    colored::control::unset_override();
}

#[test]
fn test_empty_configuration_fields() {
    let error = McpError::configuration_required("test-server", vec![], vec![]);

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("Configuration required for: test-server"));
    assert!(stripped.contains("Missing fields:"));
}

#[test]
fn test_client_not_found_empty_available() {
    let error = McpError::client_not_found("Test Client", vec![], "No clients available");

    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    assert!(stripped.contains("MCP client not found: Test Client"));
    assert!(stripped.contains("Installation guidance:"));
    assert!(stripped.contains("No clients available"));
}

#[test]
fn test_complex_install_instructions() {
    let instructions = InstallInstructions {
        windows: vec![
            InstallMethod {
                name: "winget".to_string(),
                command: "winget install OpenJS.NodeJS".to_string(),
                description: Some("Recommended method".to_string()),
            },
            InstallMethod {
                name: "chocolatey".to_string(),
                command: "choco install nodejs".to_string(),
                description: Some("Alternative method".to_string()),
            },
        ],
        macos: vec![
            InstallMethod {
                name: "homebrew".to_string(),
                command: "brew install node".to_string(),
                description: None,
            },
            InstallMethod {
                name: "macports".to_string(),
                command: "sudo port install nodejs18".to_string(),
                description: Some("MacPorts alternative".to_string()),
            },
        ],
        linux: vec![InstallMethod {
            name: "apt".to_string(),
            command: "sudo apt-get install nodejs npm".to_string(),
            description: Some("Debian/Ubuntu".to_string()),
        }],
    };

    let error = McpError::missing_dependency("Node.js", None, instructions);
    let display = format!("{error}");
    let stripped = strip_ansi_codes(&display);

    // Verify multiple methods are shown for current platform
    #[cfg(target_os = "windows")]
    {
        assert!(stripped.contains("winget"));
        assert!(stripped.contains("chocolatey"));
        assert!(stripped.contains("Recommended method"));
        assert!(stripped.contains("Alternative method"));
    }
    #[cfg(target_os = "macos")]
    {
        assert!(stripped.contains("homebrew"));
        assert!(stripped.contains("macports"));
        assert!(stripped.contains("MacPorts alternative"));
    }
    #[cfg(target_os = "linux")]
    {
        // On Linux, we should see package manager instructions
        assert!(!stripped.is_empty());
        assert!(stripped.contains("Node.js"));
    }
}
