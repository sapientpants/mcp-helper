use crate::deps::{
    base::{CommonVersionParsers, DependencyCheckerBase},
    Dependency, DependencyCheck, DependencyChecker, DependencyStatus, InstallInstructions,
    InstallMethod,
};
use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug)]
pub struct DockerChecker {
    min_version: Option<String>,
    check_compose: bool,
}

impl DockerChecker {
    pub fn new() -> Self {
        Self {
            min_version: None,
            check_compose: false,
        }
    }

    pub fn with_min_version(min_version: impl Into<String>) -> Self {
        Self {
            min_version: Some(min_version.into()),
            check_compose: false,
        }
    }

    pub fn with_compose_check(mut self) -> Self {
        self.check_compose = true;
        self
    }

    fn check_docker_version(&self) -> Result<Option<String>> {
        let output = DependencyCheckerBase::get_command_version("docker", &["--version"])?;

        Ok(output.and_then(|version_line| {
            CommonVersionParsers::parse_standard_format(&version_line, "Docker version ")
        }))
    }

    fn check_docker_running(&self) -> Result<bool> {
        Ok(DependencyCheckerBase::is_command_available(
            "docker",
            &["info"],
        ))
    }

    fn check_docker_compose(&self) -> Result<Option<String>> {
        // Try docker compose (new syntax) first
        if let Some(version) = self.try_docker_compose_new_syntax()? {
            return Ok(Some(version));
        }

        // Try docker-compose (legacy syntax)
        self.try_docker_compose_legacy_syntax()
    }

    fn try_docker_compose_new_syntax(&self) -> Result<Option<String>> {
        let output = Command::new("docker")
            .args(["compose", "version"])
            .output()
            .context("Failed to execute docker compose version")?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Some(version_line) = version_output.lines().next() {
                return Ok(self.parse_docker_compose_new_format(version_line));
            }
        }

        Ok(None)
    }

    fn try_docker_compose_legacy_syntax(&self) -> Result<Option<String>> {
        let output = Command::new("docker-compose")
            .args(["--version"])
            .output()
            .context("Failed to execute docker-compose --version")?;

        if output.status.success() {
            let version_output = String::from_utf8_lossy(&output.stdout);
            if let Some(version_line) = version_output.lines().next() {
                return Ok(self.parse_docker_compose_legacy_format(version_line));
            }
        }

        Ok(None)
    }

    fn parse_docker_compose_new_format(&self, version_line: &str) -> Option<String> {
        // Parse "Docker Compose version vX.Y.Z" format
        version_line
            .strip_prefix("Docker Compose version v")
            .map(|version_part| version_part.trim().to_string())
    }

    fn parse_docker_compose_legacy_format(&self, version_line: &str) -> Option<String> {
        // Parse "docker-compose version X.Y.Z, build abcdef" format
        version_line
            .strip_prefix("docker-compose version ")
            .map(|version_part| {
                if let Some(comma_pos) = version_part.find(',') {
                    version_part[..comma_pos].to_string()
                } else {
                    version_part.trim().to_string()
                }
            })
    }

    fn get_install_instructions() -> InstallInstructions {
        InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "Docker Desktop".to_string(),
                    command: "Download and install from https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe".to_string(),
                    description: Some("Official Docker Desktop for Windows with GUI".to_string()),
                },
                InstallMethod {
                    name: "Chocolatey".to_string(),
                    command: "choco install docker-desktop".to_string(),
                    description: Some("Package manager installation".to_string()),
                },
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install Docker.DockerDesktop".to_string(),
                    description: Some("Windows Package Manager".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "Docker Desktop".to_string(),
                    command: "Download and install from https://desktop.docker.com/mac/main/amd64/Docker.dmg".to_string(),
                    description: Some("Official Docker Desktop for macOS with GUI".to_string()),
                },
                InstallMethod {
                    name: "Homebrew".to_string(),
                    command: "brew install --cask docker".to_string(),
                    description: Some("Package manager installation".to_string()),
                },
                InstallMethod {
                    name: "Homebrew (CLI only)".to_string(),
                    command: "brew install docker docker-compose".to_string(),
                    description: Some("Command-line only (requires separate Docker daemon)".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "Docker Engine (Ubuntu/Debian)".to_string(),
                    command: "curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh".to_string(),
                    description: Some("Official Docker installation script".to_string()),
                },
                InstallMethod {
                    name: "apt (Ubuntu/Debian)".to_string(),
                    command: "sudo apt update && sudo apt install docker.io docker-compose".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "dnf (Fedora)".to_string(),
                    command: "sudo dnf install docker docker-compose".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "yum (RHEL/CentOS)".to_string(),
                    command: "sudo yum install docker docker-compose".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "pacman (Arch)".to_string(),
                    command: "sudo pacman -S docker docker-compose".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "snap".to_string(),
                    command: "sudo snap install docker".to_string(),
                    description: Some("Universal Linux packages".to_string()),
                },
            ],
        }
    }
}

impl Default for DockerChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyChecker for DockerChecker {
    fn check(&self) -> Result<DependencyCheck> {
        let dependency = Dependency::Docker {
            min_version: self.min_version.clone(),
            requires_compose: self.check_compose,
        };

        let docker_version = self.check_docker_version()?;
        let status = self.determine_status(docker_version)?;
        let install_instructions = self.get_install_instructions_if_needed(&status);

        Ok(DependencyCheck {
            dependency,
            status,
            install_instructions,
        })
    }
}

impl DockerChecker {
    fn determine_status(&self, docker_version: Option<String>) -> Result<DependencyStatus> {
        match docker_version {
            Some(version) => self.check_installed_docker(&version),
            None => Ok(DependencyStatus::Missing),
        }
    }

