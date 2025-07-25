use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::deps::{Dependency, DependencyCheck, DependencyChecker, DependencyStatus};
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{
    detect_server_type, ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

// Mock client for testing
#[derive(Clone)]
struct MockClient {
    name: String,
    installed: bool,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
}

impl MockClient {
    fn new(name: &str, installed: bool) -> Self {
        Self {
            name: name.to_string(),
            installed,
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        PathBuf::from(format!("/mock/{}/config.json", self.name))
    }

    fn is_installed(&self) -> bool {
        self.installed
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> anyhow::Result<()> {
        self.servers
            .lock()
            .unwrap()
            .insert(name.to_string(), config);
        Ok(())
    }

    fn list_servers(&self) -> anyhow::Result<HashMap<String, ServerConfig>> {
        Ok(self.servers.lock().unwrap().clone())
    }
}

// Mock server for testing
#[allow(dead_code)]
struct MockServer {
    metadata: ServerMetadata,
    dependency_satisfied: bool,
}

impl McpServer for MockServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, _config: &HashMap<String, String>) -> anyhow::Result<()> {
        Ok(())
    }

    fn generate_command(&self) -> anyhow::Result<(String, Vec<String>)> {
        Ok(("mock".to_string(), vec!["--stdio".to_string()]))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        Box::new(MockDependencyChecker {
            satisfied: self.dependency_satisfied,
        })
    }
}

// Mock dependency checker
#[allow(dead_code)]
struct MockDependencyChecker {
    satisfied: bool,
}

impl DependencyChecker for MockDependencyChecker {
    fn check(&self) -> anyhow::Result<DependencyCheck> {
        Ok(DependencyCheck {
            dependency: Dependency::NodeJs {
                min_version: Some("16.0.0".to_string()),
            },
            status: if self.satisfied {
                DependencyStatus::Installed {
                    version: Some("18.0.0".to_string()),
                }
            } else {
                DependencyStatus::Missing
            },
            install_instructions: None,
        })
    }
}

#[test]
fn test_install_command_creation() {
    let _install = InstallCommand::new(false);
    // Should create successfully
    assert!(true); // Basic smoke test
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
            assert_eq!(tag, "latest");
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
    let client = MockClient::new("test-client", true);
    let config = ServerConfig {
        command: "npx".to_string(),
        args: vec!["test-server".to_string()],
        env: HashMap::new(),
    };

    client.add_server("test-server", config.clone()).unwrap();

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
        DependencyStatus::Missing => assert!(true),
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
