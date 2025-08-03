//! Comprehensive tests for McpServer trait and core server functionality

use anyhow::Result;
use mcp_helper::deps::{Dependency, DependencyCheck, DependencyChecker, DependencyStatus};
use mcp_helper::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock implementation of McpServer for testing trait behavior
#[derive(Clone)]
struct TestMcpServer {
    metadata: ServerMetadata,
    validate_should_fail: bool,
    validate_error_message: String,
    command: String,
    args: Vec<String>,
    dependency: Dependency,
    dependency_status: DependencyStatus,
}

impl TestMcpServer {
    fn new(name: &str) -> Self {
        let metadata = ServerMetadata {
            name: name.to_string(),
            description: Some("Test server".to_string()),
            server_type: ServerType::Npm {
                package: name.to_string(),
                version: None,
            },
            required_config: vec![],
            optional_config: vec![],
        };

        Self {
            metadata,
            validate_should_fail: false,
            validate_error_message: "Validation failed".to_string(),
            command: "node".to_string(),
            args: vec![name.to_string()],
            dependency: Dependency::NodeJs {
                min_version: Some("18.0.0".to_string()),
            },
            dependency_status: DependencyStatus::Installed {
                version: Some("18.0.0".to_string()),
            },
        }
    }

    fn with_description(mut self, desc: &str) -> Self {
        self.metadata.description = Some(desc.to_string());
        self
    }

    fn with_required_config(mut self, fields: Vec<ConfigField>) -> Self {
        self.metadata.required_config = fields;
        self
    }

    fn with_optional_config(mut self, fields: Vec<ConfigField>) -> Self {
        self.metadata.optional_config = fields;
        self
    }

    fn with_validation_failure(mut self, message: &str) -> Self {
        self.validate_should_fail = true;
        self.validate_error_message = message.to_string();
        self
    }

    fn with_command(mut self, cmd: &str, args: Vec<String>) -> Self {
        self.command = cmd.to_string();
        self.args = args;
        self
    }

    fn with_dependency(mut self, dep: Dependency, status: DependencyStatus) -> Self {
        self.dependency = dep;
        self.dependency_status = status;
        self
    }

    fn with_server_type(mut self, server_type: ServerType) -> Self {
        self.metadata.server_type = server_type;
        self
    }
}

impl McpServer for TestMcpServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, _config: &HashMap<String, String>) -> Result<()> {
        if self.validate_should_fail {
            anyhow::bail!("{}", self.validate_error_message);
        }
        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        Ok((self.command.clone(), self.args.clone()))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        Box::new(TestDependencyChecker {
            dependency: self.dependency.clone(),
            status: self.dependency_status.clone(),
        })
    }
}

struct TestDependencyChecker {
    dependency: Dependency,
    status: DependencyStatus,
}

impl DependencyChecker for TestDependencyChecker {
    fn check(&self) -> Result<DependencyCheck> {
        Ok(DependencyCheck {
            dependency: self.dependency.clone(),
            status: self.status.clone(),
            install_instructions: None,
        })
    }
}

