use crate::deps::{
    base::{CommonVersionParsers, DependencyCheckerBase},
    Dependency, DependencyCheck, DependencyChecker, DependencyStatus, InstallInstructions,
    InstallMethod,
};
use anyhow::Result;
use std::process::Command;

#[derive(Debug)]
pub struct PythonChecker {
    min_version: Option<String>,
}

impl PythonChecker {
    pub fn new() -> Self {
        Self { min_version: None }
    }

    pub fn with_min_version(min_version: impl Into<String>) -> Self {
        Self {
            min_version: Some(min_version.into()),
        }
    }

    fn get_python_command() -> Vec<&'static str> {
        // Try different Python commands in order of preference
        vec!["python3", "python", "py"]
    }

    fn check_python_version(&self, python_cmd: &str) -> Result<Option<String>> {
        let output = DependencyCheckerBase::get_command_version(python_cmd, &["--version"])?;

        Ok(output.and_then(|version_line| {
            CommonVersionParsers::parse_standard_format(&version_line, "Python ")
        }))
    }

    fn get_install_instructions() -> InstallInstructions {
        InstallInstructions {
            windows: vec![
                InstallMethod {
                    name: "Python.org".to_string(),
                    command: "Download and install from https://python.org/downloads/".to_string(),
                    description: Some("Official Python installer with pip included".to_string()),
                },
                InstallMethod {
                    name: "Microsoft Store".to_string(),
                    command: "Install Python from Microsoft Store".to_string(),
                    description: Some("Easy installation through Windows Store".to_string()),
                },
                InstallMethod {
                    name: "Chocolatey".to_string(),
                    command: "choco install python".to_string(),
                    description: Some("Package manager installation".to_string()),
                },
                InstallMethod {
                    name: "winget".to_string(),
                    command: "winget install Python.Python.3".to_string(),
                    description: Some("Windows Package Manager".to_string()),
                },
            ],
            macos: vec![
                InstallMethod {
                    name: "Homebrew".to_string(),
                    command: "brew install python3".to_string(),
                    description: Some("Most popular macOS package manager".to_string()),
                },
                InstallMethod {
                    name: "Python.org".to_string(),
                    command: "Download and install from https://python.org/downloads/".to_string(),
                    description: Some("Official Python installer".to_string()),
                },
                InstallMethod {
                    name: "pyenv".to_string(),
                    command: "pyenv install 3.11.0 && pyenv global 3.11.0".to_string(),
                    description: Some("Python version manager (install pyenv first)".to_string()),
                },
            ],
            linux: vec![
                InstallMethod {
                    name: "apt (Ubuntu/Debian)".to_string(),
                    command: "sudo apt update && sudo apt install python3 python3-pip".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "dnf (Fedora)".to_string(),
                    command: "sudo dnf install python3 python3-pip".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "yum (RHEL/CentOS)".to_string(),
                    command: "sudo yum install python3 python3-pip".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "pacman (Arch)".to_string(),
                    command: "sudo pacman -S python python-pip".to_string(),
                    description: Some("System package manager".to_string()),
                },
                InstallMethod {
                    name: "snap".to_string(),
                    command: "sudo snap install python3 --classic".to_string(),
                    description: Some("Universal Linux packages".to_string()),
                },
            ],
        }
    }
}

impl Default for PythonChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyChecker for PythonChecker {
    fn check(&self) -> Result<DependencyCheck> {
        let python_commands = Self::get_python_command();
        let mut found_version: Option<String> = None;
        let mut _working_command: Option<String> = None;

        // Try each Python command
        for &cmd in &python_commands {
            if let Ok(Some(version)) = self.check_python_version(cmd) {
                found_version = Some(version);
                _working_command = Some(cmd.to_string());
                break;
            }
        }

        let dependency = Dependency::Python {
            min_version: self.min_version.clone(),
        };

        let status = match found_version {
            Some(version) => {
                if let Some(ref min_version) = self.min_version {
                    DependencyCheckerBase::check_version_requirement(&version, min_version)?
                } else {
                    DependencyStatus::Installed {
                        version: Some(version),
                    }
                }
            }
            None => DependencyStatus::Missing,
        };

        let install_instructions =
            if DependencyCheckerBase::should_provide_install_instructions(&status) {
                Some(Self::get_install_instructions())
            } else {
                None
            };

        Ok(DependencyCheck {
            dependency,
            status,
            install_instructions,
        })
    }
}

/// Check if pip is available
pub fn check_pip_available() -> Result<bool> {
    let pip_commands = vec!["pip3", "pip", "python3 -m pip", "python -m pip"];

    for cmd_str in pip_commands {
        let cmd_parts: Vec<&str> = cmd_str.split_whitespace().collect();
        if cmd_parts.is_empty() {
            continue;
        }

        let test_args: Vec<&str> = cmd_parts[1..]
            .iter()
            .cloned()
            .chain(std::iter::once("--version"))
            .collect();

        if DependencyCheckerBase::is_command_available(cmd_parts[0], &test_args) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get the best pip command to use
pub fn get_pip_command() -> Result<String> {
    let pip_commands = vec!["pip3", "pip", "python3 -m pip", "python -m pip"];

    for cmd_str in pip_commands {
        let cmd_parts: Vec<&str> = cmd_str.split_whitespace().collect();
        let mut command = Command::new(cmd_parts[0]);
        for part in &cmd_parts[1..] {
            command.arg(part);
        }
        command.arg("--version");

        if let Ok(output) = command.output() {
            if output.status.success() {
                return Ok(cmd_str.to_string());
            }
        }
    }

    anyhow::bail!("No pip command found. Please install pip.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_checker_creation() {
        let checker = PythonChecker::new();
        assert!(checker.min_version.is_none());

        let checker_with_version = PythonChecker::with_min_version("3.8.0");
        assert_eq!(checker_with_version.min_version, Some("3.8.0".to_string()));
    }

    #[test]
    fn test_python_checker_default() {
        let checker = PythonChecker::default();
        assert!(checker.min_version.is_none());
    }

    #[test]
    fn test_get_python_command() {
        let commands = PythonChecker::get_python_command();
        assert!(!commands.is_empty());
        assert!(commands.contains(&"python3"));
        assert!(commands.contains(&"python"));
    }

    #[test]
    fn test_dependency_check_structure() {
        let checker = PythonChecker::new();
        let result = checker.check();

        // Should not fail to create the check structure
        assert!(result.is_ok());

        let check = result.unwrap();
        match check.dependency {
            Dependency::Python { min_version } => {
                assert_eq!(min_version, None);
            }
            _ => panic!("Expected Python dependency"),
        }
    }

    #[test]
    fn test_install_instructions() {
        let instructions = PythonChecker::get_install_instructions();

        assert!(!instructions.windows.is_empty());
        assert!(!instructions.macos.is_empty());
        assert!(!instructions.linux.is_empty());

        // Check that each platform has reasonable options
        assert!(instructions
            .windows
            .iter()
            .any(|m| m.name.contains("Python.org")));
        assert!(instructions
            .macos
            .iter()
            .any(|m| m.name.contains("Homebrew")));
        assert!(instructions.linux.iter().any(|m| m.name.contains("apt")));
    }

    #[test]
    fn test_version_parsing_scenarios() {
        let checker = PythonChecker::new();

        // This test would require mocking the Command execution
        // For now, we'll just verify the dependency is created correctly
        let result = checker.check();
        assert!(result.is_ok());
    }
}
