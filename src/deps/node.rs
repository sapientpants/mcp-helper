use crate::deps::{
    get_install_instructions, Dependency, DependencyCheck, DependencyChecker, DependencyStatus,
};
use anyhow::{Context, Result};
use semver::Version;
use std::process::Command;
use which::which;

#[derive(Debug)]
pub struct NodeChecker {
    min_version: Option<String>,
}

impl NodeChecker {
    pub fn new() -> Self {
        Self { min_version: None }
    }

    pub fn with_min_version(mut self, version: String) -> Self {
        self.min_version = Some(version);
        self
    }

    fn check_node_command() -> Option<String> {
        // Try to find node executable
        if which("node").is_ok() {
            Some("node".to_string())
        } else {
            None
        }
    }

    fn get_node_version(node_cmd: &str) -> Result<String> {
        let output = Command::new(node_cmd)
            .arg("--version")
            .output()
            .context("Failed to execute node --version")?;

        if !output.status.success() {
            anyhow::bail!("node --version failed with status: {}", output.status);
        }

        let version_str = String::from_utf8(output.stdout)
            .context("Failed to parse node version output as UTF-8")?;

        // Node outputs version as "v16.14.0", we need to strip the 'v'
        let version_str = version_str.trim();
        let version_str = version_str.strip_prefix('v').unwrap_or(version_str);

        Ok(version_str.to_string())
    }

    fn compare_versions(&self, installed: &str) -> Result<DependencyStatus> {
        if let Some(min_required) = &self.min_version {
            let installed_version =
                Version::parse(installed).context("Failed to parse installed Node.js version")?;

            let required_version =
                Version::parse(min_required).context("Failed to parse required Node.js version")?;

            if installed_version < required_version {
                Ok(DependencyStatus::VersionMismatch {
                    installed: installed.to_string(),
                    required: min_required.clone(),
                })
            } else {
                Ok(DependencyStatus::Installed {
                    version: Some(installed.to_string()),
                })
            }
        } else {
            Ok(DependencyStatus::Installed {
                version: Some(installed.to_string()),
            })
        }
    }

    fn check_npx_available() -> bool {
        #[cfg(target_os = "windows")]
        return which("npx.cmd").is_ok() || which("npx").is_ok();

        #[cfg(not(target_os = "windows"))]
        return which("npx").is_ok();
    }
}

impl Default for NodeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyChecker for NodeChecker {
    fn check(&self) -> Result<DependencyCheck> {
        let dependency = Dependency::NodeJs {
            min_version: self.min_version.clone(),
        };

        let node_cmd = match Self::check_node_command() {
            Some(cmd) => cmd,
            None => {
                return Ok(DependencyCheck {
                    dependency: dependency.clone(),
                    status: DependencyStatus::Missing,
                    install_instructions: Some(get_install_instructions(&dependency)),
                });
            }
        };

        // Get Node.js version
        let version = match Self::get_node_version(&node_cmd) {
            Ok(v) => v,
            Err(_e) => {
                return Ok(DependencyCheck {
                    dependency: dependency.clone(),
                    status: DependencyStatus::Missing,
                    install_instructions: Some(get_install_instructions(&dependency)),
                });
            }
        };

        // Compare versions if required
        let status = self.compare_versions(&version)?;

        // For version mismatches or missing NPX, provide install instructions
        let install_instructions = match &status {
            DependencyStatus::VersionMismatch { .. } => Some(get_install_instructions(&dependency)),
            DependencyStatus::Installed { .. } => {
                // Also check if npx is available
                if !Self::check_npx_available() {
                    Some(get_install_instructions(&dependency))
                } else {
                    None
                }
            }
            _ => Some(get_install_instructions(&dependency)),
        };

        Ok(DependencyCheck {
            dependency,
            status,
            install_instructions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_checker_new() {
        let checker = NodeChecker::new();
        assert!(checker.min_version.is_none());
    }

    #[test]
    fn test_node_checker_with_min_version() {
        let checker = NodeChecker::new().with_min_version("16.0.0".to_string());
        assert_eq!(checker.min_version, Some("16.0.0".to_string()));
    }

    #[test]
    fn test_node_checker_default() {
        let checker = NodeChecker::default();
        assert!(checker.min_version.is_none());
    }

    #[test]
    fn test_version_parsing() {
        // Test that version strings can be parsed
        let version = Version::parse("16.14.0").unwrap();
        assert_eq!(version.major, 16);
        assert_eq!(version.minor, 14);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("16.0.0").unwrap();
        let v2 = Version::parse("18.0.0").unwrap();
        assert!(v1 < v2);
    }

    #[test]
    fn test_compare_versions_no_requirement() {
        let checker = NodeChecker::new();
        let status = checker.compare_versions("16.14.0").unwrap();
        match status {
            DependencyStatus::Installed { version } => {
                assert_eq!(version, Some("16.14.0".to_string()));
            }
            _ => panic!("Expected Installed status"),
        }
    }

    #[test]
    fn test_compare_versions_meets_requirement() {
        let checker = NodeChecker::new().with_min_version("16.0.0".to_string());
        let status = checker.compare_versions("18.0.0").unwrap();
        match status {
            DependencyStatus::Installed { version } => {
                assert_eq!(version, Some("18.0.0".to_string()));
            }
            _ => panic!("Expected Installed status"),
        }
    }

    #[test]
    fn test_compare_versions_below_requirement() {
        let checker = NodeChecker::new().with_min_version("18.0.0".to_string());
        let status = checker.compare_versions("16.0.0").unwrap();
        match status {
            DependencyStatus::VersionMismatch {
                installed,
                required,
            } => {
                assert_eq!(installed, "16.0.0");
                assert_eq!(required, "18.0.0");
            }
            _ => panic!("Expected VersionMismatch status"),
        }
    }
}
