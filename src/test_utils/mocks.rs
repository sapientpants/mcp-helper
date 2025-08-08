//! Common mock implementations for testing
//!
//! This module provides reusable mock implementations of traits and structures
//! used throughout the test suite to reduce duplication and standardize testing approaches.

use crate::client::{McpClient, ServerConfig};
use crate::deps::{Dependency, DependencyCheck, DependencyChecker, DependencyStatus};
use crate::server::{McpServer, ServerMetadata, ServerType};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// Type alias for config validation function
type ConfigValidator = Box<dyn Fn(&HashMap<String, String>) -> Result<()> + Send + Sync>;

/// A builder for creating mock MCP servers with customizable behavior
pub struct MockServerBuilder {
    name: String,
    server_type: ServerType,
    description: Option<String>,
    dependency: Dependency,
    config_validator: ConfigValidator,
}

impl MockServerBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            server_type: ServerType::Npm {
                package: name_str,
                version: None,
            },
            description: None,
            dependency: Dependency::NodeJs { min_version: None },
            config_validator: Box::new(|_| Ok(())),
        }
    }

    pub fn with_type(mut self, server_type: ServerType) -> Self {
        self.server_type = server_type;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_dependency(mut self, dependency: Dependency) -> Self {
        self.dependency = dependency;
        self
    }

    pub fn with_config_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&HashMap<String, String>) -> Result<()> + Send + Sync + 'static,
    {
        self.config_validator = Box::new(validator);
        self
    }

    pub fn build(self) -> MockServer {
        MockServer {
            metadata: ServerMetadata {
                name: self.name,
                server_type: self.server_type,
                description: self.description,
                required_config: vec![],
                optional_config: vec![],
            },
            dependency: self.dependency,
            config_validator: self.config_validator,
        }
    }
}

/// A flexible mock implementation of the McpServer trait
pub struct MockServer {
    metadata: ServerMetadata,
    dependency: Dependency,
    config_validator: ConfigValidator,
}

impl McpServer for MockServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        Box::new(MockDependencyChecker::new(self.dependency.clone()))
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()> {
        (self.config_validator)(config)
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        match &self.metadata.server_type {
            ServerType::Npm { package, .. } => Ok(("npx".to_string(), vec![package.clone()])),
            _ => Ok(("mock".to_string(), vec!["command".to_string()])),
        }
    }
}

/// A builder for creating mock dependency checkers
pub struct MockDependencyCheckerBuilder {
    dependency: Dependency,
    status: DependencyStatus,
}

impl MockDependencyCheckerBuilder {
    pub fn new(dependency: Dependency) -> Self {
        Self {
            dependency,
            status: DependencyStatus::Installed {
                version: Some("1.0.0".to_string()),
            },
        }
    }

    pub fn with_status(mut self, status: DependencyStatus) -> Self {
        self.status = status;
        self
    }

    pub fn installed(mut self, version: impl Into<String>) -> Self {
        self.status = DependencyStatus::Installed {
            version: Some(version.into()),
        };
        self
    }

    pub fn missing(mut self) -> Self {
        self.status = DependencyStatus::Missing;
        self
    }

    pub fn version_mismatch(
        mut self,
        installed: impl Into<String>,
        required: impl Into<String>,
    ) -> Self {
        self.status = DependencyStatus::VersionMismatch {
            installed: installed.into(),
            required: required.into(),
        };
        self
    }

    pub fn build(self) -> MockDependencyChecker {
        MockDependencyChecker {
            dependency: self.dependency,
            status: self.status,
        }
    }
}

/// A mock implementation of the DependencyChecker trait
pub struct MockDependencyChecker {
    dependency: Dependency,
    status: DependencyStatus,
}

impl MockDependencyChecker {
    pub fn new(dependency: Dependency) -> Self {
        Self {
            dependency,
            status: DependencyStatus::Installed {
                version: Some("1.0.0".to_string()),
            },
        }
    }
}

impl DependencyChecker for MockDependencyChecker {
    fn check(&self) -> Result<DependencyCheck> {
        Ok(DependencyCheck {
            dependency: self.dependency.clone(),
            status: self.status.clone(),
            install_instructions: None,
        })
    }
}

/// A builder for creating mock MCP clients
pub struct MockClientBuilder {
    name: String,
    config_path: PathBuf,
    servers: HashMap<String, ServerConfig>,
    is_installed: bool,
}

impl MockClientBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            config_path: PathBuf::from("/tmp/mock-config.json"),
            servers: HashMap::new(),
            is_installed: true,
        }
    }

    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = path.into();
        self
    }

    pub fn with_server(mut self, name: impl Into<String>, config: ServerConfig) -> Self {
        self.servers.insert(name.into(), config);
        self
    }

    pub fn not_installed(mut self) -> Self {
        self.is_installed = false;
        self
    }

    pub fn build(self) -> MockClient {
        MockClient {
            name: self.name,
            config_path: self.config_path,
            servers: self.servers,
            is_installed: self.is_installed,
        }
    }
}

/// A mock implementation of the McpClient trait
pub struct MockClient {
    name: String,
    config_path: PathBuf,
    servers: HashMap<String, ServerConfig>,
    is_installed: bool,
}

impl McpClient for MockClient {
    fn name(&self) -> &str {
        &self.name
    }

    fn is_installed(&self) -> bool {
        self.is_installed
    }

    fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    fn add_server(&self, _name: &str, _config: ServerConfig) -> Result<()> {
        Ok(())
    }

    fn list_servers(&self) -> Result<HashMap<String, ServerConfig>> {
        Ok(self.servers.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_builder() {
        let server = MockServerBuilder::new("test-server")
            .with_type(ServerType::Npm {
                package: "test-server".to_string(),
                version: None,
            })
            .with_description("A test server")
            .with_dependency(Dependency::NodeJs {
                min_version: Some("18.0.0".to_string()),
            })
            .build();

        assert_eq!(server.metadata().name, "test-server");
        match &server.metadata().server_type {
            ServerType::Npm { package, version } => {
                assert_eq!(package, "test-server");
                assert_eq!(version, &None);
            }
            _ => panic!("Expected NPM server type"),
        }
        assert_eq!(
            server.metadata().description,
            Some("A test server".to_string())
        );
    }

    #[test]
    fn test_mock_dependency_checker_builder() {
        let checker = MockDependencyCheckerBuilder::new(Dependency::Docker {
            min_version: None,
            requires_compose: false,
        })
        .missing()
        .build();

        let result = checker.check().unwrap();
        assert!(matches!(result.status, DependencyStatus::Missing));
    }

    #[test]
    fn test_mock_client_builder() {
        let client = MockClientBuilder::new("MockClient")
            .with_config_path("/custom/path/config.json")
            .with_server(
                "test-server",
                ServerConfig {
                    command: "npx".to_string(),
                    args: vec!["test-server".to_string()],
                    env: HashMap::new(),
                },
            )
            .build();

        assert_eq!(client.name(), "MockClient");
        assert!(client.is_installed());
        assert_eq!(
            client.config_path(),
            PathBuf::from("/custom/path/config.json")
        );
    }
}
