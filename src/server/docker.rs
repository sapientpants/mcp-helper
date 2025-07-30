use crate::deps::{DependencyChecker, DockerChecker};
use crate::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use anyhow::{Context, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct DockerServer {
    metadata: ServerMetadata,
    image: String,
    tag: Option<String>,
    entrypoint: Option<String>,
    working_dir: Option<String>,
}

impl DockerServer {
    pub fn new(docker_spec: &str) -> Result<Self> {
        let (image, tag) = Self::parse_docker_spec(docker_spec);

        let metadata = ServerMetadata {
            name: image.clone(),
            description: Some(format!("Docker MCP server: {image}")),
            server_type: ServerType::Docker {
                image: image.clone(),
                tag: tag.clone(),
            },
            required_config: vec![],
            optional_config: vec![
                ConfigField {
                    name: "volumes".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some(
                        "Volume mounts (host:container format, comma-separated)".to_string(),
                    ),
                    default: None,
                },
                ConfigField {
                    name: "environment".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some(
                        "Environment variables (KEY=value format, comma-separated)".to_string(),
                    ),
                    default: None,
                },
                ConfigField {
                    name: "ports".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some(
                        "Port mappings (host:container format, comma-separated)".to_string(),
                    ),
                    default: None,
                },
                ConfigField {
                    name: "network".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some("Docker network to use".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "entrypoint".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some("Custom entrypoint command".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "working_dir".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Working directory inside container".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "user".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some("User to run as (uid:gid or username)".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "restart_policy".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some(
                        "Container restart policy (no, always, unless-stopped, on-failure)"
                            .to_string(),
                    ),
                    default: Some("unless-stopped".to_string()),
                },
                ConfigField {
                    name: "memory_limit".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some("Memory limit (e.g., 512m, 1g)".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "cpu_limit".to_string(),
                    field_type: ConfigFieldType::String,
                    description: Some("CPU limit (e.g., 0.5, 2)".to_string()),
                    default: None,
                },
            ],
        };

        Ok(Self {
            metadata,
            image,
            tag,
            entrypoint: None,
            working_dir: None,
        })
    }

    fn parse_docker_spec(docker_spec: &str) -> (String, Option<String>) {
        if let Some(colon_pos) = docker_spec.rfind(':') {
            // Check if this is a version tag (not a port in hostname)
            let after_colon = &docker_spec[colon_pos + 1..];
            let before_colon = &docker_spec[..colon_pos];

            // If there's a slash after the last colon, it's likely part of the image name (registry:port/image)
            // If there's no slash after the colon and it looks like a tag, treat it as a tag
            if !after_colon.contains('/')
                && (after_colon
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
                    || after_colon.starts_with('v')
                        && after_colon[1..]
                            .chars()
                            .all(|c| c.is_ascii_digit() || c == '.'))
            {
                // This looks like a version tag
                let image = before_colon.to_string();
                let tag = Some(after_colon.to_string());
                (image, tag)
            } else {
                // No tag specified, the colon is part of the image name (e.g., registry:port/image)
                (docker_spec.to_string(), None)
            }
        } else {
            // No colon, so no tag specified
            (docker_spec.to_string(), None)
        }
    }

    pub fn with_entrypoint(mut self, entrypoint: impl Into<String>) -> Self {
        self.entrypoint = Some(entrypoint.into());
        self
    }

    pub fn with_working_dir(mut self, working_dir: impl Into<String>) -> Self {
        self.working_dir = Some(working_dir.into());
        self
    }

    fn parse_volumes(&self, volumes_str: &str) -> Vec<String> {
        let mut result = Vec::new();
        for volume in volumes_str.split(',') {
            let volume = volume.trim();
            if !volume.is_empty() {
                result.push("-v".to_string());
                result.push(volume.to_string());
            }
        }
        result
    }

    fn parse_environment(&self, env_str: &str) -> Vec<String> {
        let mut result = Vec::new();
        for env_var in env_str.split(',') {
            let env_var = env_var.trim();
            if !env_var.is_empty() && env_var.contains('=') {
                result.push("-e".to_string());
                result.push(env_var.to_string());
            }
        }
        result
    }

    fn parse_ports(&self, ports_str: &str) -> Vec<String> {
        let mut result = Vec::new();
        for port in ports_str.split(',') {
            let port = port.trim();
            if !port.is_empty() {
                result.push("-p".to_string());
                result.push(port.to_string());
            }
        }
        result
    }

    fn generate_container_name(&self) -> String {
        let base_name = self.image.replace(['/', ':'], "-");
        format!("mcp-{base_name}")
    }

    pub fn pull_image(&self) -> Result<()> {
        let full_image = if let Some(ref tag) = self.tag {
            format!("{}:{}", self.image, tag)
        } else {
            self.image.clone()
        };

        println!("🐳 Pulling Docker image: {full_image}");

        let output = std::process::Command::new("docker")
            .args(["pull", &full_image])
            .output()
            .context("Failed to execute docker pull command")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to pull Docker image {}: {}", full_image, error_msg);
        }

        println!("✅ Successfully pulled {full_image}");
        Ok(())
    }

    pub fn image_exists(&self) -> Result<bool> {
        let full_image = if let Some(ref tag) = self.tag {
            format!("{}:{}", self.image, tag)
        } else {
            self.image.clone()
        };

        let output = std::process::Command::new("docker")
            .args(["image", "inspect", &full_image])
            .output()
            .context("Failed to execute docker image inspect command")?;

        Ok(output.status.success())
    }
}

impl McpServer for DockerServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()> {
        // Validate volumes format
        if let Some(volumes) = config.get("volumes") {
            for volume in volumes.split(',') {
                let volume = volume.trim();
                if !volume.is_empty() && !volume.contains(':') {
                    anyhow::bail!(
                        "Invalid volume format '{}'. Expected 'host:container' format",
                        volume
                    );
                }
            }
        }

        // Validate environment variables format
        if let Some(env_vars) = config.get("environment") {
            for env_var in env_vars.split(',') {
                let env_var = env_var.trim();
                if !env_var.is_empty() && !env_var.contains('=') {
                    anyhow::bail!(
                        "Invalid environment variable format '{}'. Expected 'KEY=value' format",
                        env_var
                    );
                }
            }
        }

        // Validate ports format
        if let Some(ports) = config.get("ports") {
            for port in ports.split(',') {
                let port = port.trim();
                if !port.is_empty() && !port.contains(':') {
                    anyhow::bail!(
                        "Invalid port format '{}'. Expected 'host:container' format",
                        port
                    );
                }
            }
        }

        // Validate restart policy
        if let Some(restart_policy) = config.get("restart_policy") {
            let valid_policies = ["no", "always", "unless-stopped", "on-failure"];
            if !valid_policies.contains(&restart_policy.as_str()) {
                anyhow::bail!(
                    "Invalid restart policy '{}'. Valid options: {}",
                    restart_policy,
                    valid_policies.join(", ")
                );
            }
        }

        // Validate numeric values
        if let Some(cpu_limit) = config.get("cpu_limit") {
            cpu_limit
                .parse::<f64>()
                .with_context(|| format!("Invalid CPU limit '{cpu_limit}'. Expected a number"))?;
        }

        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        let config = HashMap::new(); // Use default config for command generation
        self.generate_command_with_config(&config)
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        Box::new(DockerChecker::new())
    }
}

impl DockerServer {
    pub fn generate_command_with_config(
        &self,
        config: &HashMap<String, String>,
    ) -> Result<(String, Vec<String>)> {
        let mut args = vec!["run".to_string()];

        // Add common options
        args.push("--rm".to_string()); // Remove container when it exits
        args.push("-i".to_string()); // Interactive
        args.push("--init".to_string()); // Use proper init process

        // Add container name
        args.push("--name".to_string());
        args.push(self.generate_container_name());

        // Add volumes
        if let Some(volumes) = config.get("volumes") {
            args.extend(self.parse_volumes(volumes));
        }

        // Add environment variables
        if let Some(env_vars) = config.get("environment") {
            args.extend(self.parse_environment(env_vars));
        }

        // Add ports
        if let Some(ports) = config.get("ports") {
            args.extend(self.parse_ports(ports));
        }

        // Add network
        if let Some(network) = config.get("network") {
            args.push("--network".to_string());
            args.push(network.clone());
        }

        // Add user
        if let Some(user) = config.get("user") {
            args.push("--user".to_string());
            args.push(user.clone());
        }

        // Add restart policy
        if let Some(restart_policy) = config.get("restart_policy") {
            args.push("--restart".to_string());
            args.push(restart_policy.clone());
        }

        // Add memory limit
        if let Some(memory_limit) = config.get("memory_limit") {
            args.push(format!("--memory={memory_limit}"));
        }

        // Add CPU limit
        if let Some(cpu_limit) = config.get("cpu_limit") {
            args.push(format!("--cpus={cpu_limit}"));
        }

        // Add working directory
        let working_dir = config.get("working_dir").or(self.working_dir.as_ref());
        if let Some(wd) = working_dir {
            args.push("--workdir".to_string());
            args.push(wd.clone());
        }

        // Add entrypoint if specified
        let entrypoint = config.get("entrypoint").or(self.entrypoint.as_ref());
        if let Some(ep) = entrypoint {
            args.push("--entrypoint".to_string());
            args.push(ep.clone());
        }

        // Add the Docker image
        let full_image = if let Some(ref tag) = self.tag {
            format!("{}:{}", self.image, tag)
        } else {
            self.image.clone()
        };
        args.push(full_image);

        Ok(("docker".to_string(), args))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_spec() {
        // Test with tag
        let (image, tag) = DockerServer::parse_docker_spec("nginx:1.21");
        assert_eq!(image, "nginx");
        assert_eq!(tag, Some("1.21".to_string()));

        // Test with complex tag
        let (image, tag) = DockerServer::parse_docker_spec("ubuntu:20.04");
        assert_eq!(image, "ubuntu");
        assert_eq!(tag, Some("20.04".to_string()));

        // Test without tag
        let (image, tag) = DockerServer::parse_docker_spec("nginx");
        assert_eq!(image, "nginx");
        assert_eq!(tag, None);

        // Test with registry and namespace
        let (image, tag) = DockerServer::parse_docker_spec("registry.io/user/app:v1.0");
        assert_eq!(image, "registry.io/user/app");
        assert_eq!(tag, Some("v1.0".to_string()));
    }

    #[test]
    fn test_docker_server_creation() {
        let server = DockerServer::new("nginx:1.21").unwrap();
        assert_eq!(server.metadata.name, "nginx");
        assert_eq!(server.image, "nginx");
        assert_eq!(server.tag, Some("1.21".to_string()));
    }

    #[test]
    fn test_docker_server_without_tag() {
        let server = DockerServer::new("ubuntu").unwrap();
        assert_eq!(server.metadata.name, "ubuntu");
        assert_eq!(server.image, "ubuntu");
        assert_eq!(server.tag, None);
    }

    #[test]
    fn test_with_entrypoint() {
        let server = DockerServer::new("nginx")
            .unwrap()
            .with_entrypoint("/bin/bash");
        assert_eq!(server.entrypoint, Some("/bin/bash".to_string()));
    }

    #[test]
    fn test_with_working_dir() {
        let server = DockerServer::new("nginx").unwrap().with_working_dir("/app");
        assert_eq!(server.working_dir, Some("/app".to_string()));
    }

    #[test]
    fn test_parse_volumes() {
        let server = DockerServer::new("nginx").unwrap();
        let volumes = server.parse_volumes("/host/path:/container/path,/another:/path");
        assert_eq!(
            volumes,
            vec!["-v", "/host/path:/container/path", "-v", "/another:/path"]
        );
    }

    #[test]
    fn test_parse_environment() {
        let server = DockerServer::new("nginx").unwrap();
        let env_vars = server.parse_environment("KEY1=value1,KEY2=value2");
        assert_eq!(env_vars, vec!["-e", "KEY1=value1", "-e", "KEY2=value2"]);
    }

    #[test]
    fn test_parse_ports() {
        let server = DockerServer::new("nginx").unwrap();
        let ports = server.parse_ports("8080:80,8443:443");
        assert_eq!(ports, vec!["-p", "8080:80", "-p", "8443:443"]);
    }

    #[test]
    fn test_generate_container_name() {
        let server = DockerServer::new("registry.io/user/app:v1.0").unwrap();
        let name = server.generate_container_name();
        assert_eq!(name, "mcp-registry.io-user-app");
    }

    #[test]
    fn test_validate_config_volumes() {
        let server = DockerServer::new("nginx").unwrap();

        let mut config = HashMap::new();
        config.insert("volumes".to_string(), "/host:/container".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("volumes".to_string(), "invalid_volume".to_string());
        assert!(server.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_environment() {
        let server = DockerServer::new("nginx").unwrap();

        let mut config = HashMap::new();
        config.insert("environment".to_string(), "KEY=value".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("environment".to_string(), "INVALID_ENV".to_string());
        assert!(server.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_restart_policy() {
        let server = DockerServer::new("nginx").unwrap();

        let mut config = HashMap::new();
        config.insert("restart_policy".to_string(), "always".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("restart_policy".to_string(), "invalid".to_string());
        assert!(server.validate_config(&config).is_err());
    }

    #[test]
    fn test_validate_config_cpu_limit() {
        let server = DockerServer::new("nginx").unwrap();

        let mut config = HashMap::new();
        config.insert("cpu_limit".to_string(), "1.5".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("cpu_limit".to_string(), "invalid".to_string());
        assert!(server.validate_config(&config).is_err());
    }

    #[test]
    fn test_generate_command_basic() {
        let server = DockerServer::new("nginx:1.21").unwrap();
        let (cmd, args) = server.generate_command().unwrap();

        assert_eq!(cmd, "docker");
        assert!(args.contains(&"run".to_string()));
        assert!(args.contains(&"nginx:1.21".to_string()));
        assert!(args.contains(&"--rm".to_string()));
        assert!(args.contains(&"-i".to_string()));
    }

    #[test]
    fn test_generate_command_with_config() {
        let server = DockerServer::new("nginx").unwrap();
        let mut config = HashMap::new();
        config.insert("volumes".to_string(), "/host:/container".to_string());
        config.insert("environment".to_string(), "KEY=value".to_string());
        config.insert("ports".to_string(), "8080:80".to_string());

        let (cmd, args) = server.generate_command_with_config(&config).unwrap();

        assert_eq!(cmd, "docker");

        // Check for volume arguments (should appear as separate -v flag and value)
        let volume_index = args.iter().position(|arg| arg == "-v");
        assert!(volume_index.is_some());
        let volume_index = volume_index.unwrap();
        assert_eq!(args[volume_index + 1], "/host:/container");

        // Check for environment arguments (should appear as separate -e flag and value)
        let env_index = args.iter().position(|arg| arg == "-e");
        assert!(env_index.is_some());
        let env_index = env_index.unwrap();
        assert_eq!(args[env_index + 1], "KEY=value");

        // Check for port arguments (should appear as separate -p flag and value)
        let port_index = args.iter().position(|arg| arg == "-p");
        assert!(port_index.is_some());
        let port_index = port_index.unwrap();
        assert_eq!(args[port_index + 1], "8080:80");
    }
}
