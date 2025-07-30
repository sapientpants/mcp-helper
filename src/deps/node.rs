use crate::deps::{
    get_install_instructions, version::VersionHelper, Dependency, DependencyCheck,
    DependencyChecker, DependencyStatus,
};
use crate::logging;
use anyhow::{Context, Result};
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

        Ok(version_str.trim().to_string())
    }

    fn compare_versions(&self, installed: &str) -> Result<DependencyStatus> {
        // Parse the installed version (handles 'v' prefix)
        let installed_version = VersionHelper::parse_version(installed)?;
        let installed_str = installed_version.to_string();

        if let Some(min_required) = &self.min_version {
            // Use VersionHelper to check if the installed version satisfies the requirement
            let satisfies = VersionHelper::satisfies(&installed_str, &format!(">={min_required}"))?;

            if !satisfies {
                Ok(DependencyStatus::VersionMismatch {
                    installed: installed_str,
                    required: min_required.clone(),
                })
            } else {
                Ok(DependencyStatus::Installed {
                    version: Some(installed_str),
                })
            }
        } else {
            Ok(DependencyStatus::Installed {
                version: Some(installed_str),
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
            Some(cmd) => {
                tracing::debug!("Found Node.js command: {}", cmd);
                cmd
            }
            None => {
                logging::log_dependency_check("Node.js", "missing");
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

        // Log the dependency check result
        match &status {
            DependencyStatus::Installed { version } => {
                let version_str = version.as_deref().unwrap_or("unknown");
                logging::log_dependency_check("Node.js", &format!("installed ({version_str})"));
            }
            DependencyStatus::VersionMismatch {
                installed,
                required,
            } => {
                logging::log_dependency_check(
                    "Node.js",
                    &format!("version mismatch ({installed} < {required})"),
                );
            }
            _ => {
                logging::log_dependency_check("Node.js", "missing or invalid");
            }
        }

        // For version mismatches or missing NPX, provide install instructions
        let install_instructions = match &status {
            DependencyStatus::VersionMismatch { .. } => Some(get_install_instructions(&dependency)),
            DependencyStatus::Installed { .. } => {
                // Also check if npx is available
                if !Self::check_npx_available() {
                    tracing::warn!("Node.js installed but npx not available");
                    Some(get_install_instructions(&dependency))
                } else {
                    tracing::debug!("Node.js and npx both available");
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
        let version = VersionHelper::parse_version("16.14.0").unwrap();
        assert_eq!(version.major, 16);
        assert_eq!(version.minor, 14);
        assert_eq!(version.patch, 0);

        // Test with 'v' prefix
        let version = VersionHelper::parse_version("v16.14.0").unwrap();
        assert_eq!(version.major, 16);
        assert_eq!(version.minor, 14);
        assert_eq!(version.patch, 0);
    }

    #[test]
    fn test_version_comparison() {
        use std::cmp::Ordering;
        let ordering = VersionHelper::compare("16.0.0", "18.0.0").unwrap();
        assert_eq!(ordering, Ordering::Less);
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
