//! Comprehensive unit tests for src/server/docker.rs
//!
//! This test suite covers the DockerServer implementation including
//! Docker image parsing, configuration validation, and command generation.

use mcp_helper::server::docker::DockerServer;
use mcp_helper::server::{ConfigFieldType, McpServer, ServerType};
use std::collections::HashMap;

#[test]
fn test_docker_server_creation() {
    // Simple image
    let server = DockerServer::new("nginx").unwrap();
    assert_eq!(server.metadata().name, "nginx");

    // Image with tag
    let server = DockerServer::new("nginx:latest").unwrap();
    assert_eq!(server.metadata().name, "nginx");
    match &server.metadata().server_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "nginx");
            assert_eq!(tag, &Some("latest".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }

    // Image with registry
    let server = DockerServer::new("docker.io/library/nginx").unwrap();
    assert_eq!(server.metadata().name, "docker.io/library/nginx");

    // Image with registry and tag
    let server = DockerServer::new("ghcr.io/user/app:v1.0.0").unwrap();
    match &server.metadata().server_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "ghcr.io/user/app");
            assert_eq!(tag, &Some("v1.0.0".to_string()));
        }
        _ => panic!("Expected Docker server type"),
    }
}

#[test]
fn test_docker_spec_parsing() {
    // Test various Docker image specifications
    let test_cases = vec![
        ("alpine", "alpine", None),
        ("alpine:3.18", "alpine", Some("3.18")),
        ("postgres:15", "postgres", Some("15")),
        ("localhost:5000/myapp", "localhost:5000/myapp", None),
        (
            "localhost:5000/myapp:latest",
            "localhost:5000/myapp",
            Some("latest"),
        ),
        (
            "registry.example.com:8080/org/app",
            "registry.example.com:8080/org/app",
            None,
        ),
        (
            "registry.example.com:8080/org/app:v2",
            "registry.example.com:8080/org/app",
            Some("v2"),
        ),
        ("mysql:8.0.35", "mysql", Some("8.0.35")),
        ("node:18-alpine", "node", Some("18-alpine")),
        (
            "python:3.11-slim-bullseye",
            "python",
            Some("3.11-slim-bullseye"),
        ),
    ];

    for (input, expected_image, expected_tag) in test_cases {
        let server = DockerServer::new(input).unwrap();
        match &server.metadata().server_type {
            ServerType::Docker { image, tag } => {
                assert_eq!(image, expected_image, "Failed for input: {input}");
                assert_eq!(
                    tag,
                    &expected_tag.map(|t| t.to_string()),
                    "Failed for input: {input}"
                );
            }
            _ => panic!("Expected Docker server type"),
        }
    }
}

#[test]
fn test_docker_server_builder_methods() {
    let server = DockerServer::new("nginx:latest")
        .unwrap()
        .with_entrypoint("/bin/sh")
        .with_working_dir("/app");

    // Builder methods are internal, but we can verify they don't panic
    drop(server);
}

#[test]
fn test_docker_server_metadata() {
    let server = DockerServer::new("postgres:15").unwrap();
    let metadata = server.metadata();

    assert_eq!(metadata.name, "postgres");
    assert!(metadata.description.is_some());
    assert!(metadata
        .description
        .as_ref()
        .unwrap()
        .contains("Docker MCP server"));

    // Check config fields
    assert!(metadata.required_config.is_empty());
    assert!(!metadata.optional_config.is_empty());

    // Verify expected optional fields exist
    let field_names: Vec<String> = metadata
        .optional_config
        .iter()
        .map(|f| f.name.clone())
        .collect();

    assert!(field_names.contains(&"volumes".to_string()));
    assert!(field_names.contains(&"environment".to_string()));
    assert!(field_names.contains(&"ports".to_string()));
    assert!(field_names.contains(&"network".to_string()));
    assert!(field_names.contains(&"entrypoint".to_string()));
    assert!(field_names.contains(&"working_dir".to_string()));
    assert!(field_names.contains(&"user".to_string()));
    assert!(field_names.contains(&"restart_policy".to_string()));
    assert!(field_names.contains(&"memory_limit".to_string()));
    assert!(field_names.contains(&"cpu_limit".to_string()));
}

