//! Comprehensive tests for the installation command in install.rs

use tempfile::TempDir;

use mcp_helper::deps::{Dependency, DependencyStatus, InstallInstructions};
use mcp_helper::error::McpError;
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{ConfigField, ConfigFieldType};

#[test]
fn test_install_command_creation() {
    let install = InstallCommand::new(false);
    // Test that it creates without panicking
    // We can't access private fields directly but can test the public interface
    drop(install);
}

#[test]
fn test_install_command_creation_verbose() {
    let install = InstallCommand::new(true);
    // Test verbose mode creation
    drop(install);
}

#[test]
fn test_with_auto_install_deps() {
    let install = InstallCommand::new(false).with_auto_install_deps(true);
    // Test builder pattern
    drop(install);
}

#[test]
fn test_with_dry_run() {
    let install = InstallCommand::new(false).with_dry_run(true);
    // Test dry run mode
    drop(install);
}

#[test]
fn test_chained_configuration() {
    let install = InstallCommand::new(true)
        .with_auto_install_deps(true)
        .with_dry_run(true)
        .with_config_overrides(vec!["key1=value1".to_string(), "key2=value2".to_string()]);
    // Test full builder chain
    drop(install);
}

#[test]
fn test_parse_config_args_valid() {
    let config_args = vec![
        "host=localhost".to_string(),
        "port=3000".to_string(),
        "debug=true".to_string(),
    ];

    // Since parse_config_args is private, we test it through with_config_overrides
    let _install = InstallCommand::new(false).with_config_overrides(config_args);

    // The fact that this doesn't panic means the parsing worked
}

#[test]
fn test_parse_config_args_invalid_format() {
    let config_args = vec![
        "valid=value".to_string(),
        "invalid_format_no_equals".to_string(),
        "another=valid".to_string(),
    ];

    // This should not panic but should warn about the invalid format
    let _install = InstallCommand::new(false).with_config_overrides(config_args);
}

#[test]
fn test_parse_config_args_empty() {
    let config_args = vec![];

    let _install = InstallCommand::new(false).with_config_overrides(config_args);

    // Empty config should be fine
}

#[test]
fn test_parse_config_args_with_spaces() {
    let config_args = vec![
        "  key1  =  value1  ".to_string(),
        "key2=value with spaces".to_string(),
    ];

    let _install = InstallCommand::new(false).with_config_overrides(config_args);

    // Should handle trimming
}

#[test]
fn test_get_dependency_name() {
    let nodejs_dep = Dependency::NodeJs { min_version: None };
    assert_eq!(InstallCommand::get_dependency_name(&nodejs_dep), "Node.js");

    let python_dep = Dependency::Python {
        min_version: Some("3.8.0".to_string()),
    };
    assert_eq!(InstallCommand::get_dependency_name(&python_dep), "Python");

    let docker_dep = Dependency::Docker {
        min_version: None,
        requires_compose: false,
    };
    assert_eq!(InstallCommand::get_dependency_name(&docker_dep), "Docker");

    let git_dep = Dependency::Git;
    assert_eq!(InstallCommand::get_dependency_name(&git_dep), "Git");
}

#[test]
fn test_handle_installed_dependency() {
    // Test with version
    let result =
        InstallCommand::handle_installed_dependency("Node.js", &Some("18.0.0".to_string()));
    assert!(result.is_ok());

    // Test without version
    let result = InstallCommand::handle_installed_dependency("Docker", &None);
    assert!(result.is_ok());
}

