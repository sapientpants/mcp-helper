use crate::server::{ConfigField, ServerType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Extended server metadata with registry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedServerMetadata {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub server_type: ServerType,
    pub required_config: Vec<ConfigField>,
    pub optional_config: Vec<ConfigField>,
    pub dependencies: Vec<String>,
    pub platform_support: PlatformSupport,
    pub examples: Vec<UsageExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformSupport {
    pub windows: bool,
    pub macos: bool,
    pub linux: bool,
    pub min_node_version: Option<String>,
    pub min_python_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    pub title: String,
    pub description: Option<String>,
    pub config: HashMap<String, String>,
}

/// Package.json structure for NPM servers
#[derive(Debug, Deserialize)]
struct PackageJson {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<serde_json::Value>, // Can be string or object
    pub homepage: Option<String>,
    pub repository: Option<serde_json::Value>, // Can be string or object
    pub license: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub engines: Option<PackageEngines>,
    pub mcp: Option<McpConfig>,
}

#[derive(Debug, Deserialize)]
struct PackageEngines {
    pub node: Option<String>,
    pub python: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McpConfig {
    pub required_config: Option<Vec<ConfigField>>,
    pub optional_config: Option<Vec<ConfigField>>,
    pub examples: Option<Vec<UsageExample>>,
}

/// Server registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    pub description: String,
    pub package_name: String,
    pub server_type: ServerType,
    pub category: String,
    pub tags: Vec<String>,
    pub popularity_score: f64,
    pub last_updated: String,
    pub verified: bool,
}

/// Parameters for creating a registry entry
struct RegistryEntryParams<'a> {
    package_name: &'a str,
    name: &'a str,
    description: &'a str,
    category: &'a str,
    tags: Vec<&'a str>,
    popularity_score: f64,
    last_updated: &'a str,
    verified: bool,
}

/// Server metadata loader and manager
pub struct MetadataLoader {
    cache: HashMap<String, ExtendedServerMetadata>,
    registry_cache: HashMap<String, RegistryEntry>,
}

impl MetadataLoader {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            registry_cache: HashMap::new(),
        }
    }

    /// Load metadata from package.json file (for NPM servers)
    pub fn load_from_package_json(
        &mut self,
        package_path: &Path,
    ) -> Result<ExtendedServerMetadata> {
        let package_json_path = package_path.join("package.json");

        if !package_json_path.exists() {
            anyhow::bail!("package.json not found at: {}", package_json_path.display());
        }

        let content = fs::read_to_string(&package_json_path).with_context(|| {
            format!(
                "Failed to read package.json: {}",
                package_json_path.display()
            )
        })?;

        let package: PackageJson = crate::utils::json_validator::deserialize_json_safe(&content)
            .context("Failed to parse package.json")?;

        let metadata = self.convert_package_json_to_metadata(package)?;

        // Cache the metadata
        self.cache.insert(metadata.name.clone(), metadata.clone());

        Ok(metadata)
    }

    /// Load metadata from awesome-mcp-servers registry (mock implementation)
    pub fn load_from_registry(&mut self, server_name: &str) -> Result<Option<RegistryEntry>> {
        // In a real implementation, this would fetch from an online registry
        // For now, we'll provide some mock data for common servers
        let mock_registry = self.get_mock_registry();

        if let Some(entry) = mock_registry.get(server_name) {
            self.registry_cache
                .insert(server_name.to_string(), entry.clone());
            Ok(Some(entry.clone()))
        } else {
            Ok(None)
        }
    }

    /// Search for servers in the registry
    pub fn search_registry(&self, query: &str) -> Vec<RegistryEntry> {
        let mock_registry = self.get_mock_registry();
        let query_lower = query.to_lowercase();

        mock_registry
            .values()
            .filter(|entry| {
                entry.name.to_lowercase().contains(&query_lower)
                    || entry.description.to_lowercase().contains(&query_lower)
                    || entry
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect()
    }

    /// Get cached metadata
    pub fn get_cached_metadata(&self, server_name: &str) -> Option<&ExtendedServerMetadata> {
        self.cache.get(server_name)
    }

    /// Get cached registry entry
    pub fn get_cached_registry_entry(&self, server_name: &str) -> Option<&RegistryEntry> {
        self.registry_cache.get(server_name)
    }

    fn convert_package_json_to_metadata(
        &self,
        package: PackageJson,
    ) -> Result<ExtendedServerMetadata> {
        let author = Self::extract_json_field_as_string(&package.author, "name");
        let repository = Self::extract_json_field_as_string(&package.repository, "url");
        let platform_support = Self::create_platform_support(&package.engines);
        let server_type = ServerType::Npm {
            package: package.name.clone(),
            version: Some(package.version.clone()),
        };

        let (required_config, optional_config, examples) = Self::extract_mcp_config(&package.mcp);

        Ok(ExtendedServerMetadata {
            name: package.name,
            description: package.description,
            version: Some(package.version),
            author,
            homepage: package.homepage,
            repository,
            license: package.license,
            keywords: package.keywords.unwrap_or_default(),
            server_type,
            required_config,
            optional_config,
            dependencies: vec![], // Would need to parse package.json dependencies
            platform_support,
            examples,
        })
    }

    fn extract_json_field_as_string(
        value: &Option<serde_json::Value>,
        field_name: &str,
    ) -> Option<String> {
        match value {
            Some(serde_json::Value::String(s)) => Some(s.clone()),
            Some(serde_json::Value::Object(obj)) => obj
                .get(field_name)
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            _ => None,
        }
    }

    fn create_platform_support(engines: &Option<PackageEngines>) -> PlatformSupport {
        PlatformSupport {
            windows: true, // Assume all platforms unless specified otherwise
            macos: true,
            linux: true,
            min_node_version: engines.as_ref().and_then(|e| e.node.clone()),
            min_python_version: engines.as_ref().and_then(|e| e.python.clone()),
        }
    }

    fn extract_mcp_config(
        mcp: &Option<McpConfig>,
    ) -> (Vec<ConfigField>, Vec<ConfigField>, Vec<UsageExample>) {
        let required_config = mcp
            .as_ref()
            .and_then(|mcp| mcp.required_config.as_ref())
            .cloned()
            .unwrap_or_default();

        let optional_config = mcp
            .as_ref()
            .and_then(|mcp| mcp.optional_config.as_ref())
            .cloned()
            .unwrap_or_default();

        let examples = mcp
            .as_ref()
            .and_then(|mcp| mcp.examples.as_ref())
            .cloned()
            .unwrap_or_default();

        (required_config, optional_config, examples)
    }

    fn get_mock_registry(&self) -> HashMap<String, RegistryEntry> {
        let mut registry = HashMap::new();

        let entries = [
            (
                "@modelcontextprotocol/server-filesystem",
                "Filesystem Server",
                "MCP server for filesystem operations",
                "File Management",
                vec!["filesystem", "files", "directory"],
                9.5,
                "2024-01-15",
                true,
            ),
            (
                "@anthropic/mcp-server-slack",
                "Slack Server",
                "MCP server for Slack integration",
                "Communication",
                vec!["slack", "messaging", "api"],
                8.7,
                "2024-01-10",
                true,
            ),
            (
                "mcp-server-git",
                "Git Server",
                "MCP server for Git operations",
                "Version Control",
                vec!["git", "version-control", "repository"],
                8.2,
                "2024-01-08",
                false,
            ),
        ];

        for (
            package_name,
            name,
            description,
            category,
            tags,
            popularity_score,
            last_updated,
            verified,
        ) in entries
        {
            let entry = self.create_registry_entry(&RegistryEntryParams {
                package_name,
                name,
                description,
                category,
                tags,
                popularity_score,
                last_updated,
                verified,
            });
            registry.insert(package_name.to_string(), entry);
        }

        registry
    }

    fn create_registry_entry(&self, params: &RegistryEntryParams) -> RegistryEntry {
        RegistryEntry {
            name: params.name.to_string(),
            description: params.description.to_string(),
            package_name: params.package_name.to_string(),
            server_type: ServerType::Npm {
                package: params.package_name.to_string(),
                version: None,
            },
            category: params.category.to_string(),
            tags: params.tags.iter().map(|s| s.to_string()).collect(),
            popularity_score: params.popularity_score,
            last_updated: params.last_updated.to_string(),
            verified: params.verified,
        }
    }
}

impl Default for MetadataLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_metadata_loader_creation() {
        let loader = MetadataLoader::new();
        assert!(loader.cache.is_empty());
        assert!(loader.registry_cache.is_empty());
    }

    #[test]
    fn test_mock_registry_data() {
        let loader = MetadataLoader::new();
        let registry = loader.get_mock_registry();

        assert!(!registry.is_empty());
        assert!(registry.contains_key("@modelcontextprotocol/server-filesystem"));
        assert!(registry.contains_key("@anthropic/mcp-server-slack"));

        let filesystem_entry = &registry["@modelcontextprotocol/server-filesystem"];
        assert_eq!(filesystem_entry.name, "Filesystem Server");
        assert!(filesystem_entry.verified);
        assert!(filesystem_entry.popularity_score > 8.0);
    }

    #[test]
    fn test_search_registry() {
        let loader = MetadataLoader::new();

        let results = loader.search_registry("filesystem");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Filesystem Server");

        let results = loader.search_registry("slack");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Slack Server");

        let results = loader.search_registry("nonexistent");
        assert!(results.is_empty());
    }

    #[test]
    fn test_load_from_registry() {
        let mut loader = MetadataLoader::new();

        let result = loader
            .load_from_registry("@modelcontextprotocol/server-filesystem")
            .unwrap();
        assert!(result.is_some());

        let entry = result.unwrap();
        assert_eq!(entry.name, "Filesystem Server");
        assert!(entry.verified);

        // Should be cached now
        assert!(loader
            .get_cached_registry_entry("@modelcontextprotocol/server-filesystem")
            .is_some());
    }

    #[test]
    fn test_load_from_package_json() {
        let temp_dir = TempDir::new().unwrap();
        let package_json = json!({
            "name": "test-mcp-server",
            "version": "1.0.0",
            "description": "A test MCP server",
            "author": "Test Author",
            "homepage": "https://example.com",
            "repository": "https://github.com/test/repo",
            "license": "MIT",
            "keywords": ["mcp", "test"],
            "engines": {
                "node": ">=16.0.0"
            },
            "mcp": {
                "required_config": [
                    {
                        "name": "api_key",
                        "field_type": "String",
                        "description": "API key for authentication"
                    }
                ],
                "optional_config": [
                    {
                        "name": "timeout",
                        "field_type": "Number",
                        "description": "Timeout in seconds",
                        "default": "30"
                    }
                ],
                "examples": [
                    {
                        "title": "Basic usage",
                        "description": "Simple configuration",
                        "config": {
                            "api_key": "your-api-key-here"
                        }
                    }
                ]
            }
        });

        fs::write(
            temp_dir.path().join("package.json"),
            serde_json::to_string_pretty(&package_json).unwrap(),
        )
        .unwrap();

        let mut loader = MetadataLoader::new();
        let metadata = loader.load_from_package_json(temp_dir.path()).unwrap();

        assert_eq!(metadata.name, "test-mcp-server");
        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.description, Some("A test MCP server".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.license, Some("MIT".to_string()));
        assert_eq!(metadata.keywords, vec!["mcp", "test"]);
        assert_eq!(
            metadata.platform_support.min_node_version,
            Some(">=16.0.0".to_string())
        );
        assert_eq!(metadata.required_config.len(), 1);
        assert_eq!(metadata.optional_config.len(), 1);
        assert_eq!(metadata.examples.len(), 1);

        // Should be cached
        assert!(loader.get_cached_metadata("test-mcp-server").is_some());
    }

    #[test]
    fn test_package_json_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let mut loader = MetadataLoader::new();

        let result = loader.load_from_package_json(temp_dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("package.json not found"));
    }

    #[test]
    fn test_platform_support_default() {
        let support = PlatformSupport::default();
        assert!(!support.windows);
        assert!(!support.macos);
        assert!(!support.linux);
        assert!(support.min_node_version.is_none());
        assert!(support.min_python_version.is_none());
    }

    #[test]
    fn test_usage_example_creation() {
        let example = UsageExample {
            title: "Test Example".to_string(),
            description: Some("A test example".to_string()),
            config: {
                let mut config = HashMap::new();
                config.insert("key".to_string(), "value".to_string());
                config
            },
        };

        assert_eq!(example.title, "Test Example");
        assert_eq!(example.description, Some("A test example".to_string()));
        assert_eq!(example.config.get("key"), Some(&"value".to_string()));
    }
}