    fn check_installed_docker(&self, version: &str) -> Result<DependencyStatus> {
        if !self.check_docker_running()? {
            return Ok(DependencyStatus::ConfigurationRequired {
                issue: "Docker is installed but not running".to_string(),
                solution: "Start Docker Desktop or run 'sudo systemctl start docker'".to_string(),
            });
        }

        if let Some(ref min_version) = self.min_version {
            self.check_version_requirement(version, min_version)
        } else {
            self.check_compose_requirement(version)
        }
    }

    fn check_version_requirement(
        &self,
        version: &str,
        min_version: &str,
    ) -> Result<DependencyStatus> {
        let status = DependencyCheckerBase::check_version_requirement(version, min_version)?;

        // If version is OK, still need to check compose requirement
        if matches!(status, DependencyStatus::Installed { .. }) {
            self.check_compose_requirement(version)
        } else {
            Ok(status)
        }
    }

    fn check_compose_requirement(&self, version: &str) -> Result<DependencyStatus> {
        if !self.check_compose {
            return Ok(DependencyStatus::Installed {
                version: Some(version.to_string()),
            });
        }

        match self.check_docker_compose()? {
            Some(_) => Ok(DependencyStatus::Installed {
                version: Some(version.to_string()),
            }),
            None => Ok(DependencyStatus::ConfigurationRequired {
                issue: "Docker Compose is not available".to_string(),
                solution: "Install Docker Compose or use Docker Desktop".to_string(),
            }),
        }
    }

    fn get_install_instructions_if_needed(
        &self,
        status: &DependencyStatus,
    ) -> Option<InstallInstructions> {
        if DependencyCheckerBase::should_provide_install_instructions(status) {
            Some(Self::get_install_instructions())
        } else {
            None
        }
    }
}

/// Check if Docker is available and running
pub fn check_docker_available() -> Result<bool> {
    let checker = DockerChecker::new();
    let check_result = checker.check()?;

    Ok(matches!(
        check_result.status,
        DependencyStatus::Installed { .. }
    ))
}

/// Check if Docker Compose is available
pub fn check_compose_available() -> Result<bool> {
    let checker = DockerChecker::new().with_compose_check();
    let check_result = checker.check()?;

    Ok(matches!(
        check_result.status,
        DependencyStatus::Installed { .. }
    ))
}

/// Get the best docker command to use (docker or podman)
pub fn get_container_runtime() -> Result<String> {
    // Check for Docker first
    if let Ok(output) = Command::new("docker").arg("--version").output() {
        if output.status.success() {
            return Ok("docker".to_string());
        }
    }

    // Check for Podman as alternative
    if let Ok(output) = Command::new("podman").arg("--version").output() {
        if output.status.success() {
            return Ok("podman".to_string());
        }
    }

    anyhow::bail!("No container runtime found. Please install Docker or Podman.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_checker_creation() {
        let checker = DockerChecker::new();
        assert!(checker.min_version.is_none());
        assert!(!checker.check_compose);

        let checker_with_version = DockerChecker::with_min_version("20.10.0");
        assert_eq!(
            checker_with_version.min_version,
            Some("20.10.0".to_string())
        );
        assert!(!checker_with_version.check_compose);
    }

    #[test]
    fn test_docker_checker_with_compose() {
        let checker = DockerChecker::new().with_compose_check();
        assert!(checker.check_compose);
    }

    #[test]
    fn test_docker_checker_default() {
        let checker = DockerChecker::default();
        assert!(checker.min_version.is_none());
        assert!(!checker.check_compose);
    }

    #[test]
    fn test_dependency_check_structure() {
        let checker = DockerChecker::new();

        // Test the structure without actually checking Docker availability
        let dependency = Dependency::Docker {
            min_version: checker.min_version.clone(),
            requires_compose: checker.check_compose,
        };

        match dependency {
            Dependency::Docker {
                min_version,
                requires_compose,
            } => {
                assert_eq!(min_version, None);
                assert!(!requires_compose);
            }
            _ => panic!("Expected Docker dependency"),
        }
    }

    #[test]
    fn test_install_instructions() {
        let instructions = DockerChecker::get_install_instructions();

        assert!(!instructions.windows.is_empty());
        assert!(!instructions.macos.is_empty());
        assert!(!instructions.linux.is_empty());

        // Check that each platform has Docker Desktop option
        assert!(instructions
            .windows
            .iter()
            .any(|m| m.name.contains("Docker Desktop")));
        assert!(instructions
            .macos
            .iter()
            .any(|m| m.name.contains("Docker Desktop")));
        assert!(instructions
            .linux
            .iter()
            .any(|m| m.name.contains("Docker Engine")));
    }

    #[test]
    fn test_version_parsing_scenarios() {
        let checker = DockerChecker::new();
        // Test that we can create a checker and it has expected defaults
        assert!(checker.min_version.is_none());
        assert!(!checker.check_compose);

        // Test with version requirement
        let checker_with_version = DockerChecker::with_min_version("20.10.0");
        assert_eq!(
            checker_with_version.min_version,
            Some("20.10.0".to_string())
        );
    }

    #[test]
    fn test_container_runtime_detection() {
        // This test would require mocking commands
        // For now, we'll just ensure the function exists and can be called
        let result = get_container_runtime();
        // We can't assert the result since it depends on system state
        // but we can ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_compose_checker_structure() {
        let checker = DockerChecker::new().with_compose_check();

        // Test the structure without actually checking Docker availability
        let dependency = Dependency::Docker {
            min_version: checker.min_version.clone(),
            requires_compose: checker.check_compose,
        };

        match dependency {
            Dependency::Docker {
                requires_compose, ..
            } => {
                assert!(requires_compose);
            }
            _ => panic!("Expected Docker dependency with compose requirement"),
        }
    }
}
