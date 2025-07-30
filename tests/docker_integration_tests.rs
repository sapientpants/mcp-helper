use assert_cmd::Command;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mcp_helper::client::{McpClient, ServerConfig};
use mcp_helper::deps::DependencyChecker;
use mcp_helper::server::docker::DockerServer;
use mcp_helper::server::McpServer;

// Mock client for testing
struct MockClient {
    name: String,
    servers: Arc<Mutex<HashMap<String, ServerConfig>>>,
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn config_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("/mock/config.json")
    }

    fn is_installed(&self) -> bool {
        true
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
fn test_docker_server_creation() {
    let docker_spec = "nginx:latest";
    let server = DockerServer::new(docker_spec).unwrap();
    let metadata = server.metadata();
    
    assert_eq!(metadata.name, "nginx");
    assert_eq!(metadata.description, Some("Docker MCP server: nginx".to_string()));
    match &metadata.server_type {
        mcp_helper::server::ServerType::Docker { image, tag } => {
            assert_eq!(image, "nginx");
            assert_eq!(tag, &Some("latest".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_docker_server_with_tag() {
    let docker_spec = "redis:6.2-alpine";
    let server = DockerServer::new(docker_spec).unwrap();
    let metadata = server.metadata();
    
    assert_eq!(metadata.name, "redis");
    if let Some(desc) = &metadata.description {
        assert!(desc.contains("redis"));
    }
}

#[test]
fn test_docker_server_command_generation() {
    let docker_spec = "postgres:13";
    let server = DockerServer::new(docker_spec).unwrap();
    
    let mut config = HashMap::new();
    config.insert("environment".to_string(), "POSTGRES_PASSWORD=secret123,POSTGRES_DB=testdb".to_string());
    
    let (command, args) = server.generate_command_with_config(&config).unwrap();
    
    assert_eq!(command, "docker");
    assert!(args.contains(&"run".to_string()));
    assert!(args.contains(&"postgres:13".to_string()));
    
    // Check environment variables are included
    let env_args: Vec<_> = args.iter()
        .enumerate()
        .filter_map(|(i, arg)| {
            if arg == "-e" && i + 1 < args.len() {
                Some(&args[i + 1])
            } else {
                None
            }
        })
        .collect();
    
    assert!(env_args.iter().any(|arg| arg.starts_with("POSTGRES_PASSWORD=")));
    assert!(env_args.iter().any(|arg| arg.starts_with("POSTGRES_DB=")));
}

#[test]
fn test_docker_server_with_volumes() {
    let docker_spec = "mysql:8.0";
    let server = DockerServer::new(docker_spec).unwrap();
    
    let mut config = HashMap::new();
    config.insert("environment".to_string(), "MYSQL_ROOT_PASSWORD=rootpass".to_string());
    config.insert("volumes".to_string(), "/data:/var/lib/mysql,/config:/etc/mysql/conf.d".to_string());
    
    let (_command, args) = server.generate_command_with_config(&config).unwrap();
    
    // Check volume mounts are included
    let volume_args: Vec<_> = args.iter()
        .enumerate()
        .filter_map(|(i, arg)| {
            if arg == "-v" && i + 1 < args.len() {
                Some(&args[i + 1])
            } else {
                None
            }
        })
        .collect();
    
    assert!(volume_args.iter().any(|arg| arg.contains("/data:/var/lib/mysql")));
    assert!(volume_args.iter().any(|arg| arg.contains("/config:/etc/mysql/conf.d")));
}

#[test]
fn test_docker_server_with_ports() {
    let docker_spec = "nginx:alpine";
    let server = DockerServer::new(docker_spec).unwrap();
    
    let mut config = HashMap::new();
    config.insert("ports".to_string(), "8080:80,8443:443".to_string());
    
    let (_command, args) = server.generate_command_with_config(&config).unwrap();
    
    // Check port mappings are included
    let port_args: Vec<_> = args.iter()
        .enumerate()
        .filter_map(|(i, arg)| {
            if arg == "-p" && i + 1 < args.len() {
                Some(&args[i + 1])
            } else {
                None
            }
        })
        .collect();
    
    assert!(port_args.iter().any(|arg| arg.contains("8080:80")));
    assert!(port_args.iter().any(|arg| arg.contains("8443:443")));
}

#[test]
fn test_docker_server_with_resource_limits() {
    let docker_spec = "node:18";
    let server = DockerServer::new(docker_spec).unwrap();
    
    let mut config = HashMap::new();
    config.insert("memory_limit".to_string(), "512m".to_string());
    config.insert("cpu_limit".to_string(), "1.5".to_string());
    
    let (_command, args) = server.generate_command_with_config(&config).unwrap();
    
    // Check resource limits are included
    assert!(args.iter().any(|arg| arg == "--memory=512m"));
    assert!(args.iter().any(|arg| arg == "--cpus=1.5"));
}

#[test]
fn test_docker_server_dependency_check() {
    let docker_spec = "ubuntu:20.04";
    let server = DockerServer::new(docker_spec).unwrap();
    let dependency_checker = server.dependency();
    
    // The check might fail if Docker isn't installed, but we can test that it doesn't panic
    let _result = dependency_checker.check();
}

#[test]
fn test_docker_dependency_checker_creation() {
    use mcp_helper::deps::docker::DockerChecker;
    
    let checker = DockerChecker::new();
    
    // The check might fail if Docker isn't installed, but we can test that it doesn't panic
    let _result = checker.check();
}

#[test]
fn test_docker_server_metadata_fields() {
    let docker_spec = "mongo:latest";
    let server = DockerServer::new(docker_spec).unwrap();
    let metadata = server.metadata();
    
    // Check that Docker-specific configuration fields are present
    let all_fields: Vec<_> = metadata.required_config.iter()
        .chain(metadata.optional_config.iter())
        .collect();
    
    let field_names: Vec<_> = all_fields.iter().map(|f| &f.name).collect();
    
    // Docker servers should have optional configuration for common Docker options
    assert!(field_names.iter().any(|name| name.contains("ports") || name.contains("volumes") || name.contains("environment")));
}

#[test]
fn test_docker_image_pull_simulation() {
    // This test simulates the pull_image functionality without actually pulling
    let docker_spec = "hello-world:latest";
    let server = DockerServer::new(docker_spec).unwrap();
    
    // The pull_image method should exist and be callable
    // Note: We can't test actual pulling without Docker being available,
    // but we can ensure the method exists and handles basic cases
    let result = server.pull_image();
    
    // This will likely fail in CI without Docker, but shouldn't panic
    // The error should be informative
    if result.is_err() {
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("docker") || error_msg.contains("Docker") || 
                error_msg.contains("command not found") || error_msg.contains("not installed"));
    }
}

#[test]
fn test_docker_spec_parsing() {
    // Test various Docker specification formats through DockerServer creation
    let test_cases = vec![
        ("nginx", "nginx", None),
        ("nginx:latest", "nginx", Some("latest".to_string())),
        ("library/nginx:1.21", "library/nginx", Some("1.21".to_string())),
        ("redis:6.2-alpine", "redis", Some("6.2-alpine".to_string())),
    ];
    
    for (input, expected_image, expected_tag) in test_cases {
        let server = DockerServer::new(input).unwrap();
        let metadata = server.metadata();
        
        match &metadata.server_type {
            mcp_helper::server::ServerType::Docker { image, tag } => {
                assert_eq!(image, expected_image, "Failed parsing image from: {}", input);
                assert_eq!(tag, &expected_tag, "Failed parsing tag from: {}", input);
            }
            _ => panic!("Expected Docker server type for input: {}", input),
        }
    }
}

#[test]
fn test_cli_docker_server_installation() {
    // Test the CLI interface for Docker server installation
    let mut cmd = Command::cargo_bin("mcp").unwrap();
    
    cmd.arg("install")
        .arg("docker:nginx:alpine")
        .arg("--config")
        .arg("ports=8080:80")
        .arg("--dry-run");
    
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    // Should recognize Docker server type
    assert!(stdout.contains("Installing") || stderr.contains("Docker") || 
            stdout.contains("docker") || stderr.contains("Detecting"));
}

#[test]
fn test_docker_server_config_validation() {
    let docker_spec = "alpine:latest";
    let server = DockerServer::new(docker_spec).unwrap();
    
    // Test with valid configuration
    let mut valid_config = HashMap::new();
    valid_config.insert("ports".to_string(), "3000:3000".to_string());
    valid_config.insert("environment".to_string(), "ENV_VAR=value".to_string());
    
    let (_command, args) = server.generate_command_with_config(&valid_config).unwrap();
    assert!(args.contains(&"alpine:latest".to_string()));
    
    // Test with empty configuration (should still work)
    let empty_config = HashMap::new();
    let (_command, args) = server.generate_command_with_config(&empty_config).unwrap();
    assert!(args.contains(&"alpine:latest".to_string()));
}

#[test]
fn test_docker_server_integration_with_client() {
    let client = MockClient {
        name: "test-client".to_string(),
        servers: Arc::new(Mutex::new(HashMap::new())),
    };
    
    let server_config = ServerConfig {
        command: "docker".to_string(),
        args: vec!["run".to_string(), "--rm".to_string(), "nginx:alpine".to_string()],
        env: {
            let mut env = HashMap::new();
            env.insert("ports".to_string(), "8080:80".to_string());
            env
        },
    };
    
    // Test adding Docker server to client
    let result = client.add_server("nginx-server", server_config);
    assert!(result.is_ok());
    
    // Verify server was added
    let servers = client.list_servers().unwrap();
    assert!(servers.contains_key("nginx-server"));
    assert_eq!(servers["nginx-server"].command, "docker");
    assert!(servers["nginx-server"].env.contains_key("ports"));
}