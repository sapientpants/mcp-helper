use crate::deps::{DependencyChecker, PythonChecker};
use crate::server::{ConfigField, ConfigFieldType, McpServer, ServerMetadata, ServerType};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct PythonServer {
    metadata: ServerMetadata,
    package: String,
    version: Option<String>,
    script_path: Option<String>,
    min_python_version: Option<String>,
}

impl PythonServer {
    pub fn new(package_spec: &str) -> Result<Self> {
        let (package, version) = Self::parse_package_spec(package_spec);

        let metadata = ServerMetadata {
            name: package.clone(),
            description: Some(format!("Python MCP server: {package}")),
            server_type: ServerType::Python {
                package: package.clone(),
                version: version.clone(),
            },
            required_config: vec![],
            optional_config: vec![
                ConfigField {
                    name: "python_path".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Path to Python interpreter".to_string()),
                    default: Some("python3".to_string()),
                },
                ConfigField {
                    name: "working_directory".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Working directory for the server".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "virtual_env".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Path to virtual environment".to_string()),
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

        Ok(Self {
            metadata,
            package,
            version,
            script_path: None,
            min_python_version: Some("3.8.0".to_string()), // Default minimum Python version
        })
    }

    pub fn from_script(script_path: &str, min_python_version: Option<String>) -> Self {
        let name = Path::new(script_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("python-server")
            .to_string();

        let metadata = ServerMetadata {
            name: name.clone(),
            description: Some(format!("Python script: {script_path}")),
            server_type: ServerType::Python {
                package: script_path.to_string(),
                version: None,
            },
            required_config: vec![],
            optional_config: vec![
                ConfigField {
                    name: "python_path".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Path to Python interpreter".to_string()),
                    default: Some("python3".to_string()),
                },
                ConfigField {
                    name: "working_directory".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Working directory for the server".to_string()),
                    default: None,
                },
                ConfigField {
                    name: "virtual_env".to_string(),
                    field_type: ConfigFieldType::Path,
                    description: Some("Path to virtual environment".to_string()),
                    default: None,
                },
            ],
        };

        Self {
            metadata,
            package: script_path.to_string(),
            version: None,
            script_path: Some(script_path.to_string()),
            min_python_version,
        }
    }

    fn parse_package_spec(package_spec: &str) -> (String, Option<String>) {
        if let Some(eq_pos) = package_spec.find("==") {
            // Handle pip-style version specification: package==1.0.0
            let package = package_spec[..eq_pos].to_string();
            let version = Some(package_spec[eq_pos + 2..].to_string());
            (package, version)
        } else if let Some(at_pos) = package_spec.rfind('@') {
            // Handle npm-style version specification: package@1.0.0
            let package = package_spec[..at_pos].to_string();
            let version = Some(package_spec[at_pos + 1..].to_string());
            (package, version)
        } else {
            // No version specified
            (package_spec.to_string(), None)
        }
    }

    pub fn with_min_python_version(mut self, version: impl Into<String>) -> Self {
        self.min_python_version = Some(version.into());
        self
    }

    fn get_python_command(&self, config: &HashMap<String, String>) -> String {
        if let Some(python_path) = config.get("python_path") {
            python_path.clone()
        } else {
            "python3".to_string()
        }
    }

    fn get_virtual_env_command(&self, config: &HashMap<String, String>) -> Option<String> {
        config.get("virtual_env").map(|venv_path| {
            #[cfg(windows)]
            {
                format!("{}/Scripts/python.exe", venv_path)
            }
            #[cfg(not(windows))]
            {
                format!("{venv_path}/bin/python")
            }
        })
    }

    #[allow(dead_code)]
    fn is_package_installed(&self, python_cmd: &str) -> Result<bool> {
        let output = std::process::Command::new(python_cmd)
            .arg("-c")
            .arg(format!("import {}", self.package))
            .output()
            .context("Failed to check if Python package is installed")?;

        Ok(output.status.success())
    }

    pub fn install_package(&self, _python_cmd: &str) -> Result<()> {
        println!("📦 Installing Python package: {}", self.package);

        let pip_cmd = crate::deps::python::get_pip_command()?;
        let package_spec = if let Some(ref version) = self.version {
            format!("{}=={}", self.package, version)
        } else {
            self.package.clone()
        };

        let mut command = std::process::Command::new(pip_cmd.split_whitespace().next().unwrap());

        // Add pip command parts (handle cases like "python3 -m pip")
        let pip_parts: Vec<&str> = pip_cmd.split_whitespace().collect();
        for part in &pip_parts[1..] {
            command.arg(part);
        }

        command.args(["install", &package_spec]);

        let output = command
            .output()
            .context("Failed to install Python package")?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install package {}: {}", package_spec, error_msg);
        }

        println!("✅ Successfully installed {package_spec}");
        Ok(())
    }
}

impl McpServer for PythonServer {
    fn metadata(&self) -> &ServerMetadata {
        &self.metadata
    }

    fn validate_config(&self, config: &HashMap<String, String>) -> Result<()> {
        // Validate Python path if provided
        if let Some(python_path) = config.get("python_path") {
            let output = std::process::Command::new(python_path)
                .arg("--version")
                .output();

            match output {
                Ok(output) if output.status.success() => {}
                _ => anyhow::bail!("Invalid Python path: {}", python_path),
            }
        }

        // Validate virtual environment if provided
        if let Some(_venv_path) = config.get("virtual_env") {
            let venv_python = self.get_virtual_env_command(config).unwrap();
            let venv_path_obj = Path::new(&venv_python);

            if !venv_path_obj.exists() {
                anyhow::bail!("Virtual environment Python not found: {}", venv_python);
            }
        }

        // Use common validation utilities
        use super::validation::ConfigValidation;
        ConfigValidation::validate_working_directory(config)?;
        ConfigValidation::validate_timeout(config)?;

        Ok(())
    }

    fn generate_command(&self) -> Result<(String, Vec<String>)> {
        let config = HashMap::new(); // Use default config for command generation

        let python_cmd = if let Some(venv_cmd) = self.get_virtual_env_command(&config) {
            venv_cmd
        } else {
            self.get_python_command(&config)
        };

        let args = if let Some(ref script_path) = self.script_path {
            // For script files, run the script directly
            vec![script_path.clone()]
        } else {
            // For packages, use -m to run as module
            vec!["-m".to_string(), self.package.clone()]
        };

        Ok((python_cmd, args))
    }

    fn dependency(&self) -> Box<dyn DependencyChecker> {
        if let Some(ref min_version) = self.min_python_version {
            Box::new(PythonChecker::with_min_version(min_version.clone()))
        } else {
            Box::new(PythonChecker::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_package_spec() {
        // Test pip-style version specification
        let (package, version) = PythonServer::parse_package_spec("requests==2.25.1");
        assert_eq!(package, "requests");
        assert_eq!(version, Some("2.25.1".to_string()));

        // Test npm-style version specification
        let (package, version) = PythonServer::parse_package_spec("requests@2.25.1");
        assert_eq!(package, "requests");
        assert_eq!(version, Some("2.25.1".to_string()));

        // Test no version
        let (package, version) = PythonServer::parse_package_spec("requests");
        assert_eq!(package, "requests");
        assert_eq!(version, None);

        // Test complex package names
        let (package, version) = PythonServer::parse_package_spec("scikit-learn==1.0.0");
        assert_eq!(package, "scikit-learn");
        assert_eq!(version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_python_server_creation() {
        let server = PythonServer::new("requests").unwrap();
        assert_eq!(server.metadata.name, "requests");
        assert_eq!(server.package, "requests");
        assert_eq!(server.version, None);
        assert!(server.script_path.is_none());
    }

    #[test]
    fn test_python_server_with_version() {
        let server = PythonServer::new("requests==2.25.1").unwrap();
        assert_eq!(server.metadata.name, "requests");
        assert_eq!(server.package, "requests");
        assert_eq!(server.version, Some("2.25.1".to_string()));
    }

    #[test]
    fn test_python_server_from_script() {
        let server = PythonServer::from_script("/path/to/server.py", Some("3.9.0".to_string()));
        assert_eq!(server.metadata.name, "server");
        assert_eq!(server.package, "/path/to/server.py");
        assert_eq!(server.script_path, Some("/path/to/server.py".to_string()));
        assert_eq!(server.min_python_version, Some("3.9.0".to_string()));
    }

    #[test]
    fn test_get_python_command() {
        let server = PythonServer::new("test").unwrap();

        // Test default
        let config = HashMap::new();
        assert_eq!(server.get_python_command(&config), "python3");

        // Test custom python path
        let mut config = HashMap::new();
        config.insert("python_path".to_string(), "/usr/bin/python3.9".to_string());
        assert_eq!(server.get_python_command(&config), "/usr/bin/python3.9");
    }

    #[test]
    fn test_get_virtual_env_command() {
        let server = PythonServer::new("test").unwrap();

        // Test no virtual env
        let config = HashMap::new();
        assert_eq!(server.get_virtual_env_command(&config), None);

        // Test with virtual env
        let mut config = HashMap::new();
        config.insert("virtual_env".to_string(), "/path/to/venv".to_string());
        let venv_cmd = server.get_virtual_env_command(&config).unwrap();

        #[cfg(windows)]
        assert_eq!(venv_cmd, "/path/to/venv/Scripts/python.exe");

        #[cfg(not(windows))]
        assert_eq!(venv_cmd, "/path/to/venv/bin/python");
    }

    #[test]
    fn test_validate_config_timeout() {
        let server = PythonServer::new("test").unwrap();

        let mut config = HashMap::new();
        config.insert("timeout".to_string(), "30".to_string());
        assert!(server.validate_config(&config).is_ok());

        config.insert("timeout".to_string(), "invalid".to_string());
        assert!(server.validate_config(&config).is_err());
    }

    #[test]
    fn test_generate_command_package() {
        let server = PythonServer::new("mypackage").unwrap();
        let (cmd, args) = server.generate_command().unwrap();

        assert_eq!(cmd, "python3");
        assert_eq!(args, vec!["-m", "mypackage"]);
    }

    #[test]
    fn test_generate_command_script() {
        let server = PythonServer::from_script("/path/to/script.py", None);
        let (cmd, args) = server.generate_command().unwrap();

        assert_eq!(cmd, "python3");
        assert_eq!(args, vec!["/path/to/script.py"]);
    }

    #[test]
    fn test_with_min_python_version() {
        let server = PythonServer::new("test")
            .unwrap()
            .with_min_python_version("3.9.0");

        assert_eq!(server.min_python_version, Some("3.9.0".to_string()));
    }
}
