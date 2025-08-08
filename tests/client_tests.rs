use mcp_helper::client::{ClientRegistry, McpClient, ServerConfig};
use mcp_helper::test_utils::mocks::MockClientBuilder;
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_server_config_creation() {
    let mut env = HashMap::new();
    env.insert("KEY".to_string(), "VALUE".to_string());

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env,
    };

    assert_eq!(config.command, "node");
    assert_eq!(config.args, vec!["server.js"]);
    assert_eq!(config.env.get("KEY"), Some(&"VALUE".to_string()));
}

#[test]
fn test_client_registry_new() {
    let registry = ClientRegistry::new();
    assert_eq!(registry.clients.len(), 0);
}

#[test]
fn test_client_registry_register() {
    let mut registry = ClientRegistry::new();
    let client = Box::new(
        MockClientBuilder::new("test")
            .with_config_path("/test/path")
            .build(),
    );

    registry.register(client);
    assert_eq!(registry.clients.len(), 1);
}

#[test]
fn test_client_registry_detect_installed() {
    let mut registry = ClientRegistry::new();

    registry.register(Box::new(
        MockClientBuilder::new("installed")
            .with_config_path("/installed")
            .build(),
    ));

    registry.register(Box::new(
        MockClientBuilder::new("not_installed")
            .with_config_path("/not_installed")
            .not_installed()
            .build(),
    ));

    let installed = registry.detect_installed();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].name(), "installed");
}

#[test]
fn test_client_registry_get_by_name() {
    let mut registry = ClientRegistry::new();

    registry.register(Box::new(
        MockClientBuilder::new("Claude")
            .with_config_path("/claude")
            .build(),
    ));

    assert!(registry.get_by_name("Claude").is_some());
    assert!(registry.get_by_name("claude").is_some()); // Case insensitive
    assert!(registry.get_by_name("CLAUDE").is_some()); // Case insensitive
    assert!(registry.get_by_name("Unknown").is_none());
}

#[test]
fn test_client_registry_default() {
    let registry = ClientRegistry::default();
    assert_eq!(registry.clients.len(), 0);
}

#[test]
fn test_mock_client_trait_methods() {
    let config = ServerConfig {
        command: "cmd".to_string(),
        args: vec![],
        env: HashMap::new(),
    };

    let client = MockClientBuilder::new("test")
        .with_config_path("/test")
        .with_server("test-server", config.clone())
        .build();

    assert_eq!(client.name(), "test");
    assert_eq!(client.config_path(), PathBuf::from("/test"));
    assert!(client.is_installed());

    assert!(client.add_server("server", config).is_ok());

    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains_key("test-server"));
}

#[test]
fn test_detect_clients() {
    let clients = mcp_helper::client::detect_clients();
    assert_eq!(clients.len(), 5); // Now we have 5 clients registered

    let names: Vec<&str> = clients.iter().map(|c| c.name()).collect();
    assert!(names.contains(&"Claude Code"));
    assert!(names.contains(&"Claude Desktop"));
    assert!(names.contains(&"Cursor"));
    assert!(names.contains(&"VS Code"));
    assert!(names.contains(&"Windsurf"));
}
