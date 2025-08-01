//! Tests for panic conditions and error conversion in src/error.rs
//!
//! This test suite ensures that error types handle all edge cases properly
//! and that error conversions don't panic.

use mcp_helper::deps::InstallInstructions;
use mcp_helper::error::{McpError, Result};
use std::io;

/// Test error display formatting for all error variants
#[test]
fn test_error_display_all_variants() {
    // Test MissingDependency
    let error = McpError::missing_dependency(
        "Node.js",
        Some("18.0.0".to_string()),
        InstallInstructions::default(),
    );
    let display = format!("{error}");
    assert!(display.contains("Missing dependency"));
    assert!(display.contains("Node.js"));
    assert!(display.contains("18.0.0"));

    // Test with empty dependency name
    let error = McpError::missing_dependency("", None, InstallInstructions::default());
    let display = format!("{error}");
    assert!(!display.is_empty());

    // Test VersionMismatch
    let error =
        McpError::version_mismatch("Python", "3.8.0", "3.10.0", InstallInstructions::default());
    let display = format!("{error}");
    assert!(display.contains("Version mismatch"));
    assert!(display.contains("3.8.0"));
    assert!(display.contains("3.10.0"));

    // Test ConfigurationRequired
    let error = McpError::configuration_required(
        "test-server",
        vec!["api_key".to_string(), "secret".to_string()],
        vec![
            ("api_key".to_string(), "Your API key".to_string()),
            ("secret".to_string(), "Your secret token".to_string()),
        ],
    );
    let display = format!("{error}");
    assert!(display.contains("Configuration required"));
    assert!(display.contains("api_key"));
    assert!(display.contains("Your API key"));

    // Test with empty fields
    let error = McpError::configuration_required("", vec![], vec![]);
    let display = format!("{error}");
    assert!(!display.is_empty());

    // Test ClientNotFound
    let error = McpError::client_not_found(
        "Unknown Client",
        vec!["Claude Desktop".to_string(), "VS Code".to_string()],
        "Please install one of the available clients",
    );
    let display = format!("{error}");
    assert!(display.contains("MCP client not found"));
    assert!(display.contains("Claude Desktop"));
    assert!(display.contains("VS Code"));

    // Test with no available clients
    let error = McpError::client_not_found("test", vec![], "");
    let display = format!("{error}");
    assert!(!display.is_empty());

    // Test ConfigError
    let error = McpError::config_error("/path/to/config.json", "Invalid JSON syntax");
    let display = format!("{error}");
    assert!(display.contains("Configuration error"));
    assert!(display.contains("/path/to/config.json"));
    assert!(display.contains("Invalid JSON"));

    // Test ServerError
    let error = McpError::server_error("npm-server", "Failed to start server");
    let display = format!("{error}");
    assert!(display.contains("Server error"));
    assert!(display.contains("npm-server"));
    assert!(display.contains("Failed to start"));

    // Test IoError
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let error = McpError::io_error("reading file", Some("/etc/passwd".to_string()), io_error);
    let display = format!("{error}");
    assert!(display.contains("I/O error"));
    assert!(display.contains("reading file"));
    assert!(display.contains("/etc/passwd"));

    // Test IoError without path
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let error = McpError::io_error("operation", None, io_error);
    let display = format!("{error}");
    assert!(display.contains("I/O error"));

    // Test Other error
    let error = McpError::Other(anyhow::anyhow!("Custom error message"));
    let display = format!("{error}");
    assert!(display.contains("Custom error message"));
}

/// Test error source chain
#[test]
fn test_error_source_chain() {
    use std::error::Error;

    // Test IoError source
    let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
    let error = McpError::io_error("test", None, io_error);
    assert!(error.source().is_some());
    let source = error.source().unwrap();
    assert_eq!(source.to_string(), "Access denied");

    // Test Other error source
    let inner_error = anyhow::anyhow!("Inner error");
    let error = McpError::Other(inner_error);
    assert!(error.source().is_some());

    // Test variants without source
    let error = McpError::missing_dependency("test", None, InstallInstructions::default());
    assert!(error.source().is_none());

    let error = McpError::config_error("path", "message");
    assert!(error.source().is_none());
}

/// Test error conversions
#[test]
fn test_error_conversions() {
    // Test From<io::Error>
    let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
    let mcp_error: McpError = io_error.into();
    match mcp_error {
        McpError::IoError { operation, .. } => {
            assert_eq!(operation, "unknown");
        }
        _ => panic!("Expected IoError variant"),
    }

    // Test From<anyhow::Error>
    let anyhow_error = anyhow::anyhow!("Test error");
    let mcp_error: McpError = anyhow_error.into();
    match mcp_error {
        McpError::Other(e) => {
            assert_eq!(e.to_string(), "Test error");
        }
        _ => panic!("Expected Other variant"),
    }

    // Test From<dialoguer::Error>
    let dialog_error = dialoguer::Error::from(io::Error::new(io::ErrorKind::UnexpectedEof, "EOF"));
    let mcp_error: McpError = dialog_error.into();
    match mcp_error {
        McpError::Other(e) => {
            assert!(e.to_string().contains("Dialog error"));
        }
        _ => panic!("Expected Other variant"),
    }
}