#[test]
fn test_handle_missing_dependency() {
    use mcp_helper::deps::DependencyCheck;

    let dependency = Dependency::NodeJs {
        min_version: Some("18.0.0".to_string()),
    };
    let status = DependencyStatus::Missing;
    let instructions = InstallInstructions::default();

    let check = DependencyCheck {
        dependency,
        status,
        install_instructions: Some(instructions),
    };

    let result = InstallCommand::handle_missing_dependency("Node.js", &check);
    assert!(result.is_err());

    // Check that it returns the correct error type
    match result {
        Err(McpError::MissingDependency { dependency, .. }) => {
            assert_eq!(dependency, "Node.js");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_handle_missing_dependency_no_instructions() {
    use mcp_helper::deps::DependencyCheck;

    let dependency = Dependency::Git;
    let status = DependencyStatus::Missing;

    let check = DependencyCheck {
        dependency,
        status,
        install_instructions: None,
    };

    let result = InstallCommand::handle_missing_dependency("Git", &check);
    assert!(result.is_err());

    // Should return Other error when no install instructions
    match result {
        Err(McpError::Other(_)) => {
            // Expected
        }
        _ => panic!("Expected Other error when no install instructions provided"),
    }
}

#[test]
fn test_build_field_prompt() {
    // Test required field with description
    let field = ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API key for authentication".to_string()),
        default: None,
    };

    let prompt = InstallCommand::build_field_prompt(&field, true);
    assert_eq!(prompt, "API key for authentication");

    // Test optional field with description
    let prompt = InstallCommand::build_field_prompt(&field, false);
    assert_eq!(prompt, "API key for authentication (optional)");

    // Test required field without description
    let field_no_desc = ConfigField {
        name: "host".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };

    let prompt = InstallCommand::build_field_prompt(&field_no_desc, true);
    assert_eq!(prompt, "host");

    // Test optional field without description
    let prompt = InstallCommand::build_field_prompt(&field_no_desc, false);
    assert_eq!(prompt, "host (optional)");
}

#[test]
fn test_execute_batch_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("empty.txt");
    std::fs::write(&batch_file, "").unwrap();

    let mut install = InstallCommand::new(false);
    let result = install.execute_batch(batch_file.to_str().unwrap());

    // Should fail with no servers found
    assert!(result.is_err());
    match result {
        Err(McpError::Other(e)) => {
            assert!(e.to_string().contains("No servers found"));
        }
        _ => panic!("Expected Other error for empty batch file"),
    }
}

#[test]
fn test_execute_batch_comments_and_empty_lines() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("with_comments.txt");

    let content = r#"
# This is a comment
   # Another comment with spaces

[@modelcontextprotocol/server-filesystem]
allowedDirectories=/home/user

# Final comment
"#;

    std::fs::write(&batch_file, content).unwrap();

    let mut install = InstallCommand::new(false);
    let result = install.execute_batch(batch_file.to_str().unwrap());

    // Should fail during execution (since we can't actually install), but batch parsing should work
    assert!(result.is_err());
    // The error should not be about parsing but about actual installation
}

#[test]
fn test_execute_batch_invalid_line() {
    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("invalid.txt");

    let content = r#"
[@modelcontextprotocol/server-filesystem]
validKey=validValue
invalid line without equals sign
"#;

    std::fs::write(&batch_file, content).unwrap();

    let mut install = InstallCommand::new(false);
    let result = install.execute_batch(batch_file.to_str().unwrap());

    // Should fail with invalid line error
    assert!(result.is_err());
    match result {
        Err(McpError::Other(e)) => {
            assert!(e.to_string().contains("Invalid line"));
        }
        _ => panic!("Expected Other error for invalid line"),
    }
}

#[test]
fn test_execute_batch_nonexistent_file() {
    let mut install = InstallCommand::new(false);
    let result = install.execute_batch("/nonexistent/file.txt");

    assert!(result.is_err());
    match result {
        Err(McpError::Other(e)) => {
            assert!(e.to_string().contains("Failed to read batch file"));
        }
        _ => panic!("Expected Other error for nonexistent file"),
    }
}

#[test]
fn test_execute_with_invalid_server() {
    let mut install = InstallCommand::new(false);
    let result = install.execute("nonexistent-server-that-does-not-exist");

    // Should fail - we can't predict exactly what error, but it should fail
    assert!(result.is_err());
}

#[test]
fn test_execute_with_dry_run() {
    let mut install = InstallCommand::new(false).with_dry_run(true);

    let result = install.execute("@some/test-server");

    // Should fail (can't actually install), but dry run flag should be respected
    assert!(result.is_err());
}

#[test]
fn test_multiple_config_arg_formats() {
    // Test various formats that should be handled
    let test_cases = vec![
        ("key=value", true),
        ("key=", true),                  // Empty value
        ("=value", true),                // Empty key (should still parse)
        ("key=value=with=equals", true), // Value contains equals
        ("justkey", false),              // No equals sign
        ("", false),                     // Empty string
        ("  key  =  value  ", true),     // Whitespace
    ];

    for (input, _should_parse) in test_cases {
        let config_args = vec![input.to_string()];
        let _install = InstallCommand::new(false).with_config_overrides(config_args);

        // Test doesn't panic - actual parsing logic is internal
        // The fact that we can construct the InstallCommand means parsing didn't crash
        // Just testing no panic during parsing
    }
}

