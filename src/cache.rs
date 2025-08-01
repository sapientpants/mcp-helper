//! Caching functionality for MCP Helper.
//!
//! This module provides caching for dependency checks, server metadata, and download artifacts
//! to improve performance and reduce redundant operations.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::deps::{Dependency, DependencyStatus};

/// Cache manager for MCP Helper operations.
#[derive(Debug)]
pub struct CacheManager {
    cache_dir: PathBuf,
    dependency_cache: DependencyCache,
    metadata_cache: MetadataCache,
}

impl CacheManager {
    /// Create a new cache manager with the default cache directory.
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir()?;
        fs::create_dir_all(&cache_dir)?;

        let dependency_cache = DependencyCache::load(&cache_dir)?;
        let metadata_cache = MetadataCache::load(&cache_dir)?;

        Ok(Self {
            cache_dir,
            dependency_cache,
            metadata_cache,
        })
    }

    /// Get the default cache directory for the current platform.
    fn default_cache_dir() -> Result<PathBuf> {
        let base = directories::ProjectDirs::from("com", "mcp-helper", "mcp-helper")
            .ok_or_else(|| anyhow::anyhow!("Failed to determine cache directory"))?;
        Ok(base.cache_dir().to_path_buf())
    }

    /// Get cached dependency status if available and not expired.
    pub fn get_dependency_status(&self, dependency: &Dependency) -> Option<&DependencyStatus> {
        self.dependency_cache.get(dependency)
    }

    /// Cache a dependency status result.
    pub fn cache_dependency_status(
        &mut self,
        dependency: Dependency,
        status: DependencyStatus,
    ) -> Result<()> {
        self.dependency_cache.insert(dependency, status);
        self.dependency_cache.save(&self.cache_dir)?;
        Ok(())
    }

    /// Get cached server metadata if available and not expired.
    pub fn get_server_metadata(&self, server_name: &str) -> Option<&CachedMetadata> {
        self.metadata_cache.get(server_name)
    }

    /// Cache server metadata.
    pub fn cache_server_metadata(
        &mut self,
        server_name: String,
        metadata: ServerMetadataInfo,
    ) -> Result<()> {
        self.metadata_cache.insert(server_name, metadata);
        self.metadata_cache.save(&self.cache_dir)?;
        Ok(())
    }

    /// Clear all caches.
    pub fn clear_all(&mut self) -> Result<()> {
        self.dependency_cache.clear();
        self.metadata_cache.clear();

        // Remove cache files
        let dep_cache_path = self.cache_dir.join("dependency_cache.json");
        let meta_cache_path = self.cache_dir.join("metadata_cache.json");

        if dep_cache_path.exists() {
            fs::remove_file(dep_cache_path)?;
        }
        if meta_cache_path.exists() {
            fs::remove_file(meta_cache_path)?;
        }

        Ok(())
    }

    /// Get the path to store downloaded artifacts.
    pub fn downloads_dir(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    /// Get cached download path if the file exists.
    pub fn get_cached_download(&self, url: &str) -> Option<PathBuf> {
        let filename = Self::url_to_filename(url);
        let path = self.downloads_dir().join(filename);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Convert a URL to a safe filename for caching.
    pub fn url_to_filename(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();

        // Extract filename from URL if possible
        let filename = url
            .split('/')
            .next_back()
            .and_then(|s| {
                let s = s.split('?').next().unwrap_or(s);
                if s.is_empty() || s == "download" {
                    None
                } else {
                    Some(s)
                }
            })
            .unwrap_or("download");

        format!("{hash:x}_{filename}")
    }
}

impl Default for CacheManager {
    fn default() -> Self {
        Self::new().expect("Failed to create cache manager")
    }
}

/// Cache for dependency check results.
#[derive(Debug, Serialize, Deserialize)]
struct DependencyCache {
    entries: HashMap<String, CachedDependency>,
    ttl: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedDependency {
    dependency: Dependency,
    status: DependencyStatus,
    cached_at: u64, // Unix timestamp
}

impl DependencyCache {
    const CACHE_FILE: &'static str = "dependency_cache.json";
    const DEFAULT_TTL: Duration = Duration::from_secs(3600); // 1 hour

    fn load(cache_dir: &Path) -> Result<Self> {
        let path = cache_dir.join(Self::CACHE_FILE);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(
                crate::utils::json_validator::deserialize_json_safe(&content)
                    .unwrap_or_else(|_| Self::new()),
            )
        } else {
            Ok(Self::new())
        }
    }

    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Self::DEFAULT_TTL,
        }
    }

    fn get(&self, dependency: &Dependency) -> Option<&DependencyStatus> {
        let key = self.dependency_key(dependency);
        self.entries.get(&key).and_then(|entry| {
            if self.is_expired(entry.cached_at) {
                None
            } else {
                Some(&entry.status)
            }
        })
    }

    fn insert(&mut self, dependency: Dependency, status: DependencyStatus) {
        let key = self.dependency_key(&dependency);
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(
            key,
            CachedDependency {
                dependency,
                status,
                cached_at,
            },
        );
    }

    fn save(&self, cache_dir: &Path) -> Result<()> {
        let path = cache_dir.join(Self::CACHE_FILE);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn dependency_key(&self, dependency: &Dependency) -> String {
        match dependency {
            Dependency::NodeJs { min_version } => {
                format!("nodejs:{}", min_version.as_deref().unwrap_or("any"))
            }
            Dependency::Docker {
                min_version,
                requires_compose,
            } => {
                format!(
                    "docker:{}:{}",
                    min_version.as_deref().unwrap_or("any"),
                    requires_compose
                )
            }
            Dependency::Python { min_version } => {
                format!("python:{}", min_version.as_deref().unwrap_or("any"))
            }
            Dependency::Git => "git:any".to_string(),
        }
    }

    fn is_expired(&self, cached_at: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - cached_at > self.ttl.as_secs()
    }
}

