//! Base functionality and common patterns for dependency checkers

use super::{DependencyStatus, InstallInstructions, InstallMethod};
use anyhow::{Context, Result};
use std::process::Command;

/// Common functionality for dependency checkers
pub struct DependencyCheckerBase;

impl DependencyCheckerBase {
    /// Execute a command and return its version output if successful
    pub fn get_command_version(command: &str, args: &[&str]) -> Result<Option<String>> {
        let output = Command::new(command)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute {command}"))?;

        if !output.status.success() {
            return Ok(None);
        }

        let version_output = String::from_utf8_lossy(&output.stdout);
        Ok(Some(version_output.trim().to_string()))
    }

    /// Check if a command is available by trying to run it
    pub fn is_command_available(command: &str, test_args: &[&str]) -> bool {
        Command::new(command)
            .args(test_args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Compare installed version against a minimum required version
    pub fn check_version_requirement(installed: &str, required: &str) -> Result<DependencyStatus> {
        let installed_version = semver::Version::parse(installed)
            .with_context(|| format!("Invalid installed version format: {installed}"))?;
        let required_version = semver::Version::parse(required)
            .with_context(|| format!("Invalid required version format: {required}"))?;

        if installed_version >= required_version {
            Ok(DependencyStatus::Installed {
                version: Some(installed.to_string()),
            })
        } else {
            Ok(DependencyStatus::VersionMismatch {
                installed: installed.to_string(),
                required: required.to_string(),
            })
        }
    }

    /// Determine if install instructions should be provided based on status
    pub fn should_provide_install_instructions(status: &DependencyStatus) -> bool {
        matches!(
            status,
            DependencyStatus::Missing | DependencyStatus::VersionMismatch { .. }
        )
    }

    /// Create a platform-specific install method
    pub fn create_install_method(
        name: impl Into<String>,
        command: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> InstallMethod {
        InstallMethod {
            name: name.into(),
            command: command.into(),
            description: description.map(|d| d.into()),
        }
    }

    /// Create install instructions with all platforms empty
    pub fn empty_install_instructions() -> InstallInstructions {
        InstallInstructions {
            windows: vec![],
            macos: vec![],
            linux: vec![],
        }
    }
}

/// Helper trait for parsing version strings from command output
pub trait VersionParser {
    /// Extract version from command output
    fn parse_version_output(&self, output: &str) -> Option<String>;
}

/// Common version parsing patterns
pub struct CommonVersionParsers;

impl CommonVersionParsers {
    /// Parse version from "Program version X.Y.Z" format
    pub fn parse_standard_format(output: &str, prefix: &str) -> Option<String> {
        output
            .strip_prefix(prefix)
            .and_then(|version_part| {
                // Handle cases with additional info after version
                if let Some(comma_pos) = version_part.find(',') {
                    Some(&version_part[..comma_pos])
                } else {
                    version_part
                        .split_whitespace()
                        .next()
                        .or(Some(version_part))
                }
            })
            .map(|v| v.trim().to_string())
    }

    /// Parse semantic version (X.Y.Z) from arbitrary text
    pub fn extract_semver(text: &str) -> Option<String> {
        // Simple regex-like pattern matching for semantic versions
        let parts: Vec<&str> = text.split_whitespace().collect();
        for part in parts {
            if part.chars().filter(|&c| c == '.').count() >= 1
                && part.chars().any(|c| c.is_ascii_digit())
            {
                // Clean up the version string
                let clean_version = part.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
                if !clean_version.is_empty() {
                    return Some(clean_version.to_string());
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let status = DependencyCheckerBase::check_version_requirement("2.0.0", "1.5.0").unwrap();
        assert!(matches!(status, DependencyStatus::Installed { .. }));

        let status = DependencyCheckerBase::check_version_requirement("1.0.0", "1.5.0").unwrap();
        assert!(matches!(status, DependencyStatus::VersionMismatch { .. }));
    }

    #[test]
    fn test_should_provide_install_instructions() {
        assert!(DependencyCheckerBase::should_provide_install_instructions(
            &DependencyStatus::Missing
        ));
        assert!(DependencyCheckerBase::should_provide_install_instructions(
            &DependencyStatus::VersionMismatch {
                installed: "1.0.0".to_string(),
                required: "2.0.0".to_string(),
            }
        ));
        assert!(!DependencyCheckerBase::should_provide_install_instructions(
            &DependencyStatus::Installed {
                version: Some("2.0.0".to_string())
            }
        ));
    }

    #[test]
    fn test_parse_standard_format() {
        assert_eq!(
            CommonVersionParsers::parse_standard_format("Python 3.9.0", "Python "),
            Some("3.9.0".to_string())
        );
        assert_eq!(
            CommonVersionParsers::parse_standard_format(
                "Docker version 20.10.0, build abcdef",
                "Docker version "
            ),
            Some("20.10.0".to_string())
        );
    }

    #[test]
    fn test_extract_semver() {
        assert_eq!(
            CommonVersionParsers::extract_semver("version 1.2.3"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            CommonVersionParsers::extract_semver("v3.10.0"),
            Some("3.10.0".to_string())
        );
    }
}
