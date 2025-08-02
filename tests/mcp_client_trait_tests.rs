//! Comprehensive tests for McpClient trait and core client functionality

use anyhow::Result;
use mcp_helper::client::{
    get_home_with_fallback, detect_clients, ClientRegistry, HomeDirectoryProvider, McpClient,
    RealHomeDirectoryProvider, ServerConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Mock implementation of McpClient for testing trait behavior
#[derive(Clone)]
struct TestMcpClient {
    name: String,
    config_path: PathBuf,
    installed: bool,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
    add_server_should_fail: bool,
    list_servers_should_fail: bool,
}

impl TestMcpClient {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config_path: PathBuf::from(format!("/test/{name}/config.json")),
            installed: true,
            servers: Arc::new(Mutex::new(HashMap::new())),
            add_server_should_fail: false,
            list_servers_should_fail: false,
        }
    }

    fn with_installed(mut self, installed: bool) -> Self {
        self.installed = installed;
        self
    }

    fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = path;
        self
    }

    fn with_add_server_failure(mut self) -> Self {
        self.add_server_should_fail = true;
        self
    }

    fn with_list_servers_failure(mut self) -> Self {
        self.list_servers_should_fail = true;
        self
    }

    fn with_initial_servers(mut self, servers: HashMap<String, ServerConfig>) -> Self {
        self.servers = Arc::new(Mutex::new(servers));
        self
    }
}

impl McpClient for TestMcpClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn is_installed(&self) -> bool {
        self.installed
    }

    fn add_server(&self, name: &str, config: ServerConfig) -> Result<()> {
        if self.add_server_should_fail {
            anyhow::bail!("Mock add_server failure");
        }

        let mut servers = self.servers.lock().unwrap();
        servers.insert(name.to_string(), config);
        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        if self.list_servers_should_fail {
            anyhow::bail!("Mock list_servers failure");
        }

        let servers = self.servers.lock().unwrap();
        Ok(servers.clone())
    }
}

#[test]
fn test_mcp_client_trait_basic_implementation() {
    let client = TestMcpClient::new("TestClient");

    assert_eq!(client.name(), "TestClient");
    assert_eq!(client.config_path(), PathBuf::from("/test/TestClient/config.json"));
    assert!(client.is_installed());
}

#[test]
fn test_mcp_client_add_and_list_servers() {
    let client = TestMcpClient::new("TestClient");

    // Initially no servers
    let servers = client.list_servers().unwrap();
    assert!(servers.is_empty());

    // Add a server
    let config = ServerConfig {
        command: "node".to_string(),
        args: vec!["server.js".to_string()],
        env: HashMap::new(),
    };
    client.add_server("test-server", config.clone()).unwrap();

    // List should now contain the server
    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains_key("test-server"));
    assert_eq!(servers["test-server"].command, "node");
}

#[test]
fn test_mcp_client_error_handling() {
    // Test add_server failure
    let client = TestMcpClient::new("ErrorClient").with_add_server_failure();
    let config = ServerConfig {
        command: "test".to_string(),
        args: vec![],
        env: HashMap::new(),
    };
    
    let result = client.add_server("test", config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock add_server failure"));

    // Test list_servers failure
    let client = TestMcpClient::new("ErrorClient").with_list_servers_failure();
    let result = client.list_servers();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Mock list_servers failure"));
}

#[test]
fn test_server_config_creation_and_equality() {
    let mut env1 = HashMap::new();
    env1.insert("KEY1".to_string(), "VALUE1".to_string());
    env1.insert("KEY2".to_string(), "VALUE2".to_string());

    let config1 = ServerConfig {
        command: "npx".to_string(),
        args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
        env: env1.clone(),
    };

    let config2 = ServerConfig {
        command: "npx".to_string(),
        args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
        env: env1,
    };

    // Test equality
    assert_eq!(config1, config2);

    // Test inequality
    let config3 = ServerConfig {
        command: "python".to_string(),
        args: vec!["server.py".to_string()],
        env: HashMap::new(),
    };
    assert_ne!(config1, config3);
}

#[test]
fn test_server_config_serialization() {
    let mut env = HashMap::new();
    env.insert("API_KEY".to_string(), "secret".to_string());

    let config = ServerConfig {
        command: "docker".to_string(),
        args: vec!["run".to_string(), "-it".to_string(), "mcp-server".to_string()],
        env,
    };

    // Test serialization
    let json = serde_json::to_string(&config).unwrap();
    assert!(json.contains("\"command\":\"docker\""));
    assert!(json.contains("\"args\":[\"run\",\"-it\",\"mcp-server\"]"));
    assert!(json.contains("\"API_KEY\":\"secret\""));

    // Test deserialization
    let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, config);
}

