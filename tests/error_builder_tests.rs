use mcp_helper::deps::{InstallInstructions, InstallMethod};
use mcp_helper::error::ErrorBuilder;
use mcp_helper::McpError;

#[test]
fn test_missing_dependency_builder_basic() {
    let error = ErrorBuilder::missing_dependency("Node.js").build();

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            install_instructions,
        } => {
            assert_eq!(dependency, "Node.js");
            assert_eq!(required_version, None);
            // Default builder uses empty install instructions
            assert!(install_instructions.windows.is_empty());
            assert!(install_instructions.macos.is_empty());
            assert!(install_instructions.linux.is_empty());
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_missing_dependency_builder_with_version() {
    let error = ErrorBuilder::missing_dependency("Python")
        .version("3.9.0")
        .build();

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Python");
            assert_eq!(required_version, Some("3.9.0".to_string()));
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_missing_dependency_builder_with_instructions() {
    let instructions = InstallInstructions {
        windows: vec![InstallMethod {
            name: "winget".to_string(),
            command: "winget install nodejs".to_string(),
            description: Some("Windows Package Manager".to_string()),
        }],
        macos: vec![],
        linux: vec![],
    };

    let error = ErrorBuilder::missing_dependency("Node.js")
        .version("16.0.0")
        .instructions(instructions)
        .build();

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            install_instructions,
        } => {
            assert_eq!(dependency, "Node.js");
            assert_eq!(required_version, Some("16.0.0".to_string()));
            assert_eq!(install_instructions.windows.len(), 1);
            assert_eq!(install_instructions.windows[0].name, "winget");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_version_mismatch_builder_basic() {
    let error = ErrorBuilder::version_mismatch("Git").build();

    match error {
        McpError::VersionMismatch {
            dependency,
            current_version,
            required_version,
            upgrade_instructions,
        } => {
            assert_eq!(dependency, "Git");
            assert_eq!(current_version, "");
            assert_eq!(required_version, "");
            assert_eq!(upgrade_instructions.windows.len(), 0);
        }
        _ => panic!("Expected VersionMismatch error"),
    }
}

#[test]
fn test_version_mismatch_builder_with_versions() {
    let error = ErrorBuilder::version_mismatch("Docker")
        .installed("20.10.0")
        .required("24.0.0")
        .build();

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
        _ => panic!("Expected VersionMismatch error"),
    }
}

#[test]
fn test_version_mismatch_builder_with_all_fields() {
    let instructions = InstallInstructions {
        windows: vec![],
        macos: vec![InstallMethod {
            name: "homebrew".to_string(),
            command: "brew upgrade docker".to_string(),
            description: Some("Upgrade via Homebrew".to_string()),
        }],
        linux: vec![],
    };

    let error = ErrorBuilder::version_mismatch("Docker")
        .installed("20.10.0")
        .required("24.0.0")
        .instructions(instructions)
        .build();

    match error {
        McpError::VersionMismatch {
            dependency,
            current_version,
            required_version,
            upgrade_instructions,
        } => {
            assert_eq!(dependency, "Docker");
            assert_eq!(current_version, "20.10.0");
            assert_eq!(required_version, "24.0.0");
            assert_eq!(upgrade_instructions.macos.len(), 1);
            assert_eq!(upgrade_instructions.macos[0].command, "brew upgrade docker");
        }
        _ => panic!("Expected VersionMismatch error"),
    }
}

#[test]
fn test_config_required_builder_basic() {
    let error = ErrorBuilder::config_required("my-server").build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "my-server");
            assert_eq!(missing_fields.len(), 0);
            assert_eq!(field_descriptions.len(), 0);
        }
        _ => panic!("Expected ConfigurationRequired error"),
    }
}

#[test]
fn test_config_required_builder_with_single_field() {
    let error = ErrorBuilder::config_required("api-server")
        .field("api_key", "API key for authentication")
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "api-server");
            assert_eq!(missing_fields, vec!["api_key"]);
            assert_eq!(field_descriptions.len(), 1);
            assert_eq!(field_descriptions[0].0, "api_key");
            assert_eq!(field_descriptions[0].1, "API key for authentication");
        }
        _ => panic!("Expected ConfigurationRequired error"),
    }
}

#[test]
fn test_config_required_builder_with_multiple_fields() {
    let error = ErrorBuilder::config_required("database-server")
        .field("host", "Database host")
        .field("port", "Database port")
        .field("password", "Database password")
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "database-server");
            assert_eq!(missing_fields.len(), 3);
            assert_eq!(missing_fields[0], "host");
            assert_eq!(missing_fields[1], "port");
            assert_eq!(missing_fields[2], "password");
            assert_eq!(field_descriptions.len(), 3);
        }
        _ => panic!("Expected ConfigurationRequired error"),
    }
}

#[test]
fn test_config_required_builder_with_fields_iterator() {
    let fields = vec![
        ("url", "Server URL"),
        ("token", "Authentication token"),
        ("timeout", "Request timeout in seconds"),
    ];

    let error = ErrorBuilder::config_required("http-server")
        .fields(fields)
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "http-server");
            assert_eq!(missing_fields.len(), 3);
            assert_eq!(missing_fields[0], "url");
            assert_eq!(missing_fields[1], "token");
            assert_eq!(missing_fields[2], "timeout");
            assert_eq!(field_descriptions.len(), 3);
            assert_eq!(
                field_descriptions[0],
                ("url".to_string(), "Server URL".to_string())
            );
            assert_eq!(
                field_descriptions[1],
                ("token".to_string(), "Authentication token".to_string())
            );
            assert_eq!(
                field_descriptions[2],
                (
                    "timeout".to_string(),
                    "Request timeout in seconds".to_string()
                )
            );
        }
        _ => panic!("Expected ConfigurationRequired error"),
    }
}