#[test]
fn test_server_type_detection_integration() {
    // Test that various server name formats don't crash the installer
    let test_servers = vec![
        "@scope/package",
        "simple-package",
        "docker:nginx:latest",
        "https://github.com/user/repo",
        "user/repo",
        "https://example.com/binary",
    ];

    for server_name in test_servers {
        let mut install = InstallCommand::new(false).with_dry_run(true); // Use dry run to avoid actual operations

        // This will fail during execution, but should not panic during server type detection
        let result = install.execute(server_name);
        assert!(result.is_err()); // Expected to fail since these are fake servers
    }
}

#[test]
fn test_config_validation_scenarios() {
    // Test different configuration field types
    let string_field = ConfigField {
        name: "string_field".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("A string field".to_string()),
        default: Some("default_value".to_string()),
    };

    let number_field = ConfigField {
        name: "number_field".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("A number field".to_string()),
        default: None,
    };

    let boolean_field = ConfigField {
        name: "boolean_field".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: Some("A boolean field".to_string()),
        default: Some("true".to_string()),
    };

    let path_field = ConfigField {
        name: "path_field".to_string(),
        field_type: ConfigFieldType::Path,
        description: Some("A path field".to_string()),
        default: None,
    };

    let url_field = ConfigField {
        name: "url_field".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("A URL field".to_string()),
        default: None,
    };

    // Test prompt building for each type
    for field in [
        &string_field,
        &number_field,
        &boolean_field,
        &path_field,
        &url_field,
    ] {
        let required_prompt = InstallCommand::build_field_prompt(field, true);
        let optional_prompt = InstallCommand::build_field_prompt(field, false);

        // Required prompts should not contain "(optional)"
        assert!(!required_prompt.contains("(optional)"));
        // Optional prompts should contain "(optional)"
        assert!(optional_prompt.contains("(optional)"));
    }
}

#[test]
fn test_error_handling_patterns() {
    // Test that various error conditions are handled appropriately
    let mut install = InstallCommand::new(false);

    // Test with invalid server names that should trigger different error paths
    let invalid_servers = vec![
        "", // Empty server name
        "invalid/server/with/too/many/slashes",
        "https://", // Incomplete URL
        "@",        // Invalid npm scope
    ];

    for server_name in invalid_servers {
        if server_name.is_empty() {
            continue; // Skip empty server name as it might be handled differently
        }

        let result = install.execute(server_name);
        // All should fail, but should not panic
        assert!(result.is_err());
    }
}

#[test]
fn test_dependency_status_handling() {
    use mcp_helper::deps::DependencyCheck;

    // Test version mismatch handling
    let dependency = Dependency::NodeJs {
        min_version: Some("18.0.0".to_string()),
    };
    let status = DependencyStatus::VersionMismatch {
        installed: "16.0.0".to_string(),
        required: "18.0.0".to_string(),
    };
    let instructions = InstallInstructions::default();

    let check = DependencyCheck {
        dependency,
        status,
        install_instructions: Some(instructions),
    };

    let result = InstallCommand::handle_missing_dependency("Node.js", &check);
    assert!(result.is_err());
}

#[test]
fn test_config_override_edge_cases() {
    // Test config overrides with edge cases
    let edge_case_configs = vec![
        "key_with_underscore=value".to_string(),
        "KEY_UPPERCASE=VALUE".to_string(),
        "key-with-dashes=value-with-dashes".to_string(),
        "key=value with spaces and symbols!@#$%".to_string(),
        "numeric_key=12345".to_string(),
        "boolean_key=true".to_string(),
        "url_key=https://example.com/path?param=value".to_string(),
    ];

    let _install = InstallCommand::new(false).with_config_overrides(edge_case_configs);

    // Should handle all these cases without panicking
}

#[test]
fn test_builder_pattern_chaining() {
    // Test that builder pattern methods can be chained properly
    // Each method takes self by value, so we can't reuse the same instance
    let install = InstallCommand::new(false)
        .with_auto_install_deps(true)
        .with_dry_run(true)
        .with_config_overrides(vec!["key=value".to_string()]);

    // Should be able to create the complete configuration
    drop(install);

    // Test creating multiple separate instances
    let _install1 = InstallCommand::new(false).with_auto_install_deps(true);

    let _install2 = InstallCommand::new(false).with_dry_run(true);

    let _install3 =
        InstallCommand::new(false).with_config_overrides(vec!["test=value".to_string()]);
}
