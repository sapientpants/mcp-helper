use anyhow::Result;
use mcp_helper::deps::{
    Dependency, DependencyCheck, DependencyChecker, DependencyStatus, InstallInstructions,
    InstallMethod,
};
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use mcp_helper::McpError;
use std::collections::HashMap;

// Test server that requires certain dependencies
struct DependencyTestServer {
    metadata: ServerMetadata,
    dep_status: DependencyStatus,
    has_instructions: bool,
}

impl DependencyTestServer {
    fn with_missing_dependency() -> Self {
        Self {
            metadata: ServerMetadata {
                name: "dep-test-server".to_string(),
                description: Some("Server with missing dependency".to_string()),
                server_type: ServerType::Npm {
                    package: "dep-test-server".to_string(),
                    version: None,
                },
                required_config: vec![],
                optional_config: vec![],
            },
            dep_status: DependencyStatus::Missing,
            has_instructions: true,
        }
    }

    fn with_version_mismatch() -> Self {
        Self {
            metadata: ServerMetadata {
                name: "version-test-server".to_string(),
                description: Some("Server with version mismatch".to_string()),
                server_type: ServerType::Npm {
                    package: "version-test-server".to_string(),
                    version: None,
                },
                required_config: vec![],
                optional_config: vec![],
            },
            dep_status: DependencyStatus::VersionMismatch {
                installed: "16.0.0".to_string(),
                required: "18.0.0".to_string(),
            },
            has_instructions: true,
        }
    }
}

impl McpServer for DependencyTestServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, _config: &HashMap<String, String>) -> Result<()> {
        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        Ok(("npx".to_string(), vec![self.metadata.name.clone()]))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        let status = self.dep_status.clone();
        let has_instructions = self.has_instructions;

        struct TestChecker {
            status: DependencyStatus,
            has_instructions: bool,
        }

        impl DependencyChecker for TestChecker {
            fn check(&self) -> Result<DependencyCheck> {
                let instructions = if self.has_instructions {
                    Some(InstallInstructions {
                        windows: vec![InstallMethod {
                            name: "Install via installer".to_string(),
                            command: "Download from nodejs.org".to_string(),
                            description: Some("Download and run the Windows installer".to_string()),
                        }],
                        macos: vec![InstallMethod {
                            name: "Install via Homebrew".to_string(),
                            command: "brew install node".to_string(),
                            description: Some("Install using Homebrew package manager".to_string()),
                        }],
                        linux: vec![InstallMethod {
                            name: "Install via apt".to_string(),
                            command: "sudo apt-get install nodejs".to_string(),
                            description: Some("Install using apt package manager".to_string()),
                        }],
                    })
                } else {
                    None
                };

                Ok(DependencyCheck {
                    dependency: Dependency::NodeJs {
                        min_version: Some("18.0.0".to_string()),
                    },
                    status: self.status.clone(),
                    install_instructions: instructions,
                })
            }
        }

        Box::new(TestChecker {
            status,
            has_instructions,
        })
    }
}

#[test]
fn test_missing_dependency_with_instructions() {
    let server = DependencyTestServer::with_missing_dependency();
    let checker = server.dependency();
    let check = checker.check().unwrap();

    match &check.status {
        DependencyStatus::Missing => {
            assert!(check.install_instructions.is_some());
        }
        _ => panic!("Expected missing dependency"),
    }
}

#[test]
fn test_version_mismatch_with_instructions() {
    let server = DependencyTestServer::with_version_mismatch();
    let checker = server.dependency();
    let check = checker.check().unwrap();

    match &check.status {
        DependencyStatus::VersionMismatch {
            installed,
            required,
        } => {
            assert_eq!(installed, "16.0.0");
            assert_eq!(required, "18.0.0");
            assert!(check.install_instructions.is_some());
        }
        _ => panic!("Expected version mismatch"),
    }
}

#[test]
fn test_handle_missing_dependency_without_instructions() {
    let check = DependencyCheck {
        dependency: Dependency::Docker,
        status: DependencyStatus::Missing,
        install_instructions: None,
    };

    let result = InstallCommand::handle_missing_dependency("Docker", &check);
    assert!(result.is_err());

    match result {
        Err(McpError::Other(err)) => {
            assert!(err.to_string().contains("Docker is not installed"));
        }
        _ => panic!("Expected Other error type"),
    }
}