#[test]
fn test_config_required_builder_mixed_methods() {
    let error = ErrorBuilder::config_required("complex-server")
        .field("primary", "Primary configuration")
        .fields(vec![
            ("secondary", "Secondary configuration"),
            ("tertiary", "Tertiary configuration"),
        ])
        .field("final", "Final configuration")
        .build();

    match error {
        McpError::ConfigurationRequired {
            missing_fields,
            field_descriptions,
            ..
        } => {
            assert_eq!(missing_fields.len(), 4);
            assert_eq!(missing_fields[0], "primary");
            assert_eq!(missing_fields[1], "secondary");
            assert_eq!(missing_fields[2], "tertiary");
            assert_eq!(missing_fields[3], "final");
            assert_eq!(field_descriptions.len(), 4);
        }
        _ => panic!("Expected ConfigurationRequired error"),
    }
}

#[test]
fn test_error_builder_chaining() {
    // Test that all builder methods return self for chaining
    let error1 = ErrorBuilder::missing_dependency("test")
        .version("1.0.0")
        .instructions(InstallInstructions {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        })
        .build();

    let error2 = ErrorBuilder::version_mismatch("test")
        .installed("1.0.0")
        .required("2.0.0")
        .instructions(InstallInstructions {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        })
        .build();

    let error3 = ErrorBuilder::config_required("test")
        .field("a", "A")
        .fields(vec![("b", "B")])
        .build();

    // Just ensure they compile and produce the right error types
    assert!(matches!(error1, McpError::MissingDependency { .. }));
    assert!(matches!(error2, McpError::VersionMismatch { .. }));
    assert!(matches!(error3, McpError::ConfigurationRequired { .. }));
}

#[test]
fn test_builder_with_string_types() {
    // Test that builders accept different string types
    let error1 = ErrorBuilder::missing_dependency("Test1").build();
    let error2 = ErrorBuilder::missing_dependency(&String::from("Test2")).build();
    let error3 = ErrorBuilder::missing_dependency(&String::from("Test3")).build();

    match (error1, error2, error3) {
        (
            McpError::MissingDependency {
                dependency: dep1, ..
            },
            McpError::MissingDependency {
                dependency: dep2, ..
            },
            McpError::MissingDependency {
                dependency: dep3, ..
            },
        ) => {
            assert_eq!(dep1, "Test1");
            assert_eq!(dep2, "Test2");
            assert_eq!(dep3, "Test3");
        }
        _ => panic!("Expected MissingDependency variants"),
    }
}

#[test]
fn test_config_required_fields_with_owned_strings() {
    let fields: Vec<(String, String)> = vec![
        ("field1".to_string(), "Description 1".to_string()),
        ("field2".to_string(), "Description 2".to_string()),
    ];

    let error = ErrorBuilder::config_required("test").fields(fields).build();

    match error {
        McpError::ConfigurationRequired { missing_fields, .. } => {
            assert_eq!(missing_fields.len(), 2);
            assert_eq!(missing_fields[0], "field1");
            assert_eq!(missing_fields[1], "field2");
        }
        _ => panic!("Expected ConfigurationRequired variant"),
    }
}

#[test]
fn test_version_mismatch_builder_chaining() {
    // Test that methods can be chained in any order
    let error1 = ErrorBuilder::version_mismatch("Test")
        .required("2.0")
        .installed("1.0")
        .build();

    let error2 = ErrorBuilder::version_mismatch("Test")
        .installed("1.0")
        .required("2.0")
        .build();

    match (error1, error2) {
        (
            McpError::VersionMismatch {
                current_version: v1_current,
                required_version: v1_required,
                ..
            },
            McpError::VersionMismatch {
                current_version: v2_current,
                required_version: v2_required,
                ..
            },
        ) => {
            assert_eq!(v1_current, v2_current);
            assert_eq!(v1_required, v2_required);
        }
        _ => panic!("Expected VersionMismatch variants"),
    }
}

#[test]
fn test_version_mismatch_empty_versions() {
    // Test that empty strings can be set for versions
    let error = ErrorBuilder::version_mismatch("Test")
        .installed("")
        .required("")
        .build();

    match error {
        McpError::VersionMismatch {
            current_version,
            required_version,
            ..
        } => {
            assert_eq!(current_version, "");
            assert_eq!(required_version, "");
        }
        _ => panic!("Expected VersionMismatch variant"),
    }
}

#[test]
fn test_complex_config_scenario() {
    // Test a realistic configuration scenario
    let error = ErrorBuilder::config_required("oauth-server")
        .field("client_id", "OAuth2 client identifier")
        .field("client_secret", "OAuth2 client secret")
        .fields(vec![
            ("redirect_uri", "OAuth2 redirect URI"),
            ("scope", "OAuth2 permission scope"),
        ])
        .field("auth_endpoint", "Authorization endpoint URL")
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "oauth-server");
            assert_eq!(missing_fields.len(), 5);

            // Verify order is preserved
            let expected_fields = [
                "client_id",
                "client_secret",
                "redirect_uri",
                "scope",
                "auth_endpoint",
            ];
            for (i, expected) in expected_fields.iter().enumerate() {
                assert_eq!(&missing_fields[i], expected);
            }

            // Verify all descriptions are present
            assert_eq!(field_descriptions.len(), 5);
            for (field, desc) in &field_descriptions {
                assert!(!field.is_empty());
                assert!(!desc.is_empty());
            }
        }
        _ => panic!("Expected ConfigurationRequired variant"),
    }
}