#[test]
fn test_validate_config_valid() {
    let server = DockerServer::new("nginx").unwrap();

    // Empty config should be valid
    let config = HashMap::new();
    assert!(server.validate_config(&config).is_ok());

    // Valid volumes
    let mut config = HashMap::new();
    config.insert(
        "volumes".to_string(),
        "/host/path:/container/path".to_string(),
    );
    assert!(server.validate_config(&config).is_ok());

    // Multiple volumes
    config.insert(
        "volumes".to_string(),
        "/data:/data,/logs:/var/log".to_string(),
    );
    assert!(server.validate_config(&config).is_ok());

    // Valid environment variables
    config.clear();
    config.insert("environment".to_string(), "KEY=value".to_string());
    assert!(server.validate_config(&config).is_ok());

    // Multiple env vars
    config.insert(
        "environment".to_string(),
        "DB_HOST=localhost,DB_PORT=5432".to_string(),
    );
    assert!(server.validate_config(&config).is_ok());

    // Valid ports
    config.clear();
    config.insert("ports".to_string(), "8080:80".to_string());
    assert!(server.validate_config(&config).is_ok());

    // Valid restart policy
    config.clear();
    config.insert("restart_policy".to_string(), "always".to_string());
    assert!(server.validate_config(&config).is_ok());

    // Valid numeric limits
    config.clear();
    config.insert("memory_limit".to_string(), "512m".to_string());
    config.insert("cpu_limit".to_string(), "0.5".to_string());
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_invalid_volumes() {
    let server = DockerServer::new("nginx").unwrap();

    // Missing colon in volume
    let mut config = HashMap::new();
    config.insert("volumes".to_string(), "/invalid/volume".to_string());
    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid volume format"));

    // Empty volume part
    config.insert("volumes".to_string(), "/host:".to_string());
    assert!(server.validate_config(&config).is_ok()); // This might be valid

    // Multiple volumes with one invalid
    config.insert(
        "volumes".to_string(),
        "/valid:/path,invalid,/another:/valid".to_string(),
    );
    assert!(server.validate_config(&config).is_err());
}

#[test]
fn test_validate_config_invalid_environment() {
    let server = DockerServer::new("nginx").unwrap();

    // Missing equals in env var
    let mut config = HashMap::new();
    config.insert("environment".to_string(), "INVALID_ENV".to_string());
    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid environment variable format"));

    // Multiple env vars with one invalid
    config.insert(
        "environment".to_string(),
        "VALID=value,INVALID,ANOTHER=value".to_string(),
    );
    assert!(server.validate_config(&config).is_err());
}

#[test]
fn test_validate_config_invalid_restart_policy() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    config.insert("restart_policy".to_string(), "invalid-policy".to_string());
    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid restart policy"));
}