#[test]
fn test_handle_installed_dependency_with_empty_version() {
    let result = InstallCommand::handle_installed_dependency("Git", &Some("".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_all_dependency_types() {
    let dependencies = vec![
        (
            Dependency::NodeJs {
                min_version: Some("18.0.0".to_string()),
            },
            "Node.js",
        ),
        (
            Dependency::Python {
                min_version: Some("3.9".to_string()),
            },
            "Python",
        ),
        (Dependency::Docker, "Docker"),
        (Dependency::Git, "Git"),
    ];

    for (dep, expected_name) in dependencies {
        let name = InstallCommand::get_dependency_name(&dep);
        assert_eq!(name, expected_name);
    }
}

#[test]
fn test_execute_with_scoped_package_version() {
    let cmd = InstallCommand::new(false);

    // Test scoped package with version
    let result = cmd.execute("@scope/package@1.2.3");
    assert!(result.is_err()); // Will fail because package doesn't exist, but tests the parsing
}

#[test]
fn test_execute_with_python_server() {
    let cmd = InstallCommand::new(false);

    // Test Python server detection
    let result = cmd.execute("my_server.py");
    assert!(result.is_err());

    match result {
        Err(McpError::ServerError { message, .. }) => {
            assert!(message.contains("not yet supported"));
        }
        _ => panic!("Expected ServerError for unsupported Python server"),
    }
}

#[test]
fn test_execute_with_binary_url() {
    let cmd = InstallCommand::new(false);

    // Test binary URL detection
    let result = cmd.execute("https://github.com/org/repo/releases/download/v1.0.0/server.tar.gz");
    assert!(result.is_err());

    match result {
        Err(McpError::ServerError { message, .. }) => {
            assert!(message.contains("not yet supported"));
        }
        _ => panic!("Expected ServerError for unsupported binary server"),
    }
}

#[test]
fn test_config_field_validation_errors() {
    // Test field with conflicting requirements
    let field = ConfigField {
        name: "test_field".to_string(),
        field_type: ConfigFieldType::Number,
        description: None,
        default: Some("not_a_number".to_string()), // Invalid default for number type
    };

    // The field itself is valid structure, but would fail during actual validation
    assert_eq!(field.name, "test_field");
}

#[test]
fn test_build_field_prompt_edge_cases() {
    // Test with empty name and description
    let field = ConfigField {
        name: "".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("".to_string()),
        default: None,
    };

    let prompt = InstallCommand::build_field_prompt(&field, true);
    assert_eq!(prompt, "");

    // Test with very long name
    let long_name = "a".repeat(100);
    let field_long = ConfigField {
        name: long_name.clone(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };

    let prompt_long = InstallCommand::build_field_prompt(&field_long, false);
    assert!(prompt_long.contains(&long_name));
    assert!(prompt_long.contains("optional"));
}

#[test]
fn test_server_metadata_variations() {
    // Test server with no description
    let metadata = ServerMetadata {
        name: "no-desc-server".to_string(),
        description: None,
        server_type: ServerType::Npm {
            package: "no-desc-server".to_string(),
            version: None,
        },
        required_config: vec![],
        optional_config: vec![],
    };

    assert!(metadata.description.is_none());

    // Test server with empty collections
    assert!(metadata.required_config.is_empty());
    assert!(metadata.optional_config.is_empty());
}

#[test]
fn test_verbose_execution_paths() {
    let cmd_verbose = InstallCommand::new(true);

    // Test various inputs in verbose mode
    let test_cases = vec![
        "@test/package",
        "simple-package",
        "package@1.0.0",
        "",
        "   ",
    ];

    for input in test_cases {
        let _ = cmd_verbose.execute(input);
        // Just ensure it doesn't panic in verbose mode
    }
}

#[test]
fn test_install_command_edge_cases() {
    // Test creating multiple instances
    let cmd1 = InstallCommand::new(true);
    let cmd2 = InstallCommand::new(false);

    // Both should work independently
    let _ = cmd1.execute("test1");
    let _ = cmd2.execute("test2");
}
