use mcp_helper::client::{ClientRegistry, McpClient, ServerConfig};
use mcp_helper::deps::{Dependency, DependencyCheck, DependencyStatus, InstallInstructions};
use mcp_helper::error::McpError;
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{ConfigField, ConfigFieldType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Mock client for testing
#[derive(Clone)]
#[allow(dead_code)]
struct MockClient {
    name: String,
    is_installed: bool,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
}

impl MockClient {
    fn new(name: &str, is_installed: bool) -> Self {
        Self {
            name: name.to_string(),
            is_installed,
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[allow(dead_code)]
    fn get_servers(&self) -> HashMap<String, ServerConfig> {
        self.servers.lock().unwrap().clone()
    }
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_installed(&self) -> bool {
        self.is_installed
    }

    fn config_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("/tmp/test_{}.json", self.name))
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

#[test]
fn test_install_command_new() {
    let cmd = InstallCommand::new(false);
    // Just test that it creates successfully
    let _ = cmd;

    let verbose_cmd = InstallCommand::new(true);
    // Just test that it creates successfully
    let _ = verbose_cmd;
}

#[test]
fn test_get_dependency_name() {
    assert_eq!(
        InstallCommand::get_dependency_name(&Dependency::NodeJs { min_version: None }),
        "Node.js"
    );
    assert_eq!(
        InstallCommand::get_dependency_name(&Dependency::Python {
            min_version: Some("3.9".to_string())
        }),
        "Python"
    );
    assert_eq!(
        InstallCommand::get_dependency_name(&Dependency::Docker),
        "Docker"
    );
    assert_eq!(InstallCommand::get_dependency_name(&Dependency::Git), "Git");
}

#[test]
fn test_handle_installed_dependency() {
    // Test without version
    let result = InstallCommand::handle_installed_dependency("Node.js", &None);
    assert!(result.is_ok());

    // Test with version
    let result = InstallCommand::handle_installed_dependency("Python", &Some("3.9.0".to_string()));
    assert!(result.is_ok());
}

#[test]
fn test_execute_with_binary_server() {
    let cmd = InstallCommand::new(false);
    // Binary servers are not yet supported
    let result = cmd.execute("https://example.com/binary.tar.gz");
    assert!(result.is_err());
}

#[test]
fn test_build_field_prompt() {
    let field = ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API key for authentication".to_string()),
        default: None,
    };

    // Test required field with description
    let prompt = InstallCommand::build_field_prompt(&field, true);
    assert_eq!(prompt, "API key for authentication");

    // Test optional field with description
    let prompt = InstallCommand::build_field_prompt(&field, false);
    assert_eq!(prompt, "API key for authentication (optional)");

    // Test field without description
    let field_no_desc = ConfigField {
        name: "token".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };

    let prompt = InstallCommand::build_field_prompt(&field_no_desc, true);
    assert_eq!(prompt, "token");

    let prompt = InstallCommand::build_field_prompt(&field_no_desc, false);
    assert_eq!(prompt, "token (optional)");
}

#[test]
fn test_handle_missing_dependency_with_instructions() {
    let dep_check = DependencyCheck {
        dependency: Dependency::NodeJs {
            min_version: Some("16.0.0".to_string()),
        },
        status: DependencyStatus::Missing,
        install_instructions: Some(InstallInstructions {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        }),
    };

    let result = InstallCommand::handle_missing_dependency("Node.js", &dep_check);
    assert!(result.is_err());
    match result.unwrap_err() {
        McpError::MissingDependency {
            dependency,
            required_version,
            ..
        } => {
            assert_eq!(dependency, "Node.js");
            assert_eq!(required_version, Some("16.0.0".to_string()));
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_handle_missing_dependency_without_instructions() {
    let dep_check = DependencyCheck {
        dependency: Dependency::Docker,
        status: DependencyStatus::Missing,
        install_instructions: None,
    };

    let result = InstallCommand::handle_missing_dependency("Docker", &dep_check);
    assert!(result.is_err());
    match result.unwrap_err() {
        McpError::Other(err) => {
            assert!(err.to_string().contains("Docker is not installed"));
        }
        _ => panic!("Expected Other error"),
    }
}

#[test]
fn test_execute_no_clients() {
    let cmd = InstallCommand::new(false);
    let result = cmd.execute("@test/package");

    // Should fail because no clients are detected in test environment
    assert!(result.is_err());
}

#[test]
fn test_execute_npm_server() {
    let cmd = InstallCommand::new(false);

    // Test that NPM packages can be handled
    let result = cmd.execute("@scope/package");
    // Should fail due to no clients in test environment
    assert!(result.is_err());

    // Test package with version
    let result = cmd.execute("express@4.18.0");
    // Should fail due to no clients in test environment
    assert!(result.is_err());
}

#[test]
fn test_field_prompt_variations() {
    // Test all combinations of description and required
    let test_cases = vec![
        (Some("Test description"), true, "Test description"),
        (
            Some("Test description"),
            false,
            "Test description (optional)",
        ),
        (None, true, "field_name"),
        (None, false, "field_name (optional)"),
    ];

    for (desc, required, expected) in test_cases {
        let field = ConfigField {
            name: "field_name".to_string(),
            field_type: ConfigFieldType::String,
            description: desc.map(|s| s.to_string()),
            default: None,
        };
        assert_eq!(
            InstallCommand::build_field_prompt(&field, required),
            expected
        );
    }
}

#[test]
fn test_dependency_status_handling() {
    // Test installed dependency
    let result = InstallCommand::handle_installed_dependency("Git", &Some("2.40.0".to_string()));
    assert!(result.is_ok());

    // Test version mismatch scenario
    let _dep_check = DependencyCheck {
        dependency: Dependency::NodeJs {
            min_version: Some("18.0.0".to_string()),
        },
        status: DependencyStatus::VersionMismatch {
            installed: "16.0.0".to_string(),
            required: "18.0.0".to_string(),
        },
        install_instructions: Some(InstallInstructions {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        }),
    };

    // This should be tested through check_dependencies method
    // but we can't easily test it without mocking the dependency checker
}

// Integration test helpers
#[allow(dead_code)]
mod integration_helpers {
    use super::*;

    pub fn create_test_registry_with_clients() -> ClientRegistry {
        let mut registry = ClientRegistry::new();
        registry.register(Box::new(MockClient::new("test-client-1", true)));
        registry.register(Box::new(MockClient::new("test-client-2", true)));
        registry.register(Box::new(MockClient::new("test-client-3", false))); // Not installed
        registry
    }

    pub fn create_test_server_config() -> ServerConfig {
        ServerConfig {
            command: "npx".to_string(),
            args: vec!["--yes".to_string(), "test-server".to_string()],
            env: HashMap::new(),
        }
    }
}