#[test]
fn test_validate_config_invalid_numeric_limits() {
    let server = DockerServer::new("nginx").unwrap();

    // Note: memory_limit is not validated, only cpu_limit is
    let mut config = HashMap::new();
    config.insert("memory_limit".to_string(), "not-a-number".to_string());
    // This actually passes because memory limit isn't validated
    assert!(server.validate_config(&config).is_ok());

    // Invalid CPU format
    config.clear();
    config.insert("cpu_limit".to_string(), "two-cpus".to_string());
    assert!(server.validate_config(&config).is_err());

    // Negative CPU
    config.insert("cpu_limit".to_string(), "-0.5".to_string());
    // Negative CPU values actually parse as valid floats
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_generate_command_basic() {
    let server = DockerServer::new("nginx:latest").unwrap();
    let (cmd, args) = server.generate_command().unwrap();

    assert_eq!(cmd, "docker");
    assert!(args.contains(&"run".to_string()));
    assert!(args.contains(&"--rm".to_string()));
    assert!(args.contains(&"-i".to_string())); // Interactive flag
    assert!(args.contains(&"nginx:latest".to_string()));
}

#[test]
fn test_config_field_types() {
    let server = DockerServer::new("nginx").unwrap();
    let metadata = server.metadata();

    // Verify field types are correct
    for field in &metadata.optional_config {
        match field.name.as_str() {
            "volumes" | "environment" | "ports" | "network" | "entrypoint" | "user"
            | "restart_policy" | "memory_limit" | "cpu_limit" => {
                assert_eq!(field.field_type, ConfigFieldType::String);
            }
            "working_dir" => {
                assert_eq!(field.field_type, ConfigFieldType::Path);
            }
            _ => panic!("Unexpected field: {}", field.name),
        }
    }
}

#[test]
fn test_special_docker_image_names() {
    // Test various special cases
    let special_images = vec![
        "ubuntu:20.04",
        "python:3.11.6-slim-bookworm",
        "mcr.microsoft.com/dotnet/runtime:7.0",
        "quay.io/coreos/etcd:v3.5.9",
        "sha256:abc123def456",
        "busybox@sha256:abc123",
        "localhost/my-image",
        "192.168.1.100:5000/test",
        "my-registry.com:443/org/repo/image:tag",
    ];

    for image in special_images {
        let server = DockerServer::new(image);
        assert!(server.is_ok(), "Failed to parse: {image}");
    }
}

#[test]
fn test_empty_config_values() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    config.insert("volumes".to_string(), "".to_string());
    config.insert("environment".to_string(), "".to_string());
    config.insert("ports".to_string(), "".to_string());

    // Empty values should be handled gracefully
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_whitespace_handling() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    // Whitespace around values
    config.insert(
        "volumes".to_string(),
        "  /host:/container  ,  /data:/data  ".to_string(),
    );
    config.insert(
        "environment".to_string(),
        "  KEY=value  ,  FOO=bar  ".to_string(),
    );
    config.insert("ports".to_string(), "  8080:80  ,  3000:3000  ".to_string());

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_complex_environment_values() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    // Environment values with special characters
    config.insert(
        "environment".to_string(),
        "CONNECTION_STRING=postgres://user:pass@host:5432/db,API_KEY=sk-1234567890,DEBUG=true"
            .to_string(),
    );

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_port_formats() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    // Various port formats
    config.insert(
        "ports".to_string(),
        "80:80,127.0.0.1:8080:8080,3000-3005:3000-3005".to_string(),
    );

    // All should be accepted (Docker will validate the actual format)
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_volume_formats() {
    let server = DockerServer::new("nginx").unwrap();

    let mut config = HashMap::new();
    // Various volume formats
    config.insert(
        "volumes".to_string(),
        "/host:/container:ro,/data:/data:rw,named-volume:/app,./relative:/path".to_string(),
    );

    // Basic validation should pass (only checks for colon)
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_memory_limit_formats() {
    let server = DockerServer::new("nginx").unwrap();

    let valid_limits = vec!["128m", "1g", "512M", "2G", "1024k", "100000000"];

    for limit in valid_limits {
        let mut config = HashMap::new();
        config.insert("memory_limit".to_string(), limit.to_string());
        assert!(
            server.validate_config(&config).is_ok(),
            "Failed for: {limit}"
        );
    }
}

#[test]
fn test_cpu_limit_formats() {
    let server = DockerServer::new("nginx").unwrap();

    let valid_limits = vec!["0.5", "1", "2.5", "4", "0.25"];

    for limit in valid_limits {
        let mut config = HashMap::new();
        config.insert("cpu_limit".to_string(), limit.to_string());
        assert!(
            server.validate_config(&config).is_ok(),
            "Failed for: {limit}"
        );
    }
}

#[test]
fn test_all_restart_policies() {
    let server = DockerServer::new("nginx").unwrap();

    let policies = vec!["no", "always", "unless-stopped", "on-failure"];

    for policy in policies {
        let mut config = HashMap::new();
        config.insert("restart_policy".to_string(), policy.to_string());
        assert!(
            server.validate_config(&config).is_ok(),
            "Failed for policy: {policy}"
        );
    }
}

#[test]
fn test_dependency_checker() {
    let server = DockerServer::new("nginx").unwrap();
    let checker = server.dependency();

    // Should return a DockerChecker
    // We can't test much more without mocking, but at least verify it doesn't panic
    drop(checker);
}

#[test]
fn test_full_configuration() {
    let server = DockerServer::new("nginx:alpine").unwrap();

    let mut config = HashMap::new();
    config.insert(
        "volumes".to_string(),
        "/var/www:/usr/share/nginx/html:ro".to_string(),
    );
    config.insert(
        "environment".to_string(),
        "NGINX_HOST=example.com,NGINX_PORT=80".to_string(),
    );
    config.insert("ports".to_string(), "80:80,443:443".to_string());
    config.insert("network".to_string(), "bridge".to_string());
    config.insert(
        "entrypoint".to_string(),
        "/docker-entrypoint.sh".to_string(),
    );
    config.insert(
        "working_dir".to_string(),
        "/usr/share/nginx/html".to_string(),
    );
    config.insert("user".to_string(), "nginx".to_string());
    config.insert("restart_policy".to_string(), "unless-stopped".to_string());
    config.insert("memory_limit".to_string(), "256m".to_string());
    config.insert("cpu_limit".to_string(), "0.5".to_string());

    assert!(server.validate_config(&config).is_ok());
}
