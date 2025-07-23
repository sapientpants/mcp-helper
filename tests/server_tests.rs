use mcp_helper::server::{
    detect_server_type, parse_npm_package, ConfigField, ConfigFieldType, McpServer, ServerMetadata,
    ServerType,
};
use std::collections::HashMap;

#[test]
fn test_detect_npm_package() {
    match detect_server_type("@modelcontextprotocol/server-filesystem") {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert_eq!(version, None);
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_detect_npm_with_version() {
    match detect_server_type("@modelcontextprotocol/server-filesystem@1.0.0") {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert_eq!(version, Some("1.0.0".to_string()));
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_detect_docker() {
    match detect_server_type("docker:ollama/ollama:latest") {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "ollama/ollama");
            assert_eq!(tag, "latest");
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_detect_binary() {
    let result = detect_server_type("https://github.com/owner/repo/releases/download/v1.0/binary");
    match result {
        ServerType::Binary { url, .. } => {
            assert_eq!(
                url,
                "https://github.com/owner/repo/releases/download/v1.0/binary"
            );
        }
        _ => panic!("Expected Binary server type, got: {:?}", result),
    }
}

#[test]
fn test_detect_python() {
    match detect_server_type("server.py") {
        ServerType::Python { package, version } => {
            assert_eq!(package, "server.py");
            assert_eq!(version, None);
        }
        _ => panic!("Expected Python server type"),
    }
}

#[test]
fn test_detect_simple_npm() {
    match detect_server_type("express") {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "express");
            assert_eq!(version, None);
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_detect_docker_without_tag() {
    match detect_server_type("docker:nginx") {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "nginx");
            assert_eq!(tag, "latest");
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_server_metadata_creation() {
    let metadata = ServerMetadata {
        name: "test-server".to_string(),
        description: Some("A test server".to_string()),
        server_type: ServerType::Npm {
            package: "test".to_string(),
            version: None,
        },
        required_config: vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key".to_string()),
            default: None,
        }],
        optional_config: vec![],
    };

    assert_eq!(metadata.name, "test-server");
    assert_eq!(metadata.description, Some("A test server".to_string()));
    assert_eq!(metadata.required_config.len(), 1);
    assert_eq!(metadata.optional_config.len(), 0);
}

#[test]
fn test_config_field_types() {
    let fields = vec![
        ConfigField {
            name: "string_field".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: Some("default".to_string()),
        },
        ConfigField {
            name: "number_field".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        },
        ConfigField {
            name: "bool_field".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: Some("true".to_string()),
        },
        ConfigField {
            name: "path_field".to_string(),
            field_type: ConfigFieldType::Path,
            description: None,
            default: None,
        },
        ConfigField {
            name: "url_field".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        },
    ];

    assert_eq!(fields[0].field_type, ConfigFieldType::String);
    assert_eq!(fields[1].field_type, ConfigFieldType::Number);
    assert_eq!(fields[2].field_type, ConfigFieldType::Boolean);
    assert_eq!(fields[3].field_type, ConfigFieldType::Path);
    assert_eq!(fields[4].field_type, ConfigFieldType::Url);
}

#[test]
fn test_parse_npm_simple_package() {
    let (pkg, version) = parse_npm_package("lodash");
    assert_eq!(pkg, "lodash");
    assert_eq!(version, None);
}

#[test]
fn test_parse_npm_simple_with_version() {
    let (pkg, version) = parse_npm_package("lodash@4.17.21");
    assert_eq!(pkg, "lodash");
    assert_eq!(version, Some("4.17.21".to_string()));
}

#[test]
fn test_parse_npm_scoped_no_version() {
    let (pkg, version) = parse_npm_package("@types/node");
    assert_eq!(pkg, "@types/node");
    assert_eq!(version, None);
}

#[test]
fn test_parse_npm_scoped_with_version() {
    let (pkg, version) = parse_npm_package("@types/node@18.0.0");
    assert_eq!(pkg, "@types/node");
    assert_eq!(version, Some("18.0.0".to_string()));
}

#[test]
fn test_server_type_equality() {
    let npm1 = ServerType::Npm {
        package: "test".to_string(),
        version: None,
    };
    let npm2 = ServerType::Npm {
        package: "test".to_string(),
        version: None,
    };
    let npm3 = ServerType::Npm {
        package: "other".to_string(),
        version: None,
    };

    assert_eq!(npm1, npm2);
    assert_ne!(npm1, npm3);
}

#[test]
fn test_config_field_defaults() {
    let field = ConfigField {
        name: "test".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("Test field".to_string()),
        default: Some("default_value".to_string()),
    };

    assert_eq!(field.name, "test");
    assert_eq!(field.description, Some("Test field".to_string()));
    assert_eq!(field.default, Some("default_value".to_string()));
}

// Mock implementation for testing the trait
struct MockServer {
    metadata: ServerMetadata,
}

impl McpServer for MockServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> anyhow::Result<()> {
        for field in &self.metadata.required_config {
            if !config.contains_key(&field.name) {
                anyhow::bail!("Missing required field: {}", field.name);
            }
        }
        Ok(())
    }

    fn generate_command(&self) -> anyhow::Result<(String, Vec<String>)> {
        match &self.metadata.server_type {
            ServerType::Npm { package, .. } => Ok(("npx".to_string(), vec![package.clone()])),
            _ => Ok(("echo".to_string(), vec!["test".to_string()])),
        }
    }
}

#[test]
fn test_mock_server_trait() {
    let server = MockServer {
        metadata: ServerMetadata {
            name: "test".to_string(),
            description: None,
            server_type: ServerType::Npm {
                package: "test-package".to_string(),
                version: None,
            },
            required_config: vec![ConfigField {
                name: "required".to_string(),
                field_type: ConfigFieldType::String,
                description: None,
                default: None,
            }],
            optional_config: vec![],
        },
    };

    assert_eq!(server.metadata().name, "test");

    let mut config = HashMap::new();
    assert!(server.validate_config(&config).is_err());

    config.insert("required".to_string(), "value".to_string());
    assert!(server.validate_config(&config).is_ok());

    let (cmd, args) = server.generate_command().unwrap();
    assert_eq!(cmd, "npx");
    assert_eq!(args, vec!["test-package"]);
}

#[test]
fn test_detect_docker_with_complex_tag() {
    match detect_server_type("docker:user/repo:v1.2.3") {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "user/repo");
            assert_eq!(tag, "v1.2.3");
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_parse_npm_edge_cases() {
    // Test package name that starts with @ but has no slash (edge case)
    let (pkg, version) = parse_npm_package("@package");
    assert_eq!(pkg, "@package");
    assert_eq!(version, None);
}
