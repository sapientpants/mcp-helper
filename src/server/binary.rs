use crate::deps::{Dependency, DependencyChecker, DependencyStatus};
use crate::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    #[allow(dead_code)]
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug)]
pub struct BinaryServer {
    metadata: ServerMetadata,
    url: String,
    checksum: Option<String>,
    binary_path: Option<PathBuf>,
}

impl BinaryServer {
    pub fn new(url: &str, checksum: Option<String>) -> Self {
        let name = Self::extract_name_from_url(url);
        let metadata = ServerMetadata {
            name: name.clone(),
            description: Some(format!("Binary server from: {url}")),
            server_type: ServerType::Binary {
                url: url.to_string(),
                checksum: checksum.clone(),
            },
            required_config: vec![],
            optional_config: vec![
                ConfigField {
                    name: "working_directory".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Working directory for the server".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "timeout".to_string(),
                    field_type: ConfigFieldType::Number,
                    description: Some("Server timeout in seconds".to_string()),
                    default: Some("30".to_string()),
                },
            ],
        };

        Self {
            metadata,
            url: url.to_string(),
            checksum,
            binary_path: None,
        }
    }

    pub fn from_github_repo(repo: &str, version: Option<&str>) -> Result<Self> {
        let client = Client::new();
        let api_url = if let Some(v) = version {
            format!("https://api.github.com/repos/{repo}/releases/tags/{v}")
        } else {
            format!("https://api.github.com/repos/{repo}/releases/latest")
        };

        let response = client
            .get(&api_url)
            .header("User-Agent", "mcp-helper")
            .send()
            .context("Failed to fetch GitHub release")?;

        if !response.status().is_success() {
            anyhow::bail!("GitHub API request failed: {}", response.status());
        }

        let release: GitHubRelease = response
            .json()
            .context("Failed to parse GitHub release response")?;

        let platform_asset = Self::select_platform_asset(&release.assets)?;

        Ok(Self::new(&platform_asset.browser_download_url, None))
    }

    fn extract_name_from_url(url: &str) -> String {
        if let Some(repo) = Self::extract_github_repo(url) {
            repo.split('/')
                .next_back()
                .unwrap_or("binary-server")
                .to_string()
        } else {
            url.split('/')
                .next_back()
                .and_then(|name| name.split('?').next())
                .unwrap_or("binary-server")
                .to_string()
        }
    }

    fn extract_github_repo(url: &str) -> Option<String> {
        if url.contains("github.com") {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 5 && parts[2] == "github.com" {
                return Some(format!("{}/{}", parts[3], parts[4]));
            }
        }
        None
    }

    fn select_platform_asset(assets: &[GitHubAsset]) -> Result<&GitHubAsset> {
        let platform = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Platform-specific patterns
        let patterns = match platform {
            "windows" => vec!["windows", "win", "pc"],
            "macos" => vec!["darwin", "macos", "osx", "apple"],
            "linux" => vec!["linux", "gnu"],
            _ => vec![platform],
        };

        let arch_patterns = match arch {
            "x86_64" => vec!["x86_64", "x64", "amd64"],
            "aarch64" => vec!["aarch64", "arm64"],
            _ => vec![arch],
        };

        // Find best matching asset
        for asset in assets {
            let name_lower = asset.name.to_lowercase();

            let platform_match = patterns.iter().any(|p| name_lower.contains(p));
            let arch_match = arch_patterns.iter().any(|a| name_lower.contains(a));

            if platform_match && arch_match {
                return Ok(asset);
            }
        }

        // Fallback: try platform match only
        for asset in assets {
            let name_lower = asset.name.to_lowercase();
            if patterns.iter().any(|p| name_lower.contains(p)) {
                return Ok(asset);
            }
        }

        anyhow::bail!(
            "No suitable binary found for platform: {} {}. Available assets: {}",
            platform,
            arch,
            assets
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    pub fn download_and_install(&mut self) -> Result<PathBuf> {
        let bin_dir = self.get_bin_directory()?;
        fs::create_dir_all(&bin_dir)?;

        let filename = self
            .url
            .split('/')
            .next_back()
            .and_then(|name| name.split('?').next())
            .context("Could not determine filename from URL")?;

        let binary_path = bin_dir.join(filename);

        // Download the binary
        self.download_binary(&binary_path)?;

        // Verify checksum if provided
        if let Some(expected_checksum) = &self.checksum {
            self.verify_checksum(&binary_path, expected_checksum)?;
        }

        // Make executable on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        self.binary_path = Some(binary_path.clone());
        println!("✅ Binary installed to: {}", binary_path.display());

        Ok(binary_path)
    }

    fn get_bin_directory(&self) -> Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .context("Could not determine home directory")?
            .home_dir()
            .to_path_buf();

        Ok(home.join(".mcp").join("bin"))
    }

    fn download_binary(&self, output_path: &Path) -> Result<()> {
        let client = Client::new();
        let response = client
            .get(&self.url)
            .send()
            .context("Failed to start download")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed with status: {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);

        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("#>-"));

        let mut file = fs::File::create(output_path)
            .with_context(|| format!("Failed to create file: {}", output_path.display()))?;

        let content = response.bytes().context("Failed to read response body")?;

        file.write_all(&content)
            .context("Failed to write binary data")?;

        pb.set_position(content.len() as u64);
        pb.finish_with_message("Download complete");

        Ok(())
    }