#[test]
fn test_mcp_server_trait_basic_implementation() {
    let server = TestMcpServer::new("test-server");

    // Test metadata access
    let metadata = server.metadata();
    assert_eq!(metadata.name, "test-server");
    assert_eq!(metadata.description, Some("Test server".to_string()));

    // Test server type
    match &metadata.server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "test-server");
            assert_eq!(version, &None);
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_mcp_server_with_config_fields() {
    let required_fields = vec![
        ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key for authentication".to_string()),
            default: None,
        },
        ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: Some("Port to listen on".to_string()),
            default: Some("8080".to_string()),
        },
    ];

    let optional_fields = vec![
        ConfigField {
            name: "debug".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: Some("Enable debug mode".to_string()),
            default: Some("false".to_string()),
        },
        ConfigField {
            name: "config_path".to_string(),
            field_type: ConfigFieldType::Path,
            description: Some("Path to configuration file".to_string()),
            default: None,
        },
    ];

    let server = TestMcpServer::new("config-test")
        .with_required_config(required_fields.clone())
        .with_optional_config(optional_fields.clone());

    let metadata = server.metadata();
    assert_eq!(metadata.required_config.len(), 2);
    assert_eq!(metadata.optional_config.len(), 2);

    // Verify required fields
    assert_eq!(metadata.required_config[0].name, "api_key");
    assert_eq!(
        metadata.required_config[0].field_type,
        ConfigFieldType::String
    );
    assert_eq!(metadata.required_config[1].name, "port");
    assert_eq!(
        metadata.required_config[1].field_type,
        ConfigFieldType::Number
    );
    assert_eq!(
        metadata.required_config[1].default,
        Some("8080".to_string())
    );

    // Verify optional fields
    assert_eq!(metadata.optional_config[0].name, "debug");
    assert_eq!(
        metadata.optional_config[0].field_type,
        ConfigFieldType::Boolean
    );
    assert_eq!(
        metadata.optional_config[0].default,
        Some("false".to_string())
    );
}

#[test]
fn test_mcp_server_validate_config_success() {
    let server = TestMcpServer::new("validation-test");

    let mut config = HashMap::new();
    config.insert("key1".to_string(), "value1".to_string());
    config.insert("key2".to_string(), "value2".to_string());

    let result = server.validate_config(&config);
    assert!(result.is_ok());
}

#[test]
fn test_mcp_server_validate_config_failure() {
    let server = TestMcpServer::new("validation-fail")
        .with_validation_failure("Missing required field: api_key");

    let config = HashMap::new();

    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing required field: api_key"));
}

#[test]
fn test_mcp_server_generate_command() {
    let server = TestMcpServer::new("command-test")
        .with_command("python", vec!["-m".to_string(), "mcp_server".to_string()]);

    let (cmd, args) = server.generate_command().unwrap();
    assert_eq!(cmd, "python");
    assert_eq!(args, vec!["-m", "mcp_server"]);
}

#[test]
fn test_mcp_server_dependency_check() {
    let server = TestMcpServer::new("dep-test").with_dependency(
        Dependency::Python {
            min_version: Some("3.11.0".to_string()),
        },
        DependencyStatus::Installed {
            version: Some("3.11.0".to_string()),
        },
    );

    let dep_checker = server.dependency();
    let dep_check = dep_checker.check().unwrap();

    match &dep_check.dependency {
        Dependency::Python { min_version } => {
            assert_eq!(min_version, &Some("3.11.0".to_string()));
        }
        _ => panic!("Expected Python dependency"),
    }

    match &dep_check.status {
        DependencyStatus::Installed { version } => {
            assert_eq!(version, &Some("3.11.0".to_string()));
        }
        _ => panic!("Expected installed status"),
    }
}

#[test]
fn test_mcp_server_different_server_types() {
    // Test Docker server type
    let docker_server = TestMcpServer::new("docker-test").with_server_type(ServerType::Docker {
        image: "mcp/server".to_string(),
        tag: Some("latest".to_string()),
    });

    match &docker_server.metadata().server_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "mcp/server");
            assert_eq!(tag, &Some("latest".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }

    // Test Binary server type
    let binary_server = TestMcpServer::new("binary-test").with_server_type(ServerType::Binary {
        url: "https://github.com/example/mcp/releases/download/v1.0/mcp".to_string(),
        checksum: Some("sha256:abcdef123456".to_string()),
    });

    match &binary_server.metadata().server_type {
        ServerType::Binary { url, checksum } => {
            assert!(url.contains("github.com"));
            assert_eq!(checksum, &Some("sha256:abcdef123456".to_string()));
        }
        _ => panic!("Expected Binary server type"),
    }

    // Test Python server type
    let python_server = TestMcpServer::new("python-test").with_server_type(ServerType::Python {
        package: "mcp-server".to_string(),
        version: Some("2.0.0".to_string()),
    });

    match &python_server.metadata().server_type {
        ServerType::Python { package, version } => {
            assert_eq!(package, "mcp-server");
            assert_eq!(version, &Some("2.0.0".to_string()));
        }
        _ => panic!("Expected Python server type"),
    }
}

