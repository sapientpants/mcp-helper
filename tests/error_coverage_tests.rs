//! Comprehensive coverage tests for error.rs module
//!
//! This test suite ensures all error types, constructors, display formatting,
//! and conversion traits are fully covered.

use mcp_helper::deps::{InstallInstructions, InstallMethod};
use mcp_helper::error::{ErrorBuilder, McpError, Result};
use std::error::Error;
use std::io;

/// Helper to strip ANSI color codes for testing display output
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1B' && chars.peek() == Some(&'[') {
            chars.next();
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

#[test]
fn test_missing_dependency_constructor() {
    // Test with version
    let error1 = McpError::missing_dependency(
        "Node.js",
        Some("18.0.0".to_string()),
        InstallInstructions::default(),
    );

    match error1 {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Node.js");
            assert_eq!(required_version, Some("18.0.0".to_string()));
        }
        _ => panic!("Wrong error type"),
    }

    // Test without version
    let error2 =
        McpError::missing_dependency(String::from("Python"), None, InstallInstructions::default());

    match error2 {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Python");
            assert_eq!(required_version, None);
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_version_mismatch_constructor() {
    let error = McpError::version_mismatch(
        "Docker",
        "20.10.0",
        "24.0.0",
        InstallInstructions::default(),
    );

    match error {
        McpError::VersionMismatch {
            dependency,
            current_version,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Docker");
            assert_eq!(current_version, "20.10.0");
            assert_eq!(required_version, "24.0.0");
        }
        _ => panic!("Wrong error type"),
    }

    // Test with String types
    let error2 = McpError::version_mismatch(
        String::from("Git"),
        String::from("2.25.0"),
        String::from("2.35.0"),
        InstallInstructions::default(),
    );

    match error2 {
        McpError::VersionMismatch { dependency, .. } => {
            assert_eq!(dependency, "Git");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_configuration_required_constructor() {
    let fields = vec!["api_key".to_string(), "region".to_string()];
    let descriptions = vec![
        (
            "api_key".to_string(),
            "API key for authentication".to_string(),
        ),
        ("region".to_string(), "AWS region".to_string()),
    ];

    let error =
        McpError::configuration_required("aws-server", fields.clone(), descriptions.clone());

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "aws-server");
            assert_eq!(missing_fields, fields);
            assert_eq!(field_descriptions, descriptions);
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_client_not_found_constructor() {
    let available = vec!["Claude Desktop".to_string(), "VS Code".to_string()];

    let error = McpError::client_not_found(
        "Cursor",
        available.clone(),
        "Install Cursor from https://cursor.sh",
    );

    match error {
        McpError::ClientNotFound {
            client_name,
            available_clients,
            install_guidance,
        } => {
            assert_eq!(client_name, "Cursor");
            assert_eq!(available_clients, available);
            assert_eq!(install_guidance, "Install Cursor from https://cursor.sh");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_config_error_constructor() {
    let error = McpError::config_error(
        "/home/user/.config/mcp/config.json",
        "Invalid JSON: expected '}' at line 10",
    );

    match error {
        McpError::ConfigError { path, message } => {
            assert_eq!(path, "/home/user/.config/mcp/config.json");
            assert_eq!(message, "Invalid JSON: expected '}' at line 10");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_server_error_constructor() {
    let error = McpError::server_error(
        "@modelcontextprotocol/server-filesystem",
        "Failed to download: 404 Not Found",
    );

    match error {
        McpError::ServerError {
            server_name,
            message,
        } => {
            assert_eq!(server_name, "@modelcontextprotocol/server-filesystem");
            assert_eq!(message, "Failed to download: 404 Not Found");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_io_error_constructor() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");

    // With path
    let error1 = McpError::io_error(
        "reading configuration",
        Some("/etc/mcp/config.json".to_string()),
        io_err,
    );

    match error1 {
        McpError::IoError {
            operation,
            path,
            source,
        } => {
            assert_eq!(operation, "reading configuration");
            assert_eq!(path, Some("/etc/mcp/config.json".to_string()));
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Wrong error type"),
    }

    // Without path
    let io_err2 = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let error2 = McpError::io_error("creating temporary file", None, io_err2);

    match error2 {
        McpError::IoError { path, .. } => {
            assert_eq!(path, None);
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_display_missing_dependency() {
    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "winget".to_string(),
            command: "winget install nodejs".to_string(),
            description: Some("Windows Package Manager".to_string()),
        }],
        macos: vec![InstallMethod {
            name: "homebrew".to_string(),
            command: "brew install node".to_string(),
            description: Some("Homebrew package manager".to_string()),
        }],
        linux: vec![InstallMethod {
            name: "apt".to_string(),
            command: "sudo apt install nodejs".to_string(),
            description: Some("Debian/Ubuntu".to_string()),
        }],
    };

    // With version
    let error1 =
        McpError::missing_dependency("Node.js", Some("18.0.0".to_string()), instructions.clone());

    let output1 = strip_ansi_codes(&format!("{error1}"));
    assert!(output1.contains("Missing dependency: Node.js"));
    assert!(output1.contains("Required version: 18.0.0"));
    assert!(output1.contains("How to install:"));

    // Without version
    let error2 = McpError::missing_dependency("Python", None, instructions);

    let output2 = strip_ansi_codes(&format!("{error2}"));
    assert!(output2.contains("Missing dependency: Python"));
    assert!(!output2.contains("Required version:"));
}

#[test]
fn test_display_version_mismatch() {
    let instructions = InstallInstructions {
        windows: vec![],
        macos: vec![InstallMethod {
            name: "homebrew".to_string(),
            command: "brew upgrade docker".to_string(),
            description: None,
        }],
        linux: vec![],
    };

    let error = McpError::version_mismatch("Docker", "20.10.0", "24.0.0", instructions);

    let output = strip_ansi_codes(&format!("{error}"));
    assert!(output.contains("Version mismatch for: Docker"));
    assert!(output.contains("Current version: 20.10.0"));
    assert!(output.contains("Required version: 24.0.0"));
    assert!(output.contains("How to upgrade:"));
}

#[test]
fn test_display_configuration_required() {
    // With descriptions
    let error1 = McpError::configuration_required(
        "api-server",
        vec!["api_key".to_string(), "region".to_string()],
        vec![
            ("api_key".to_string(), "Your API key".to_string()),
            ("region".to_string(), "Server region".to_string()),
        ],
    );

    let output1 = strip_ansi_codes(&format!("{error1}"));
    assert!(output1.contains("Configuration required for: api-server"));
    assert!(output1.contains("Missing fields:"));
    assert!(output1.contains("api_key"));
    assert!(output1.contains("region"));
    assert!(output1.contains("Field descriptions:"));
    assert!(output1.contains("Your API key"));
    assert!(output1.contains("Server region"));

    // Without descriptions
    let error2 =
        McpError::configuration_required("simple-server", vec!["port".to_string()], vec![]);

    let output2 = strip_ansi_codes(&format!("{error2}"));
    assert!(output2.contains("port"));
    assert!(!output2.contains("Field descriptions:"));
}

#[test]
fn test_display_client_not_found() {
    // With available clients
    let error1 = McpError::client_not_found(
        "Cursor",
        vec!["Claude Desktop".to_string(), "VS Code".to_string()],
        "Visit https://cursor.sh to install",
    );

    let output1 = strip_ansi_codes(&format!("{error1}"));
    assert!(output1.contains("MCP client not found: Cursor"));
    assert!(output1.contains("Available clients:"));
    assert!(output1.contains("Claude Desktop"));
    assert!(output1.contains("VS Code"));
    assert!(output1.contains("Installation guidance:"));
    assert!(output1.contains("Visit https://cursor.sh to install"));

    // Without available clients
    let error2 = McpError::client_not_found("Unknown", vec![], "No MCP clients detected");

    let output2 = strip_ansi_codes(&format!("{error2}"));
    assert!(!output2.contains("Available clients:"));
}

#[test]
fn test_display_config_error() {
    let error = McpError::config_error("/home/user/.mcp/config.json", "Unexpected token at line 5");

    let output = strip_ansi_codes(&format!("{error}"));
    assert!(output.contains("Configuration error"));
    assert!(output.contains("Path: /home/user/.mcp/config.json"));
    assert!(output.contains("Error: Unexpected token at line 5"));
}

#[test]
fn test_display_server_error() {
    let error = McpError::server_error("custom-server", "Connection timeout after 30 seconds");

    let output = strip_ansi_codes(&format!("{error}"));
    assert!(output.contains("Server error: custom-server"));
    assert!(output.contains("Connection timeout after 30 seconds"));
}

#[test]
fn test_display_io_error() {
    // With path
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
    let error1 = McpError::io_error(
        "writing config",
        Some("/etc/config.json".to_string()),
        io_err,
    );

    let output1 = strip_ansi_codes(&format!("{error1}"));
    assert!(output1.contains("I/O error during: writing config"));
    assert!(output1.contains("Path: /etc/config.json"));
    assert!(output1.contains("Error:"));

    // Without path
    let io_err2 = io::Error::new(io::ErrorKind::NotFound, "not found");
    let error2 = McpError::io_error("reading", None, io_err2);

    let output2 = strip_ansi_codes(&format!("{error2}"));
    assert!(!output2.contains("Path:"));
}

#[test]
fn test_display_other_error() {
    let anyhow_err = anyhow::anyhow!("Something went wrong");
    let error = McpError::Other(anyhow_err);

    let output = strip_ansi_codes(&format!("{error}"));
    assert!(output.contains("Something went wrong"));
}

#[test]
fn test_error_trait_source() {
    // IoError has source
    let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
    let error1 = McpError::io_error("test", None, io_err);
    assert!(error1.source().is_some());

    // Other has source
    let anyhow_err = anyhow::anyhow!("test");
    let error2 = McpError::Other(anyhow_err);
    assert!(error2.source().is_some());

    // Others have no source
    let error3 = McpError::missing_dependency("test", None, InstallInstructions::default());
    assert!(error3.source().is_none());

    let error4 = McpError::config_error("path", "msg");
    assert!(error4.source().is_none());
}

#[test]
fn test_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let mcp_err: McpError = io_err.into();

    match mcp_err {
        McpError::IoError {
            operation,
            path,
            source,
        } => {
            assert_eq!(operation, "unknown");
            assert_eq!(path, None);
            assert_eq!(source.kind(), io::ErrorKind::PermissionDenied);
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_from_anyhow_error() {
    let anyhow_err = anyhow::anyhow!("Custom error");
    let mcp_err: McpError = anyhow_err.into();

    match mcp_err {
        McpError::Other(err) => {
            assert_eq!(err.to_string(), "Custom error");
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_from_dialoguer_error() {
    // We can't easily create a real dialoguer::Error, so we test the conversion path
    // This is covered by the From implementation
    let dialoguer_err = dialoguer::Error::from(io::Error::other("test"));
    let mcp_err: McpError = dialoguer_err.into();

    match mcp_err {
        McpError::Other(err) => {
            assert!(err.to_string().contains("Dialog error"));
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_result_type_alias() {
    // Test that Result<T> works as expected
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }

    fn returns_err() -> Result<()> {
        Err(McpError::config_error("test", "error"))
    }

    assert!(returns_ok().is_ok());
    assert!(returns_err().is_err());
}

#[test]
fn test_platform_specific_install_instructions() {
    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "Windows Method".to_string(),
            command: "windows command".to_string(),
            description: Some("Windows only".to_string()),
        }],
        macos: vec![InstallMethod {
            name: "macOS Method".to_string(),
            command: "macos command".to_string(),
            description: Some("macOS only".to_string()),
        }],
        linux: vec![InstallMethod {
            name: "Linux Method".to_string(),
            command: "linux command".to_string(),
            description: Some("Linux only".to_string()),
        }],
    };

    let error = McpError::missing_dependency("Test", None, instructions);
    let output = strip_ansi_codes(&format!("{error}"));

    // Verify platform-specific content appears
    #[cfg(target_os = "windows")]
    {
        assert!(output.contains("Windows Method"));
        assert!(output.contains("windows command"));
    }

    #[cfg(target_os = "macos")]
    {
        assert!(output.contains("macOS Method"));
        assert!(output.contains("macos command"));
    }

    #[cfg(target_os = "linux")]
    {
        assert!(output.contains("Linux Method"));
        assert!(output.contains("linux command"));
    }
}

#[test]
fn test_empty_install_instructions() {
    let empty_instructions = InstallInstructions::default();

    let error = McpError::missing_dependency("Tool", None, empty_instructions);
    let output = format!("{error}");

    // Should not panic even with empty instructions
    assert!(output.contains("Tool"));
}

#[test]
fn test_install_method_without_description() {
    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "Method".to_string(),
            command: "command".to_string(),
            description: None, // No description
        }],
        macos: vec![],
        linux: vec![],
    };

    let error = McpError::version_mismatch("App", "1.0", "2.0", instructions);
    let output = strip_ansi_codes(&format!("{error}"));

    // Should not show "Note:" line when no description
    assert!(!output.contains("Note:"));
}

#[test]
fn test_unicode_in_error_messages() {
    let error1 = McpError::server_error("测试服务器", "错误信息 with émojis 🚀");

    let output1 = format!("{error1}");
    assert!(output1.contains("测试服务器"));
    assert!(output1.contains("🚀"));

    let error2 = McpError::configuration_required(
        "сервер",
        vec!["ключ".to_string()],
        vec![("ключ".to_string(), "описание".to_string())],
    );

    let output2 = format!("{error2}");
    assert!(output2.contains("сервер"));
    assert!(output2.contains("ключ"));
}

#[test]
fn test_very_long_error_messages() {
    let long_path = "a".repeat(500);
    let long_message = "b".repeat(1000);

    let error = McpError::config_error(&long_path, &long_message);
    let output = format!("{error}");

    // Should handle long strings without panic
    assert!(output.len() > 1000);
}

#[test]
fn test_error_builder_integration() {
    // Test that ErrorBuilder creates proper McpError instances
    let error1 = ErrorBuilder::missing_dependency("Node.js")
        .version("18.0.0")
        .build();

    match error1 {
        McpError::MissingDependency { dependency, .. } => {
            assert_eq!(dependency, "Node.js");
        }
        _ => panic!("Wrong error type"),
    }

    let error2 = ErrorBuilder::config_required("server")
        .field("key", "description")
        .build();

    match error2 {
        McpError::ConfigurationRequired { server_name, .. } => {
            assert_eq!(server_name, "server");
        }
        _ => panic!("Wrong error type"),
    }
}
