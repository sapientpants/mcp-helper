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

#[test]
fn test_error_builder_struct() {
    // Test that ErrorBuilder is a unit struct
    let _ = ErrorBuilder;
    // This ensures the struct itself is covered
}

#[test]
fn test_missing_dependency_builder_struct_fields() {
    // Test that builder fields are set correctly
    let builder = ErrorBuilder::missing_dependency("test-dep");
    // Cannot access private fields directly, but we test the build output
    let error = builder.build();

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            install_instructions,
        } => {
            assert_eq!(dependency, "test-dep");
            assert_eq!(required_version, None);
            // Default InstallInstructions should have empty vectors
            assert_eq!(install_instructions.windows.len(), 0);
            assert_eq!(install_instructions.macos.len(), 0);
            assert_eq!(install_instructions.linux.len(), 0);
        }
        _ => panic!("Expected MissingDependency"),
    }
}

#[test]
fn test_version_mismatch_builder_struct_fields() {
    // Test that VersionMismatchBuilder fields are initialized correctly
    let builder = ErrorBuilder::version_mismatch("test-dep");
    let error = builder.build();

    match error {
        McpError::VersionMismatch {
            dependency,
            current_version,
            required_version,
            upgrade_instructions,
        } => {
            assert_eq!(dependency, "test-dep");
            assert_eq!(current_version, ""); // Default empty string
            assert_eq!(required_version, ""); // Default empty string
                                              // Default InstallInstructions
            assert_eq!(upgrade_instructions.windows.len(), 0);
            assert_eq!(upgrade_instructions.macos.len(), 0);
            assert_eq!(upgrade_instructions.linux.len(), 0);
        }
        _ => panic!("Expected VersionMismatch"),
    }
}

#[test]
fn test_config_required_builder_struct_fields() {
    // Test that ConfigRequiredBuilder fields are initialized correctly
    let builder = ErrorBuilder::config_required("test-server");
    let error = builder.build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "test-server");
            assert_eq!(missing_fields, Vec::<String>::new()); // Empty vec
            assert_eq!(field_descriptions, Vec::<(String, String)>::new()); // Empty vec
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}

#[test]
fn test_missing_dependency_builder_mutation() {
    // Test that builder mutations work correctly
    let mut builder = ErrorBuilder::missing_dependency("dep");

    // Test version mutation
    builder = builder.version("1.0.0");
    let error = builder.build();

    match error {
        McpError::MissingDependency {
            required_version, ..
        } => {
            assert_eq!(required_version, Some("1.0.0".to_string()));
        }
        _ => panic!("Expected MissingDependency"),
    }
}

#[test]
fn test_version_mismatch_builder_mutation() {
    // Test that builder mutations work correctly
    let mut builder = ErrorBuilder::version_mismatch("dep");

    // Test installed version mutation
    builder = builder.installed("1.0.0");
    // Test required version mutation
    builder = builder.required("2.0.0");

    let error = builder.build();

    match error {
        McpError::VersionMismatch {
            current_version,
            required_version,
            ..
        } => {
            assert_eq!(current_version, "1.0.0");
            assert_eq!(required_version, "2.0.0");
        }
        _ => panic!("Expected VersionMismatch"),
    }
}