    fn verify_checksum(&self, binary_path: &Path, expected: &str) -> Result<()> {
        let contents = fs::read(binary_path)
            .with_context(|| format!("Failed to read binary: {}", binary_path.display()))?;

        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let hash = hasher.finalize();
        let actual = hex::encode(hash);

        if actual != expected {
            anyhow::bail!(
                "Checksum verification failed!\nExpected: {}\nActual: {}",
                expected,
                actual
            );
        }

        println!("✅ Checksum verified");
        Ok(())
    }
}

impl McpServer for BinaryServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()> {
        // Validate working directory if provided
        if let Some(working_dir) = config.get("working_directory") {
            let path = Path::new(working_dir);
            if !path.exists() {
                anyhow::bail!("Working directory does not exist: {}", working_dir);
            }
            if !path.is_dir() {
                anyhow::bail!("Working directory is not a directory: {}", working_dir);
            }
        }

        // Validate timeout if provided
        if let Some(timeout_str) = config.get("timeout") {
            timeout_str
                .parse::<u64>()
                .with_context(|| format!("Invalid timeout value: {timeout_str}"))?;
        }

        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        let binary_path = self
            .binary_path
            .as_ref()
            .context("Binary not downloaded yet. Call download_and_install() first.")?;

        let command = binary_path.to_string_lossy().to_string();
        let args = vec![];

        Ok((command, args))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        Box::new(NoDependencyChecker)
    }
}

/// Dependency checker for binary servers (no dependencies required)
struct NoDependencyChecker;

impl DependencyChecker for NoDependencyChecker {
    fn check(&self) -> Result<crate::deps::DependencyCheck> {
        Ok(crate::deps::DependencyCheck {
            dependency: Dependency::NodeJs { min_version: None }, // Use a placeholder since binary has no deps
            status: DependencyStatus::Installed {
                version: Some("N/A".to_string()),
            },
            install_instructions: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_name_from_url() {
        assert_eq!(
            BinaryServer::extract_name_from_url(
                "https://github.com/user/repo/releases/download/v1.0/server-linux"
            ),
            "repo"
        );
        assert_eq!(
            BinaryServer::extract_name_from_url(
                "https://example.com/downloads/my-server?version=1.0"
            ),
            "my-server"
        );
        assert_eq!(
            BinaryServer::extract_name_from_url("https://example.com/server.exe"),
            "server.exe"
        );
    }

    #[test]
    fn test_extract_github_repo() {
        assert_eq!(
            BinaryServer::extract_github_repo(
                "https://github.com/user/repo/releases/download/v1.0/file"
            ),
            Some("user/repo".to_string())
        );
        assert_eq!(
            BinaryServer::extract_github_repo("https://example.com/file"),
            None
        );
    }

    #[test]
    fn test_binary_server_creation() {
        let server = BinaryServer::new("https://example.com/server", Some("abc123".to_string()));
        assert_eq!(server.metadata.name, "server");
        assert_eq!(server.url, "https://example.com/server");
        assert_eq!(server.checksum, Some("abc123".to_string()));
    }

    #[test]
    fn test_select_platform_asset() {
        let assets = vec![
            GitHubAsset {
                name: "server-linux-x86_64".to_string(),
                browser_download_url: "https://example.com/linux".to_string(),
                size: 1000,
            },
            GitHubAsset {
                name: "server-windows-x64.exe".to_string(),
                browser_download_url: "https://example.com/windows".to_string(),
                size: 1000,
            },
            GitHubAsset {
                name: "server-darwin-arm64".to_string(),
                browser_download_url: "https://example.com/macos".to_string(),
                size: 1000,
            },
        ];

        let result = BinaryServer::select_platform_asset(&assets);
        assert!(result.is_ok());

        // The exact result depends on the current platform, but it should find something
        let selected = result.unwrap();
        assert!(!selected.name.is_empty());
    }

    #[test]
    fn test_validate_config_timeout() {
        let server = BinaryServer::new("https://example.com/server", None);

        let mut config = HashMap::new();
        config.insert("timeout".to_string(), "30".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("timeout".to_string(), "invalid".to_string());
        assert!(server.validate_config(&config).is_err());
    }
}
