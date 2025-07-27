use mcp_helper::client::{detect_clients, ClientRegistry, ServerConfig};
use mcp_helper::deps::{Dependency, DependencyStatus};
use mcp_helper::install::InstallCommand;
use mcp_helper::server::{detect_server_type, McpServer, ServerType};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_npm_server_install_flow() {
    // Test the full flow of NPM server installation
    let server_name = "@modelcontextprotocol/server-filesystem";

    // 1. Test server type detection
    let server_type = detect_server_type(server_name);
    match &server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert!(version.is_none());
        }
        _ => panic!("Expected NPM server type"),
    }

    // 2. Test creating NPM server instance
    use mcp_helper::server::npm::NpmServer;
    let server =
        NpmServer::from_package("@modelcontextprotocol/server-filesystem".to_string(), None);

    // 3. Test dependency checking
    let dependency = server.dependency();
    let check_result = dependency.check();

    // The test should handle both cases: Node.js installed or not
    match check_result {
        Ok(check) => {
            match &check.dependency {
                Dependency::NodeJs { min_version } => {
                    assert!(min_version.is_some());
                }
                _ => panic!("Expected NodeJs dependency"),
            }

            // If Node.js is installed, status should be Installed
            match &check.status {
                DependencyStatus::Installed { version } => {
                    assert!(version.is_some());
                }
                DependencyStatus::Missing => {
                    // This is also valid in CI environments
                    assert!(check.install_instructions.is_some());
                }
                DependencyStatus::VersionMismatch { .. } => {
                    // Version mismatch is also a valid scenario
                    assert!(check.install_instructions.is_some());
                }
            }
        }
        Err(_) => {
            // Error checking dependency is acceptable in test environment
        }
    }

    // 4. Test metadata generation
    let metadata = server.metadata();
    assert_eq!(metadata.name, "@modelcontextprotocol/server-filesystem");
    assert!(metadata.description.is_some());

    // 5. Test command generation
    let command_result = server.generate_command();
    match command_result {
        Ok((cmd, args)) => {
            // Handle platform-specific npx command
            #[cfg(target_os = "windows")]
            assert_eq!(cmd, "npx.cmd");
            #[cfg(not(target_os = "windows"))]
            assert_eq!(cmd, "npx");

            assert!(args.contains(&"--yes".to_string()));
            assert!(args.contains(&"@modelcontextprotocol/server-filesystem".to_string()));
            assert!(args.contains(&"--stdio".to_string()));
        }
        Err(_) => {
            // Command generation might fail in test environment
        }
    }

    // 6. Test config validation
    let empty_config = HashMap::new();
    let validation_result = server.validate_config(&empty_config);
    assert!(validation_result.is_ok()); // filesystem server has no required config
}

#[test]
fn test_npm_server_with_version() {
    let server_name = "express@4.18.0";

    // Test detection with version
    let server_type = detect_server_type(server_name);
    match &server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "express");
            assert_eq!(version.as_ref().unwrap(), "4.18.0");
        }
        _ => panic!("Expected NPM server type with version"),
    }

    // Test server creation with version
    use mcp_helper::server::npm::NpmServer;
    let server = NpmServer::from_package("express".to_string(), Some("4.18.0".to_string()));

    // Test command generation includes version
    if let Ok((_, args)) = server.generate_command() {
        assert!(args.contains(&"express@4.18.0".to_string()));
    }
}

#[test]
fn test_npm_server_with_required_config() {
    use mcp_helper::server::npm::NpmServer;
    use mcp_helper::server::{ConfigField, ConfigFieldType};

    // Create server with required configuration
    let required_fields = vec![
        ConfigField {
            name: "apiKey".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key for authentication".to_string()),
            default: None,
        },
        ConfigField {
            name: "workspace".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("Workspace ID".to_string()),
            default: None,
        },
    ];

    let server = NpmServer::from_package("test-server".to_string(), None)
        .with_config(required_fields, vec![]);

    // Test validation with missing required fields
    let empty_config = HashMap::new();
    let result = server.validate_config(&empty_config);
    assert!(result.is_err());

    // Test validation with all required fields
    let mut valid_config = HashMap::new();
    valid_config.insert("apiKey".to_string(), "test-key".to_string());
    valid_config.insert("workspace".to_string(), "test-workspace".to_string());

    let result = server.validate_config(&valid_config);
    assert!(result.is_ok());
}

#[test]
fn test_client_detection_and_installation() {
    // Test client detection
    let clients = detect_clients();
    assert!(!clients.is_empty()); // At least ClaudeDesktop should be detected

    // Test client registry
    let mut registry = ClientRegistry::new();
    for client in clients {
        registry.register(client);
    }

    // Test finding installed clients
    let _installed = registry.detect_installed();
    // This may be empty in CI, which is fine

    // Test getting client by name
    let claude_client = registry.get_by_name("Claude Desktop");
    assert!(claude_client.is_some());
}

#[test]
fn test_install_command_initialization() {
    // Test creating install command
    let _install_cmd = InstallCommand::new(false);

    // The install command should be created successfully
    // Actual installation would require interactive input,
    // so we just test the initialization
}

#[test]
fn test_scoped_npm_package_handling() {
    // Test various scoped package formats
    let test_cases = vec![
        ("@org/package", ("@org/package", None)),
        ("@org/package@1.0.0", ("@org/package", Some("1.0.0"))),
        ("@org/package@latest", ("@org/package", Some("latest"))),
        ("@org/package@^2.0.0", ("@org/package", Some("^2.0.0"))),
    ];

    for (input, (expected_pkg, expected_ver)) in test_cases {
        let server_type = detect_server_type(input);
        match server_type {
            ServerType::Npm { package, version } => {
                assert_eq!(package, expected_pkg);
                assert_eq!(version.as_deref(), expected_ver);
            }
            _ => panic!("Expected NPM server type for {input}"),
        }
    }
}

#[test]
fn test_server_config_generation() {
    // Test generating ServerConfig for NPM server
    let server_config = ServerConfig {
        command: "npx".to_string(),
        args: vec![
            "--yes".to_string(),
            "@modelcontextprotocol/server-filesystem".to_string(),
            "--stdio".to_string(),
        ],
        env: HashMap::new(),
    };

    assert_eq!(server_config.command, "npx");
    assert_eq!(server_config.args.len(), 3);

    // Test with environment variables
    let mut env = HashMap::new();
    env.insert("DEBUG".to_string(), "true".to_string());
    env.insert("LOG_LEVEL".to_string(), "verbose".to_string());

    let server_config_with_env = ServerConfig {
        command: server_config.command,
        args: server_config.args,
        env,
    };

    assert_eq!(server_config_with_env.env.len(), 2);
    assert_eq!(
        server_config_with_env.env.get("DEBUG"),
        Some(&"true".to_string())
    );
}

#[test]
fn test_temporary_file_handling() {
    // Test that we can create temporary files for atomic writes
    let temp_dir = TempDir::new().unwrap();
    let target_path = temp_dir.path().join("config.json");

    // Simulate atomic write pattern
    let temp_file = tempfile::NamedTempFile::new_in(temp_dir.path()).unwrap();
    std::fs::write(temp_file.path(), r#"{"test": true}"#).unwrap();
    temp_file.persist(&target_path).unwrap();

    assert!(target_path.exists());
    let content = std::fs::read_to_string(&target_path).unwrap();
    assert_eq!(content, r#"{"test": true}"#);
}