#[test]
fn test_config_required_fields_iterator_implementation() {
    // Test the generic iterator implementation with different types
    let fields_vec: Vec<(&str, &str)> = vec![("field1", "desc1"), ("field2", "desc2")];

    let error = ErrorBuilder::config_required("server")
        .fields(fields_vec)
        .build();

    match error {
        McpError::ConfigurationRequired {
            missing_fields,
            field_descriptions,
            ..
        } => {
            assert_eq!(missing_fields.len(), 2);
            assert_eq!(field_descriptions.len(), 2);
        }
        _ => panic!("Expected ConfigurationRequired"),
    }

    // Test with array
    let fields_array = [("field3", "desc3"), ("field4", "desc4")];

    let error = ErrorBuilder::config_required("server2")
        .fields(fields_array)
        .build();

    match error {
        McpError::ConfigurationRequired { missing_fields, .. } => {
            assert_eq!(missing_fields.len(), 2);
            assert_eq!(missing_fields[0], "field3");
            assert_eq!(missing_fields[1], "field4");
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}

#[test]
fn test_config_required_fields_string_conversion() {
    // Test Into<String> conversion in fields iterator
    let fields: Vec<(String, String)> = vec![
        ("field1".to_string(), "desc1".to_string()),
        ("field2".to_string(), "desc2".to_string()),
    ];

    let error = ErrorBuilder::config_required("server")
        .fields(fields)
        .build();

    match error {
        McpError::ConfigurationRequired {
            missing_fields,
            field_descriptions,
            ..
        } => {
            assert_eq!(missing_fields.len(), 2);
            assert_eq!(field_descriptions[0].0, "field1");
            assert_eq!(field_descriptions[0].1, "desc1");
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}

#[test]
fn test_builder_edge_cases() {
    // Test with empty strings
    let error = ErrorBuilder::missing_dependency("").build();
    match error {
        McpError::MissingDependency { dependency, .. } => {
            assert_eq!(dependency, "");
        }
        _ => panic!("Expected MissingDependency"),
    }

    // Test with very long strings
    let long_string = "a".repeat(1000);
    let error = ErrorBuilder::config_required(&long_string)
        .field(&long_string, &long_string)
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, long_string);
            assert_eq!(missing_fields[0], long_string);
            assert_eq!(field_descriptions[0].1, long_string);
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}

#[test]
fn test_install_instructions_boxing() {
    // Test that InstallInstructions are properly boxed
    let large_instructions = InstallInstructions {
        windows: (0..100)
            .map(|i| InstallMethod {
                name: format!("method{i}"),
                command: format!("command{i}"),
                description: Some(format!("desc{i}")),
            })
            .collect(),
        macos: vec![],
        linux: vec![],
    };

    // Test with MissingDependency
    let error1 = ErrorBuilder::missing_dependency("dep1")
        .instructions(large_instructions.clone())
        .build();

    match error1 {
        McpError::MissingDependency {
            install_instructions,
            ..
        } => {
            assert_eq!(install_instructions.windows.len(), 100);
        }
        _ => panic!("Expected MissingDependency"),
    }

    // Test with VersionMismatch
    let error2 = ErrorBuilder::version_mismatch("dep2")
        .instructions(large_instructions)
        .build();

    match error2 {
        McpError::VersionMismatch {
            upgrade_instructions,
            ..
        } => {
            assert_eq!(upgrade_instructions.windows.len(), 100);
        }
        _ => panic!("Expected VersionMismatch"),
    }
}

#[test]
fn test_unicode_and_special_characters() {
    // Test with unicode
    let error = ErrorBuilder::missing_dependency("Python 🐍")
        .version("≥3.8")
        .build();

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Python 🐍");
            assert_eq!(required_version, Some("≥3.8".to_string()));
        }
        _ => panic!("Expected MissingDependency"),
    }

    // Test with newlines and special characters
    let error = ErrorBuilder::config_required("server\nwith\nnewlines")
        .field("field\twith\ttabs", "desc\\with\\backslashes")
        .build();

    match error {
        McpError::ConfigurationRequired {
            server_name,
            missing_fields,
            field_descriptions,
        } => {
            assert_eq!(server_name, "server\nwith\nnewlines");
            assert_eq!(missing_fields[0], "field\twith\ttabs");
            assert_eq!(field_descriptions[0].1, "desc\\with\\backslashes");
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}

#[test]
fn test_builder_lifetimes() {
    // Test that builders work with references of different lifetimes
    let dep_name = String::from("dependency");
    let version = String::from("1.0.0");

    let error = ErrorBuilder::missing_dependency(&dep_name)
        .version(&version)
        .build();

    // The original strings should still be valid
    assert_eq!(dep_name, "dependency");
    assert_eq!(version, "1.0.0");

    match error {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, dep_name);
            assert_eq!(required_version, Some(version));
        }
        _ => panic!("Expected MissingDependency"),
    }
}

#[test]
fn test_config_required_field_clone_behavior() {
    // Test that field names are properly cloned in the fields() method
    let field_name = String::from("field1");
    let field_desc = String::from("description1");

    let error = ErrorBuilder::config_required("server")
        .fields(vec![(field_name.clone(), field_desc.clone())])
        .build();

    // Original strings should still be valid
    assert_eq!(field_name, "field1");
    assert_eq!(field_desc, "description1");

    match error {
        McpError::ConfigurationRequired {
            missing_fields,
            field_descriptions,
            ..
        } => {
            assert_eq!(missing_fields[0], field_name);
            assert_eq!(field_descriptions[0].0, field_name);
            assert_eq!(field_descriptions[0].1, field_desc);
        }
        _ => panic!("Expected ConfigurationRequired"),
    }
}
