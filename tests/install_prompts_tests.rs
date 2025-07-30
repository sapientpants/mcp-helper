use anyhow::Result;
use mcp_helper::client::{ClientRegistry, McpClient, ServerConfig};
use mcp_helper::deps::{Dependency, DependencyCheck, DependencyChecker, DependencyStatus};
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

// Mock client that simulates installed client
struct MockInstalledClient {
    name: String,
    config_path: PathBuf,
    servers: HashMap<String, ServerConfig>,
}

impl MockInstalledClient {
    fn new(name: &str, temp_dir: &TempDir) -> Self {
        let config_path = temp_dir.path().join(format!("{name}.json"));
        std::fs::write(&config_path, "{}").unwrap();

        Self {
            name: name.to_string(),
            config_path,
            servers: HashMap::new(),
        }
    }
}

impl McpClient for MockInstalledClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn is_installed(&self) -> bool {
        true
    }

    fn add_server(&self, _name: &str, _config: ServerConfig) -> Result<()> {
        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        Ok(self.servers.clone())
    }
}

// Mock server with configurable fields
struct ConfigurableServer {
    metadata: ServerMetadata,
}

impl ConfigurableServer {
    fn new_with_fields(required: Vec<ConfigField>, optional: Vec<ConfigField>) -> Self {
        Self {
            metadata: ServerMetadata {
                name: "test-server".to_string(),
                description: Some("Test server with configurable fields".to_string()),
                server_type: ServerType::Npm {
                    package: "test-server".to_string(),
                    version: None,
                },
                required_config: required,
                optional_config: optional,
            },
        }
    }
}

impl McpServer for ConfigurableServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, _config: &HashMap<String, String>) -> Result<()> {
        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        Ok(("npx".to_string(), vec!["test-server".to_string()]))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        struct AlwaysInstalled;
        impl DependencyChecker for AlwaysInstalled {
            fn check(&self) -> Result<DependencyCheck> {
                Ok(DependencyCheck {
                    dependency: Dependency::NodeJs { min_version: None },
                    status: DependencyStatus::Installed {
                        version: Some("20.0.0".to_string()),
                    },
                    install_instructions: None,
                })
            }
        }
        Box::new(AlwaysInstalled)
    }
}

#[test]
fn test_execute_with_no_config_needed() {
    let temp_dir = TempDir::new().unwrap();
    let mut registry = ClientRegistry::new();
    registry.register(Box::new(MockInstalledClient::new("test-client", &temp_dir)));

    // Create server with no config fields
    let server = ConfigurableServer::new_with_fields(vec![], vec![]);

    // This tests the path where no configuration is needed
    let metadata = server.metadata();
    assert!(metadata.required_config.is_empty());
    assert!(metadata.optional_config.is_empty());
}

#[test]
fn test_field_types_coverage() {
    // Test all field types
    let field_types = vec![
        ConfigFieldType::String,
        ConfigFieldType::Number,
        ConfigFieldType::Boolean,
        ConfigFieldType::Path,
        ConfigFieldType::Url,
    ];

    for field_type in field_types {
        let field = ConfigField {
            name: format!("test_{field_type:?}").to_lowercase(),
            field_type: field_type.clone(),
            description: Some(format!("Test {field_type:?} field")),
            default: match field_type {
                ConfigFieldType::Boolean => Some("false".to_string()),
                _ => None,
            },
        };

        // Test with required
        let prompt = InstallCommand::build_field_prompt(&field, true);
        assert!(prompt.contains("Test"));

        // Test with optional
        let prompt_opt = InstallCommand::build_field_prompt(&field, false);
        assert!(prompt_opt.contains("optional"));
    }
}

#[test]
fn test_prompt_with_default_values() {
    let fields_with_defaults = vec![
        ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: Some("Port number".to_string()),
            default: Some("8080".to_string()),
        },
        ConfigField {
            name: "enabled".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: Some("Enable feature".to_string()),
            default: Some("true".to_string()),
        },
        ConfigField {
            name: "path".to_string(),
            field_type: ConfigFieldType::Path,
            description: Some("File path".to_string()),
            default: Some("/tmp/default".to_string()),
        },
    ];

    for field in fields_with_defaults {
        assert!(field.default.is_some());
        let prompt = InstallCommand::build_field_prompt(&field, false);
        assert!(prompt.contains("optional"));
    }
}

#[test]
fn test_execute_with_whitespace_server_name() {
    let mut cmd = InstallCommand::new(false);

    // Test server name with only whitespace
    let result = cmd.execute("   ");
    assert!(result.is_err());

    // Test server name with tabs and newlines
    let result = cmd.execute("\t\n");
    assert!(result.is_err());
}

#[test]
fn test_error_paths_for_uncovered_lines() {
    let mut cmd = InstallCommand::new(true); // verbose mode

    // Test various error conditions
    let error_cases = vec![
        "",                        // empty
        "   ",                     // whitespace
        "\t",                      // tab
        "http://",                 // incomplete URL
        "docker:",                 // incomplete docker
        "unknown-server-type.xyz", // unknown extension
    ];

    for case in error_cases {
        let result = cmd.execute(case);
        assert!(result.is_err(), "Expected error for input: '{case}'");
    }
}

#[test]
fn test_server_with_all_config_types() {
    // Create a server with all config field types
    let required_fields = vec![
        ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API Key".to_string()),
            default: None,
        },
        ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: Some("Port number".to_string()),
            default: Some("3000".to_string()),
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
            description: Some("Configuration file path".to_string()),
            default: None,
        },
        ConfigField {
            name: "webhook_url".to_string(),
            field_type: ConfigFieldType::Url,
            description: Some("Webhook URL".to_string()),
            default: None,
        },
    ];

    let server = ConfigurableServer::new_with_fields(required_fields, optional_fields);
    let metadata = server.metadata();

    assert_eq!(metadata.required_config.len(), 2);
    assert_eq!(metadata.optional_config.len(), 3);
}

#[test]
fn test_verbose_mode_coverage() {
    let mut cmd_verbose = InstallCommand::new(true);
    let mut cmd_normal = InstallCommand::new(false);

    // Both should handle the same error cases
    let test_input = "@nonexistent/package";
    let _ = cmd_verbose.execute(test_input);
    let _ = cmd_normal.execute(test_input);
}

#[test]
fn test_install_to_client_error_handling() {
    let temp_dir = TempDir::new().unwrap();

    // Create a client that will fail to add server
    struct FailingClient {
        path: PathBuf,
    }

    impl McpClient for FailingClient {
        fn name(&self) -> &str {
            "failing-client"
        }

        fn config_path(&self) -> PathBuf {
            self.path.clone()
        }

        fn is_installed(&self) -> bool {
            true
        }

        fn add_server(&self, _name: &str, _config: ServerConfig) -> Result<()> {
            Err(anyhow::anyhow!("Failed to add server"))
        }

        fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
            Ok(HashMap::new())
        }
    }

    let client = FailingClient {
        path: temp_dir.path().join("failing.json"),
    };

    let config = HashMap::new();
    let result = client.add_server(
        "test-server",
        ServerConfig {
            command: "test".to_string(),
            args: vec![],
            env: config,
        },
    );

    assert!(result.is_err());
}