#[test]
fn test_client_registry_operations() {
    let mut registry = ClientRegistry::new();

    // Test empty registry
    assert_eq!(registry.clients.len(), 0);
    assert!(registry.detect_installed().is_empty());
    assert!(registry.get_by_name("NonExistent").is_none());

    // Add multiple clients
    registry.register(Box::new(TestMcpClient::new("Client1").with_installed(true)));
    registry.register(Box::new(TestMcpClient::new("Client2").with_installed(false)));
    registry.register(Box::new(TestMcpClient::new("Client3").with_installed(true)));

    assert_eq!(registry.clients.len(), 3);

    // Test detect_installed
    let installed = registry.detect_installed();
    assert_eq!(installed.len(), 2);
    let installed_names: Vec<&str> = installed.iter().map(|c| c.name()).collect();
    assert!(installed_names.contains(&"Client1"));
    assert!(installed_names.contains(&"Client3"));
    assert!(!installed_names.contains(&"Client2"));

    // Test get_by_name (case insensitive)
    assert!(registry.get_by_name("Client1").is_some());
    assert!(registry.get_by_name("client1").is_some());
    assert!(registry.get_by_name("CLIENT1").is_some());
    assert!(registry.get_by_name("Client2").is_some());
    assert!(registry.get_by_name("NonExistent").is_none());
}

#[test]
fn test_client_registry_default_trait() {
    let registry1 = ClientRegistry::default();
    let registry2 = ClientRegistry::new();

    // Both should be empty
    assert_eq!(registry1.clients.len(), 0);
    assert_eq!(registry2.clients.len(), 0);
}

#[test]
fn test_home_directory_provider_trait() {
    // Create our own mock provider for testing
    struct TestHomeProvider {
        home_path: PathBuf,
    }
    
    impl HomeDirectoryProvider for TestHomeProvider {
        fn home_dir(&self) -> Option<PathBuf> {
            Some(self.home_path.clone())
        }
    }
    
    // Test mock provider
    let mock_home = PathBuf::from("/mock/home");
    let mock_provider = TestHomeProvider {
        home_path: mock_home.clone(),
    };
    assert_eq!(mock_provider.home_dir(), Some(mock_home));

    // Test real provider (just verify it returns something or None)
    let real_provider = RealHomeDirectoryProvider;
    let _ = real_provider.home_dir(); // Result depends on system
}