#[test]
fn test_mcp_server_trait_send_sync() {
    // Verify that McpServer trait objects can be sent between threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn McpServer>>();

    // Test actual usage across threads
    let server: Box<dyn McpServer> = Box::new(TestMcpServer::new("thread-safe"));
    let server = Arc::new(server);

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let server = Arc::clone(&server);
            std::thread::spawn(move || {
                let metadata = server.metadata();
                assert_eq!(metadata.name, "thread-safe");

                let (cmd, _) = server.generate_command().unwrap();
                assert_eq!(cmd, "node");

                let mut config = HashMap::new();
                config.insert(format!("key{i}"), format!("value{i}"));
                assert!(server.validate_config(&config).is_ok());
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_config_field_type_equality() {
    assert_eq!(ConfigFieldType::String, ConfigFieldType::String);
    assert_ne!(ConfigFieldType::String, ConfigFieldType::Number);
    assert_ne!(ConfigFieldType::Boolean, ConfigFieldType::Path);
    assert_ne!(ConfigFieldType::Url, ConfigFieldType::String);

    // Test all variants
    let types = [
        ConfigFieldType::String,
        ConfigFieldType::Number,
        ConfigFieldType::Boolean,
        ConfigFieldType::Path,
        ConfigFieldType::Url,
    ];

    for (i, type1) in types.iter().enumerate() {
        for (j, type2) in types.iter().enumerate() {
            if i == j {
                assert_eq!(type1, type2);
            } else {
                assert_ne!(type1, type2);
            }
        }
    }
}

#[test]
fn test_config_field_serialization() {
    let field = ConfigField {
        name: "test_field".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("A test URL field".to_string()),
        default: Some("https://example.com".to_string()),
    };

    // Test serialization
    let json = serde_json::to_string(&field).unwrap();
    assert!(json.contains("\"name\":\"test_field\""));
    assert!(json.contains("\"field_type\":\"Url\""));
    assert!(json.contains("\"description\":\"A test URL field\""));
    assert!(json.contains("\"default\":\"https://example.com\""));

    // Test deserialization
    let deserialized: ConfigField = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, field.name);
    assert_eq!(deserialized.field_type, field.field_type);
    assert_eq!(deserialized.description, field.description);
    assert_eq!(deserialized.default, field.default);
}

#[test]
fn test_server_metadata_complex_scenario() {
    let server = TestMcpServer::new("complex-server")
        .with_description("A complex MCP server with many configuration options")
        .with_required_config(vec![
            ConfigField {
                name: "database_url".to_string(),
                field_type: ConfigFieldType::Url,
                description: Some("PostgreSQL connection URL".to_string()),
                default: None,
            },
            ConfigField {
                name: "api_key".to_string(),
                field_type: ConfigFieldType::String,
                description: Some("API key for external service".to_string()),
                default: None,
            },
        ])
        .with_optional_config(vec![
            ConfigField {
                name: "port".to_string(),
                field_type: ConfigFieldType::Number,
                description: Some("Port to listen on".to_string()),
                default: Some("3000".to_string()),
            },
            ConfigField {
                name: "enable_logging".to_string(),
                field_type: ConfigFieldType::Boolean,
                description: Some("Enable detailed logging".to_string()),
                default: Some("true".to_string()),
            },
            ConfigField {
                name: "log_path".to_string(),
                field_type: ConfigFieldType::Path,
                description: Some("Path to log file".to_string()),
                default: Some("/var/log/mcp-server.log".to_string()),
            },
        ])
        .with_server_type(ServerType::Docker {
            image: "complex/mcp-server".to_string(),
            tag: Some("v2.5.0".to_string()),
        })
        .with_dependency(
            Dependency::Docker {
                min_version: Some("24.0.0".to_string()),
                requires_compose: false,
            },
            DependencyStatus::Installed {
                version: Some("24.0.0".to_string()),
            },
        );

    let metadata = server.metadata();

    // Verify all metadata
    assert_eq!(metadata.name, "complex-server");
    assert!(metadata
        .description
        .as_ref()
        .unwrap()
        .contains("complex MCP server"));
    assert_eq!(metadata.required_config.len(), 2);
    assert_eq!(metadata.optional_config.len(), 3);

    // Verify it's a Docker server
    match &metadata.server_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "complex/mcp-server");
            assert_eq!(tag, &Some("v2.5.0".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }

    // Verify dependency
    let dep_checker = server.dependency();
    let dep_check = dep_checker.check().unwrap();
    match &dep_check.dependency {
        Dependency::Docker {
            min_version,
            requires_compose,
        } => {
            assert_eq!(min_version, &Some("24.0.0".to_string()));
            assert_eq!(requires_compose, &false);
        }
        _ => panic!("Expected Docker dependency"),
    }
}

#[test]
fn test_mcp_server_validation_with_config_fields() {
    let server = TestMcpServer::new("validation-fields").with_required_config(vec![ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    }]);

    // Test with missing required field
    let empty_config = HashMap::new();
    assert!(server.validate_config(&empty_config).is_ok()); // Our mock doesn't actually validate

    // Test with provided required field
    let mut config = HashMap::new();
    config.insert("api_key".to_string(), "secret123".to_string());
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_concurrent_server_access() {
    let server = Arc::new(Mutex::new(TestMcpServer::new("concurrent-test")));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let server = Arc::clone(&server);
            std::thread::spawn(move || {
                // Read metadata
                {
                    let server = server.lock().unwrap();
                    let metadata = server.metadata();
                    assert_eq!(metadata.name, "concurrent-test");
                }

                // Generate command
                {
                    let server = server.lock().unwrap();
                    let (cmd, _) = server.generate_command().unwrap();
                    assert_eq!(cmd, "node");
                }

                // Validate config
                {
                    let server = server.lock().unwrap();
                    let mut config = HashMap::new();
                    config.insert(format!("thread_{i}"), format!("value_{i}"));
                    assert!(server.validate_config(&config).is_ok());
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_server_type_serialization() {
    // Test NPM type
    let npm_type = ServerType::Npm {
        package: "@test/package".to_string(),
        version: Some("1.2.3".to_string()),
    };
    let json = serde_json::to_string(&npm_type).unwrap();
    let deserialized: ServerType = serde_json::from_str(&json).unwrap();
    assert_eq!(npm_type, deserialized);

    // Test Docker type
    let docker_type = ServerType::Docker {
        image: "test/image".to_string(),
        tag: Some("latest".to_string()),
    };
    let json = serde_json::to_string(&docker_type).unwrap();
    let deserialized: ServerType = serde_json::from_str(&json).unwrap();
    assert_eq!(docker_type, deserialized);

    // Test Binary type
    let binary_type = ServerType::Binary {
        url: "https://example.com/binary".to_string(),
        checksum: Some("sha256:abc123".to_string()),
    };
    let json = serde_json::to_string(&binary_type).unwrap();
    let deserialized: ServerType = serde_json::from_str(&json).unwrap();
    assert_eq!(binary_type, deserialized);

    // Test Python type
    let python_type = ServerType::Python {
        package: "mcp-package".to_string(),
        version: None,
    };
    let json = serde_json::to_string(&python_type).unwrap();
    let deserialized: ServerType = serde_json::from_str(&json).unwrap();
    assert_eq!(python_type, deserialized);
}
