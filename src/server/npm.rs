use crate::deps::Dependency;
use crate::server::{
    detect_server_type, ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType,
};
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug)]
pub struct NpmServer {
    metadata: ServerMetadata,
    package: String,
    version: Option<String>,
}

impl NpmServer {
    pub fn new(package_spec: &str) -> Result<Self> {
        // Use existing detection logic
        match detect_server_type(package_spec) {
            ServerType::Npm { package, version } => {
                let metadata = ServerMetadata {
                    name: package.clone(),
                    description: Some(format!("NPM package: {package}")),
                    server_type: ServerType::Npm {
                        package: package.clone(),
                        version: version.clone(),
                    },
                    required_config: vec![],
                    optional_config: vec![],
                };

                Ok(Self {
                    metadata,
                    package,
                    version,
                })
            }
            _ => anyhow::bail!("Not a valid NPM package specification: {}", package_spec),
        }
    }

    pub fn from_package(package: String, version: Option<String>) -> Self {
        let metadata = ServerMetadata {
            name: package.clone(),
            description: Some(format!("NPM package: {package}")),
            server_type: ServerType::Npm {
                package: package.clone(),
                version: version.clone(),
            },
            required_config: vec![],
            optional_config: vec![],
        };

        Self {
            metadata,
            package,
            version,
        }
    }

    pub fn with_metadata(mut self, name: String, description: Option<String>) -> Self {
        self.metadata.name = name;
        self.metadata.description = description;
        self
    }

    pub fn with_config(mut self, required: Vec<ConfigField>, optional: Vec<ConfigField>) -> Self {
        self.metadata.required_config = required;
        self.metadata.optional_config = optional;
        self
    }

    fn get_npx_command(&self) -> String {
        #[cfg(target_os = "windows")]
        return "npx.cmd".to_string();

        #[cfg(not(target_os = "windows"))]
        return "npx".to_string();
    }

    fn build_package_arg(&self) -> String {
        match &self.version {
            Some(v) => format!("{}@{}", self.package, v),
            None => self.package.clone(),
        }
    }

    pub fn get_dependency(&self) -> Dependency {
        Dependency::NodeJs {
            min_version: Some("16.0.0".to_string()), // MCP servers typically require Node 16+
        }
    }
}

impl McpServer for NpmServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()> {
        // Check required fields
        for field in &self.metadata.required_config {
            if !config.contains_key(&field.name) {
                anyhow::bail!("Missing required configuration field: {}", field.name);
            }

            // Type validation
            if let Some(value) = config.get(&field.name) {
                match field.field_type {
                    ConfigFieldType::Number => {
                        value.parse::<f64>().map_err(|_| {
                            anyhow::anyhow!("Field '{}' must be a number", field.name)
                        })?;
                    }
                    ConfigFieldType::Boolean => {
                        value.parse::<bool>().map_err(|_| {
                            anyhow::anyhow!("Field '{}' must be true or false", field.name)
                        })?;
                    }
                    ConfigFieldType::Path => {
                        if value.is_empty() {
                            anyhow::bail!("Field '{}' (path) cannot be empty", field.name);
                        }
                    }
                    ConfigFieldType::Url => {
                        if !value.starts_with("http://") && !value.starts_with("https://") {
                            anyhow::bail!("Field '{}' must be a valid URL", field.name);
                        }
                    }
                    _ => {} // String type needs no special validation
                }
            }
        }

        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        let npx_cmd = self.get_npx_command();
        let package_arg = self.build_package_arg();

        // Basic npx arguments
        let mut args = vec![
            // Ensure package is installed/updated
            "--yes".to_string(),
            // The package to run
            package_arg,
        ];

        // Add stdio transport for MCP
        args.push("--stdio".to_string());

        Ok((npx_cmd, args))
    }

    fn dependency(&self) -> Box<dyn crate::deps::DependencyChecker> {
        use crate::deps::node::NodeChecker;
        Box::new(NodeChecker::new().with_min_version("16.0.0".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_server_new() {
        let server = NpmServer::new("@modelcontextprotocol/server-filesystem").unwrap();
        assert_eq!(server.package, "@modelcontextprotocol/server-filesystem");
        assert_eq!(server.version, None);
    }

    #[test]
    fn test_npm_server_with_version() {
        let server = NpmServer::new("express@4.18.0").unwrap();
        assert_eq!(server.package, "express");
        assert_eq!(server.version, Some("4.18.0".to_string()));
        match &server.metadata.server_type {
            ServerType::Npm { package, version } => {
                assert_eq!(package, "express");
                assert_eq!(version, &Some("4.18.0".to_string()));
            }
            _ => panic!("Expected NPM server type"),
        }
    }

    #[test]
    fn test_npm_server_invalid_package() {
        let result = NpmServer::new("https://github.com/test/repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_command() {
        let server = NpmServer::from_package("test-package".to_string(), None);
        let (cmd, args) = server.generate_command().unwrap();

        #[cfg(target_os = "windows")]
        assert_eq!(cmd, "npx.cmd");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(cmd, "npx");

        assert_eq!(args[0], "--yes");
        assert_eq!(args[1], "test-package");
        assert_eq!(args[2], "--stdio");
    }

    #[test]
    fn test_generate_command_with_version() {
        let server = NpmServer::from_package("test-package".to_string(), Some("1.0.0".to_string()));
        let (_, args) = server.generate_command().unwrap();
        assert_eq!(args[1], "test-package@1.0.0");
    }

    #[test]
    fn test_validate_config_required_field() {
        let server = NpmServer::from_package("test".to_string(), None).with_config(
            vec![ConfigField {
                name: "api_key".to_string(),
                field_type: ConfigFieldType::String,
                description: None,
                default: None,
            }],
            vec![],
        );

        let mut config = HashMap::new();
        assert!(server.validate_config(&config).is_err());

        config.insert("api_key".to_string(), "secret".to_string());
        assert!(server.validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_number_field() {
        let server = NpmServer::from_package("test".to_string(), None).with_config(
            vec![ConfigField {
                name: "port".to_string(),
                field_type: ConfigFieldType::Number,
                description: None,
                default: None,
            }],
            vec![],
        );

        let mut config = HashMap::new();
        config.insert("port".to_string(), "not-a-number".to_string());
        assert!(server.validate_config(&config).is_err());

        config.insert("port".to_string(), "8080".to_string());
        assert!(server.validate_config(&config).is_ok());
    }

    #[test]
    fn test_validate_config_url_field() {
        let server = NpmServer::from_package("test".to_string(), None).with_config(
            vec![ConfigField {
                name: "endpoint".to_string(),
                field_type: ConfigFieldType::Url,
                description: None,
                default: None,
            }],
            vec![],
        );

        let mut config = HashMap::new();
        config.insert("endpoint".to_string(), "not-a-url".to_string());
        assert!(server.validate_config(&config).is_err());

        config.insert(
            "endpoint".to_string(),
            "https://api.example.com".to_string(),
        );
        assert!(server.validate_config(&config).is_ok());
    }
}