/// Cache for server metadata.
#[derive(Debug, Serialize, Deserialize)]
struct MetadataCache {
    entries: HashMap<String, CachedMetadata>,
    ttl: Duration,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedMetadata {
    pub metadata: ServerMetadataInfo,
    cached_at: u64, // Unix timestamp
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerMetadataInfo {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
    pub dependencies: Vec<String>,
    pub config_schema: Option<serde_json::Value>,
}

impl MetadataCache {
    const CACHE_FILE: &'static str = "metadata_cache.json";
    const DEFAULT_TTL: Duration = Duration::from_secs(86400); // 24 hours

    fn load(cache_dir: &Path) -> Result<Self> {
        let path = cache_dir.join(Self::CACHE_FILE);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(
                crate::utils::json_validator::deserialize_json_safe(&content)
                    .unwrap_or_else(|_| Self::new()),
            )
        } else {
            Ok(Self::new())
        }
    }

    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Self::DEFAULT_TTL,
        }
    }

    fn get(&self, server_name: &str) -> Option<&CachedMetadata> {
        self.entries.get(server_name).and_then(|entry| {
            if self.is_expired(entry.cached_at) {
                None
            } else {
                Some(entry)
            }
        })
    }

    fn insert(&mut self, server_name: String, metadata: ServerMetadataInfo) {
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(
            server_name,
            CachedMetadata {
                metadata,
                cached_at,
            },
        );
    }

    fn save(&self, cache_dir: &Path) -> Result<()> {
        let path = cache_dir.join(Self::CACHE_FILE);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.entries.clear();
    }

    fn is_expired(&self, cached_at: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - cached_at > self.ttl.as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let cache_manager = CacheManager::new();
        assert!(cache_manager.is_ok());
    }

    #[test]
    fn test_dependency_caching() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut cache_manager = CacheManager::new().unwrap();

        let dependency = Dependency::NodeJs {
            min_version: Some("18.0.0".to_string()),
        };

        let status = DependencyStatus::Installed {
            version: Some("18.17.0".to_string()),
        };

        // Cache the dependency status
        cache_manager
            .cache_dependency_status(dependency.clone(), status.clone())
            .unwrap();

        // Retrieve from cache
        let cached = cache_manager.get_dependency_status(&dependency);
        assert!(cached.is_some());

        match cached.unwrap() {
            DependencyStatus::Installed { version } => {
                assert_eq!(version.as_deref(), Some("18.17.0"));
            }
            _ => panic!("Expected Installed status"),
        }
    }

    #[test]
    fn test_metadata_caching() {
        let temp_dir = TempDir::new().unwrap();
        // Set the appropriate cache directory environment variable
        std::env::set_var("XDG_CACHE_HOME", temp_dir.path());

        let mut cache_manager = CacheManager::new().unwrap();

        let metadata = ServerMetadataInfo {
            name: "test-server".to_string(),
            description: Some("Test server description".to_string()),
            version: Some("1.0.0".to_string()),
            dependencies: vec!["nodejs".to_string()],
            config_schema: None,
        };

        // Cache the metadata
        cache_manager
            .cache_server_metadata("test-server".to_string(), metadata.clone())
            .unwrap();

        // Retrieve from cache
        let cached = cache_manager.get_server_metadata("test-server");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().metadata.name, "test-server");

        // Clean up
        std::env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_url_to_filename() {
        let url1 = "https://github.com/user/repo/releases/download/v1.0/binary.exe";
        let filename1 = CacheManager::url_to_filename(url1);
        assert!(filename1.ends_with("_binary.exe"));

        let url2 = "https://example.com/download?file=test";
        let filename2 = CacheManager::url_to_filename(url2);
        assert!(filename2.ends_with("_download"));
    }

    #[test]
    fn test_cache_clear() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut cache_manager = CacheManager::new().unwrap();

        // Add some data
        let dependency = Dependency::NodeJs {
            min_version: Some("18.0.0".to_string()),
        };
        let status = DependencyStatus::Installed {
            version: Some("18.17.0".to_string()),
        };
        cache_manager
            .cache_dependency_status(dependency.clone(), status)
            .unwrap();

        // Clear cache
        cache_manager.clear_all().unwrap();

        // Verify it's gone
        let cached = cache_manager.get_dependency_status(&dependency);
        assert!(cached.is_none());
    }
}
