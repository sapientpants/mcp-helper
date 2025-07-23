use mcp_helper::client::{ClientRegistry, McpClient, ServerConfig};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone)]
struct MockClient {
    name: String,
    installed: bool,
    config_path: PathBuf,
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn is_installed(&self) -> bool {
        self.installed
    }

    fn add_server(&self, _name: &str, _config: ServerConfig) -> anyhow::Result<()> {
        Ok(())
    }

    fn list_servers(&self) -> anyhow::Result<HashMap<String, ServerConfig>> {
        let mut servers = HashMap::new();
        servers.insert(
            "test-server".to_string(),
            ServerConfig {
                command: "test".to_string(),
                args: vec!["arg1".to_string()],
                env: HashMap::new(),
            },
        );
        Ok(servers)
    }
}

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
    let client = Box::new(MockClient {
        name: "test".to_string(),
        installed: true,
        config_path: PathBuf::from("/test/path"),
    });

    registry.register(client);
    assert_eq!(registry.clients.len(), 1);
}

#[test]
fn test_client_registry_detect_installed() {
    let mut registry = ClientRegistry::new();

    registry.register(Box::new(MockClient {
        name: "installed".to_string(),
        installed: true,
        config_path: PathBuf::from("/installed"),
    }));

    registry.register(Box::new(MockClient {
        name: "not_installed".to_string(),
        installed: false,
        config_path: PathBuf::from("/not_installed"),
    }));

    let installed = registry.detect_installed();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].name(), "installed");
}

#[test]
fn test_client_registry_get_by_name() {
    let mut registry = ClientRegistry::new();

    registry.register(Box::new(MockClient {
        name: "Claude".to_string(),
        installed: true,
        config_path: PathBuf::from("/claude"),
    }));

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
    let client = MockClient {
        name: "test".to_string(),
        installed: true,
        config_path: PathBuf::from("/test"),
    };

    assert_eq!(client.name(), "test");
    assert_eq!(client.config_path(), PathBuf::from("/test"));
    assert!(client.is_installed());

    let config = ServerConfig {
        command: "cmd".to_string(),
        args: vec![],
        env: HashMap::new(),
    };
    assert!(client.add_server("server", config).is_ok());

    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains_key("test-server"));
}

#[test]
fn test_detect_clients() {
    let clients = mcp_helper::client::detect_clients();
    assert_eq!(clients.len(), 0); // No clients registered yet
}