#[test]
fn test_get_home_with_fallback() {
    // Create our own mock provider for testing
    struct TestHomeProvider {
        home_path: Option<PathBuf>,
    }
    
    impl HomeDirectoryProvider for TestHomeProvider {
        fn home_dir(&self) -> Option<PathBuf> {
            self.home_path.clone()
        }
    }
    
    // Test with provider that returns a home directory
    let mock_home = PathBuf::from("/test/home");
    let provider = TestHomeProvider {
        home_path: Some(mock_home.clone()),
    };
    assert_eq!(get_home_with_fallback(&provider), mock_home);

    // Test with provider that returns None (simulated)
    struct NoneHomeProvider;
    impl HomeDirectoryProvider for NoneHomeProvider {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    let none_provider = NoneHomeProvider;
    let fallback = get_home_with_fallback(&none_provider);
    // Should return a fallback path (environment variable or ".")
    assert!(!fallback.as_os_str().is_empty());
}

#[test]
fn test_detect_clients_returns_all_client_types() {
    let clients = detect_clients();
    
    // Should return exactly 5 clients
    assert_eq!(clients.len(), 5);
    
    // Verify all expected clients are present
    let client_names: Vec<&str> = clients.iter().map(|c| c.name()).collect();
    assert!(client_names.contains(&"Claude Code"));
    assert!(client_names.contains(&"Claude Desktop"));
    assert!(client_names.contains(&"Cursor"));
    assert!(client_names.contains(&"VS Code"));
    assert!(client_names.contains(&"Windsurf"));
    
    // Verify each client has a unique name
    let mut unique_names = client_names.clone();
    unique_names.sort();
    unique_names.dedup();
    assert_eq!(unique_names.len(), 5);
}

#[test]
fn test_mcp_client_trait_send_sync() {
    // Verify that McpClient trait objects can be sent between threads
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Box<dyn McpClient>>();
    
    // Test actual usage across threads
    let client: Box<dyn McpClient> = Box::new(TestMcpClient::new("ThreadSafeClient"));
    let client = Arc::new(client);
    
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let client = Arc::clone(&client);
            std::thread::spawn(move || {
                assert_eq!(client.name(), "ThreadSafeClient");
                assert!(client.is_installed());
                
                let config = ServerConfig {
                    command: format!("cmd{i}"),
                    args: vec![],
                    env: HashMap::new(),
                };
                let _ = client.add_server(&format!("server{i}"), config);
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_server_config_with_complex_environment() {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), "/usr/local/bin:/usr/bin".to_string());
    env.insert("NODE_ENV".to_string(), "production".to_string());
    env.insert("API_KEY".to_string(), "sk-1234567890".to_string());
    env.insert("EMPTY_VAR".to_string(), String::new());

    let config = ServerConfig {
        command: "node".to_string(),
        args: vec![
            "--experimental-modules".to_string(),
            "server.mjs".to_string(),
            "--port".to_string(),
            "3000".to_string(),
        ],
        env: env.clone(),
    };

    assert_eq!(config.env.len(), 4);
    assert_eq!(config.env.get("PATH"), Some(&"/usr/local/bin:/usr/bin".to_string()));
    assert_eq!(config.env.get("NODE_ENV"), Some(&"production".to_string()));
    assert_eq!(config.env.get("EMPTY_VAR"), Some(&String::new()));
}

#[test]
fn test_client_with_multiple_servers() {
    let client = TestMcpClient::new("MultiServerClient");

    // Add multiple servers
    let configs = vec![
        ("filesystem", ServerConfig {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
            env: HashMap::new(),
        }),
        ("github", ServerConfig {
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server-github".to_string()],
            env: {
                let mut env = HashMap::new();
                env.insert("GITHUB_TOKEN".to_string(), "ghp_xxx".to_string());
                env
            },
        }),
        ("python-server", ServerConfig {
            command: "python".to_string(),
            args: vec!["-m".to_string(), "mcp_server".to_string()],
            env: HashMap::new(),
        }),
    ];

    for (name, config) in &configs {
        client.add_server(name, config.clone()).unwrap();
    }

    // Verify all servers are present
    let servers = client.list_servers().unwrap();
    assert_eq!(servers.len(), 3);
    
    for (name, config) in configs {
        assert!(servers.contains_key(name));
        assert_eq!(&servers[name], &config);
    }
}

#[test]
fn test_registry_with_mixed_client_states() {
    let mut registry = ClientRegistry::new();

    // Create clients with various states
    let mut initial_servers = HashMap::new();
    initial_servers.insert("existing".to_string(), ServerConfig {
        command: "test".to_string(),
        args: vec![],
        env: HashMap::new(),
    });

    registry.register(Box::new(
        TestMcpClient::new("InstalledWithServers")
            .with_installed(true)
            .with_initial_servers(initial_servers)
    ));
    
    registry.register(Box::new(
        TestMcpClient::new("InstalledEmpty")
            .with_installed(true)
    ));
    
    registry.register(Box::new(
        TestMcpClient::new("NotInstalled")
            .with_installed(false)
    ));
    
    registry.register(Box::new(
        TestMcpClient::new("CustomPath")
            .with_installed(true)
            .with_config_path(PathBuf::from("/custom/path/config.json"))
    ));

    // Test various queries
    assert_eq!(registry.clients.len(), 4);
    
    let installed = registry.detect_installed();
    assert_eq!(installed.len(), 3);
    
    // Test specific client properties
    let client_with_servers = registry.get_by_name("InstalledWithServers").unwrap();
    let servers = client_with_servers.list_servers().unwrap();
    assert_eq!(servers.len(), 1);
    assert!(servers.contains_key("existing"));
    
    let custom_path_client = registry.get_by_name("CustomPath").unwrap();
    assert_eq!(custom_path_client.config_path(), PathBuf::from("/custom/path/config.json"));
}

#[test]
#[cfg(windows)]
fn test_get_home_with_fallback_windows() {
    use std::env;
    
    struct NoneHomeProvider;
    impl HomeDirectoryProvider for NoneHomeProvider {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    let original_userprofile = env::var("USERPROFILE").ok();
    let original_home = env::var("HOME").ok();

    // Test USERPROFILE fallback
    env::set_var("USERPROFILE", "C:\\Users\\TestUser");
    env::remove_var("HOME");
    
    let provider = NoneHomeProvider;
    assert_eq!(get_home_with_fallback(&provider), PathBuf::from("C:\\Users\\TestUser"));

    // Test HOME fallback when USERPROFILE is not set
    env::remove_var("USERPROFILE");
    env::set_var("HOME", "C:\\Home\\TestUser");
    assert_eq!(get_home_with_fallback(&provider), PathBuf::from("C:\\Home\\TestUser"));

    // Test ultimate fallback
    env::remove_var("USERPROFILE");
    env::remove_var("HOME");
    assert_eq!(get_home_with_fallback(&provider), PathBuf::from("."));

    // Restore original values
    if let Some(val) = original_userprofile {
        env::set_var("USERPROFILE", val);
    }
    if let Some(val) = original_home {
        env::set_var("HOME", val);
    }
}

#[test]
#[cfg(not(windows))]
fn test_get_home_with_fallback_unix() {
    use std::env;
    
    struct NoneHomeProvider;
    impl HomeDirectoryProvider for NoneHomeProvider {
        fn home_dir(&self) -> Option<PathBuf> {
            None
        }
    }

    let original_home = env::var("HOME").ok();

    // Test HOME fallback
    env::set_var("HOME", "/home/testuser");
    let provider = NoneHomeProvider;
    assert_eq!(get_home_with_fallback(&provider), PathBuf::from("/home/testuser"));

    // Test ultimate fallback
    env::remove_var("HOME");
    assert_eq!(get_home_with_fallback(&provider), PathBuf::from("."));

    // Restore original value
    if let Some(val) = original_home {
        env::set_var("HOME", val);
    }
}