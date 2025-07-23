pub mod npm;

use anyhow::Result;
use std::collections::HashMap;

pub use npm::NpmServer;

#[derive(Debug, Clone, PartialEq)]
pub enum ServerType {
    Npm {
        package: String,
        version: Option<String>,
    },
    Binary {
        url: String,
        checksum: Option<String>,
    },
    Python {
        package: String,
        version: Option<String>,
    },
    Docker {
        image: String,
        tag: String,
    },
}

#[derive(Debug, Clone)]
pub struct ServerMetadata {
    pub name: String,
    pub description: Option<String>,
    pub server_type: ServerType,
    pub required_config: Vec<ConfigField>,
    pub optional_config: Vec<ConfigField>,
}

#[derive(Debug, Clone)]
pub struct ConfigField {
    pub name: String,
    pub field_type: ConfigFieldType,
    pub description: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFieldType {
    String,
    Number,
    Boolean,
    Path,
    Url,
}

pub trait McpServer: Send + Sync {
    fn metadata(&self) -> &ServerMetadata;

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()>;

    fn generate_command(&self) -> Result<(String, Vec<String>)>;
}

pub fn detect_server_type(package: &str) -> ServerType {
    if let Some(stripped) = package.strip_prefix("docker:") {
        let parts: Vec<&str> = stripped.splitn(2, ':').collect();
        ServerType::Docker {
            image: parts[0].to_string(),
            tag: parts.get(1).unwrap_or(&"latest").to_string(),
        }
    } else if package.starts_with("https://") || package.starts_with("http://") {
        ServerType::Binary {
            url: package.to_string(),
            checksum: None,
        }
    } else if package.ends_with(".py") {
        ServerType::Python {
            package: package.to_string(),
            version: None,
        }
    } else if package.starts_with('@') || package.contains('/') {
        let (pkg, version) = parse_npm_package(package);
        ServerType::Npm {
            package: pkg,
            version,
        }
    } else {
        // Default to NPM for simple names
        let (pkg, version) = parse_npm_package(package);
        ServerType::Npm {
            package: pkg,
            version,
        }
    }
}

pub fn parse_npm_package(package: &str) -> (String, Option<String>) {
    if let Some(stripped) = package.strip_prefix('@') {
        // This is a scoped package
        if let Some(version_at) = stripped.rfind('@') {
            // Found a version specifier after the scope
            let at_pos = version_at + 1;
            let pkg = package[..at_pos].to_string();
            let version = Some(package[at_pos + 1..].to_string());
            (pkg, version)
        } else {
            // No version specified
            (package.to_string(), None)
        }
    } else if let Some(at_pos) = package.rfind('@') {
        // Non-scoped package with version
        let pkg = package[..at_pos].to_string();
        let version = Some(package[at_pos + 1..].to_string());
        (pkg, version)
    } else {
        // No version specified
        (package.to_string(), None)
    }
}
