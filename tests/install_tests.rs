use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::deps::DependencyStatus;
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{
    detect_server_type, ConfigField, ConfigFieldType, ServerMetadata, ServerType,
};
use mcp_helper::test_utils::mocks::MockClientBuilder;
use std::collections::HashMap;

#[test]
fn test_install_command_creation() {
    let _install = InstallCommand::new(false);
    // Should create successfully
    // Basic smoke test - just ensure it creates successfully
}

#[test]
fn test_detect_server_type_npm() {
    let server_type = detect_server_type("@modelcontextprotocol/server-filesystem");
    match server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert!(version.is_none());
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_detect_server_type_npm_with_version() {
    let server_type = detect_server_type("express@4.18.0");
    match server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "express");
            assert_eq!(version, Some("4.18.0".to_string()));
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_detect_server_type_binary() {
    let server_type =
        detect_server_type("https://github.com/example/server/releases/latest/server.tar.gz");
    match server_type {
        ServerType::Binary { url, checksum } => {
            assert_eq!(
                url,
                "https://github.com/example/server/releases/latest/server.tar.gz"
            );
            assert!(checksum.is_none());
        }
        _ => panic!("Expected Binary server type"),
    }
}

#[test]
fn test_detect_server_type_docker() {
    let server_type = detect_server_type("docker:example/server:latest");
    match server_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "example/server");
            assert_eq!(tag, Some("latest".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_detect_server_type_python() {
    let server_type = detect_server_type("server.py");
    match server_type {
        ServerType::Python { package, version } => {
            assert_eq!(package, "server.py");
            assert!(version.is_none());
        }
        _ => panic!("Expected Python server type"),
    }
}

#[test]
fn test_server_metadata_required_config() {
    let metadata = ServerMetadata {
        name: "test-server".to_string(),
        description: Some("Test server".to_string()),
        server_type: ServerType::Npm {
            package: "test-server".to_string(),
            version: None,
        },
        required_config: vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key for authentication".to_string()),
            default: None,
        }],
        optional_config: vec![ConfigField {
            name: "timeout".to_string(),
            field_type: ConfigFieldType::Number,
            description: Some("Request timeout in seconds".to_string()),
            default: Some("30".to_string()),
        }],
    };

    assert_eq!(metadata.required_config.len(), 1);
    assert_eq!(metadata.optional_config.len(), 1);
    assert_eq!(metadata.required_config[0].name, "api_key");
    assert_eq!(metadata.optional_config[0].name, "timeout");
}

#[test]
fn test_config_field_types() {
    let string_field = ConfigField {
        name: "name".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };

    let number_field = ConfigField {
        name: "port".to_string(),
        field_type: ConfigFieldType::Number,
        description: None,
        default: Some("8080".to_string()),
    };

    let bool_field = ConfigField {
        name: "debug".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: None,
        default: Some("false".to_string()),
    };

    let path_field = ConfigField {
        name: "config_path".to_string(),
        field_type: ConfigFieldType::Path,
        description: None,
        default: None,
    };

    let url_field = ConfigField {
        name: "endpoint".to_string(),
        field_type: ConfigFieldType::Url,
        description: None,
        default: None,
    };

    assert_eq!(string_field.field_type, ConfigFieldType::String);
    assert_eq!(number_field.field_type, ConfigFieldType::Number);
    assert_eq!(bool_field.field_type, ConfigFieldType::Boolean);
    assert_eq!(path_field.field_type, ConfigFieldType::Path);
    assert_eq!(url_field.field_type, ConfigFieldType::Url);
}

#[test]
fn test_mock_client_add_server() {
    let config = ServerConfig {
        command: "npx".to_string(),
        args: vec!["test-server".to_string()],
        env: HashMap::new(),
    };

    let client = MockClientBuilder::new("test-client")
        .with_server("test-server", config.clone())
        .build();

    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains_key("test-server"));
    assert_eq!(servers["test-server"].command, "npx");
}

#[test]
fn test_dependency_status_variants() {
    let installed = DependencyStatus::Installed {
        version: Some("1.0.0".to_string()),
    };

    let missing = DependencyStatus::Missing;

    let mismatch = DependencyStatus::VersionMismatch {
        installed: "1.0.0".to_string(),
        required: "2.0.0".to_string(),
    };

    match installed {
        DependencyStatus::Installed { version } => {
            assert_eq!(version, Some("1.0.0".to_string()));
        }
        _ => panic!("Expected Installed variant"),
    }

    match missing {
        DependencyStatus::Missing => {}
        _ => panic!("Expected Missing variant"),
    }

    match mismatch {
        DependencyStatus::VersionMismatch {
            installed,
            required,
        } => {
            assert_eq!(installed, "1.0.0");
            assert_eq!(required, "2.0.0");
        }
        _ => panic!("Expected VersionMismatch variant"),
    }
}