/// Test error formatting edge cases
#[test]
fn test_error_formatting_edge_cases() {
    // Very long strings
    let long_string = "a".repeat(10000);
    let error = McpError::server_error(&long_string, &long_string);
    let display = format!("{error}");
    assert!(display.len() > 1000); // Should not truncate

    // Unicode and special characters
    let error = McpError::config_error(
        "config_🚀.json",
        "Failed to parse: \n\t{\"key\": \"value\"}",
    );
    let display = format!("{error}");
    assert!(display.contains("🚀"));
    assert!(display.contains("\n"));

    // Empty install instructions
    let instructions = InstallInstructions {
        windows: vec![],
        macos: vec![],
        linux: vec![],
    };
    let error = McpError::missing_dependency("test", None, instructions);
    let display = format!("{error}");
    assert!(!display.is_empty());
}

/// Test error debug formatting
#[test]
fn test_error_debug_formatting() {
    let error = McpError::config_error("test.json", "Invalid");
    let debug = format!("{error:?}");
    assert!(debug.contains("ConfigError"));
    assert!(debug.contains("test.json"));
    assert!(debug.contains("Invalid"));

    // Test all variants have proper Debug implementation
    let errors: Vec<McpError> = vec![
        McpError::missing_dependency("test", None, InstallInstructions::default()),
        McpError::version_mismatch("test", "1.0", "2.0", InstallInstructions::default()),
        McpError::configuration_required("test", vec![], vec![]),
        McpError::client_not_found("test", vec![], ""),
        McpError::config_error("test", "error"),
        McpError::server_error("test", "error"),
        McpError::io_error("test", None, io::Error::other("test")),
        McpError::Other(anyhow::anyhow!("test")),
    ];

    for error in errors {
        let debug = format!("{error:?}");
        assert!(!debug.is_empty());
        assert!(!debug.contains("{{")); // No unformatted placeholders
    }
}

/// Test Result type alias usage
#[test]
fn test_result_type_alias() {
    fn returns_ok() -> Result<String> {
        Ok("success".to_string())
    }

    fn returns_err() -> Result<String> {
        Err(McpError::server_error("test", "failed"))
    }

    // Test Ok variant
    let result = returns_ok();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");

    // Test Err variant
    let result = returns_err();
    assert!(result.is_err());
    match result {
        Err(McpError::ServerError { server_name, .. }) => {
            assert_eq!(server_name, "test");
        }
        _ => panic!("Expected ServerError"),
    }
}

/// Test error builder pattern
#[test]
fn test_error_builder_pattern() {
    // Test building errors with builder-like pattern
    let error = McpError::config_error("test_module", "File not found");
    let display = format!("{error}");
    assert!(display.contains("test_module"));
    assert!(display.contains("File not found"));

    // Test chaining error information
    let error = McpError::configuration_required(
        "test-server",
        vec!["config1".to_string()],
        vec![(
            "config1".to_string(),
            "Check if the file exists".to_string(),
        )],
    );
    let display = format!("{error}");
    assert!(display.contains("test-server"));
    assert!(display.contains("config1"));
    assert!(display.contains("Check if the file exists"));
}

/// Test that errors don't panic on edge cases
#[test]
fn test_no_panic_on_edge_cases() {
    // Test with null bytes (should not panic)
    let error = McpError::config_error("config\0.json", "message\0with\0nulls");
    let _ = format!("{error}"); // Should not panic

    // Test with very nested error
    let mut nested_error = McpError::Other(anyhow::anyhow!("Level 0"));
    for i in 1..100 {
        nested_error = McpError::Other(anyhow::anyhow!("Level {}: {:?}", i, nested_error));
    }
    let _ = format!("{nested_error}"); // Should not panic even with deep nesting

    // Test with control characters
    let error = McpError::server_error("server\r\n\t", "message\x00\x01\x02");
    let _ = format!("{error}"); // Should not panic
}

/// Test platform-specific install instructions formatting
#[test]
fn test_install_instructions_formatting() {
    use mcp_helper::deps::InstallMethod;

    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "Windows method".to_string(),
            command: "winget install".to_string(),
            description: None,
        }],
        macos: vec![InstallMethod {
            name: "macOS method".to_string(),
            command: "brew install".to_string(),
            description: Some("Using Homebrew".to_string()),
        }],
        linux: vec![InstallMethod {
            name: "Linux method".to_string(),
            command: "apt install".to_string(),
            description: Some("For Ubuntu/Debian".to_string()),
        }],
    };

    let error = McpError::missing_dependency("test", None, instructions);
    let display = format!("{error}");

    // Should show platform-specific instructions
    #[cfg(target_os = "windows")]
    assert!(display.contains("winget install"));

    #[cfg(target_os = "macos")]
    assert!(display.contains("brew install"));

    #[cfg(target_os = "linux")]
    assert!(display.contains("apt install"));

    // Should not panic on any platform
    assert!(!display.is_empty());
}
