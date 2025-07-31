//! MCP server implementations and types.
//!
//! This module provides support for different types of MCP servers including NPM packages,
//! Docker images, GitHub repositories (binaries), and Python packages. Each server type
//! has specific installation and configuration requirements.
//!
//! # Examples
//!
//! ## Detecting Server Types
//!
//! ```rust,no_run
//! use mcp_helper::server::{detect_server_type, ServerType};
//!
//! // NPM package
//! let npm_type = detect_server_type("@modelcontextprotocol/server-filesystem");
//! match npm_type {
//!     ServerType::Npm { package, version } => {
//!         println!("NPM package: {}, version: {:?}", package, version);
//!     }
//!     _ => {}
//! }
//!
//! // Docker image
//! let docker_type = detect_server_type("docker:nginx:alpine");
//! match docker_type {
//!     ServerType::Docker { image, tag } => {
//!         println!("Docker image: {}, tag: {:?}", image, tag);
//!     }
//!     _ => {}
//! }
//! ```
//!
//! ## Working with Server Instances
//!
//! ```rust,no_run
//! use mcp_helper::server::{NpmServer, McpServer};
//!
//! // Create an NPM server instance
//! let server = NpmServer::new("@modelcontextprotocol/server-filesystem".to_string(), None);
//!
//! // Get server metadata
//! let metadata = server.metadata();
//! println!("Server: {}", metadata.name);
//! println!("Description: {:?}", metadata.description);
//!
//! // Check dependencies
//! let deps = server.dependencies();
//! for dep in deps {
//!     println!("Dependency: {:?}", dep);
//! }
//! ```

pub mod binary;
pub mod docker;
pub mod metadata;
pub mod npm;
pub mod python;
pub mod suggestions;

use anyhow::Result;
use std::collections::HashMap;

use crate::deps::DependencyChecker;

pub use binary::BinaryServer;
pub use docker::DockerServer;
pub use metadata::{
    ExtendedServerMetadata, MetadataLoader, PlatformSupport, RegistryEntry, UsageExample,
};
pub use npm::NpmServer;
pub use python::PythonServer;
pub use suggestions::{ServerSuggestions, Suggestion, SuggestionFeasibility, SuggestionReason};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
        tag: Option<String>,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigField {
    pub name: String,
    pub field_type: ConfigFieldType,
    pub description: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

    fn dependency(&self) -> Box<dyn DependencyChecker>;
}

pub fn detect_server_type(package: &str) -> ServerType {
    if let Some(stripped) = package.strip_prefix("docker:") {
        let parts: Vec<&str> = stripped.splitn(2, ':').collect();
        ServerType::Docker {
            image: parts[0].to_string(),
            tag: parts
                .get(1)
                .map(|s| s.to_string())
                .or(Some("latest".to_string())),
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
