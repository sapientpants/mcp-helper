use super::{InstallInstructions, McpError};

/// Builder for creating McpError instances with less duplication
pub struct ErrorBuilder;

impl ErrorBuilder {
    /// Creates a missing dependency error with common defaults
    pub fn missing_dependency(name: &str) -> MissingDependencyBuilder {
        MissingDependencyBuilder {
            dependency: name.to_string(),
            required_version: None,
            install_instructions: InstallInstructions::default(),
        }
    }

    /// Creates a version mismatch error
    pub fn version_mismatch(name: &str) -> VersionMismatchBuilder {
        VersionMismatchBuilder {
            dependency: name.to_string(),
            installed: String::new(),
            required: String::new(),
            install_instructions: InstallInstructions::default(),
        }
    }

    /// Creates a configuration required error
    pub fn config_required(server_name: &str) -> ConfigRequiredBuilder {
        ConfigRequiredBuilder {
            server_name: server_name.to_string(),
            missing_fields: Vec::new(),
            field_descriptions: Vec::new(),
        }
    }
}

pub struct MissingDependencyBuilder {
    dependency: String,
    required_version: Option<String>,
    install_instructions: InstallInstructions,
}

impl MissingDependencyBuilder {
    pub fn version(mut self, version: &str) -> Self {
        self.required_version = Some(version.to_string());
        self
    }

    pub fn instructions(mut self, instructions: InstallInstructions) -> Self {
        self.install_instructions = instructions;
        self
    }

    pub fn build(self) -> McpError {
        McpError::MissingDependency {
            dependency: self.dependency,
            required_version: self.required_version,
            install_instructions: Box::new(self.install_instructions),
        }
    }
}

pub struct VersionMismatchBuilder {
    dependency: String,
    installed: String,
    required: String,
    install_instructions: InstallInstructions,
}

impl VersionMismatchBuilder {
    pub fn installed(mut self, version: &str) -> Self {
        self.installed = version.to_string();
        self
    }

    pub fn required(mut self, version: &str) -> Self {
        self.required = version.to_string();
        self
    }

    pub fn instructions(mut self, instructions: InstallInstructions) -> Self {
        self.install_instructions = instructions;
        self
    }

    pub fn build(self) -> McpError {
        McpError::VersionMismatch {
            dependency: self.dependency,
            current_version: self.installed,
            required_version: self.required,
            upgrade_instructions: Box::new(self.install_instructions),
        }
    }
}

pub struct ConfigRequiredBuilder {
    server_name: String,
    missing_fields: Vec<String>,
    field_descriptions: Vec<(String, String)>,
}

impl ConfigRequiredBuilder {
    pub fn field(mut self, name: &str, description: &str) -> Self {
        self.missing_fields.push(name.to_string());
        self.field_descriptions
            .push((name.to_string(), description.to_string()));
        self
    }

    pub fn fields<I, S>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<String>,
    {
        for (name, desc) in fields {
            let name_str = name.into();
            let desc_str = desc.into();
            self.missing_fields.push(name_str.clone());
            self.field_descriptions.push((name_str, desc_str));
        }
        self
    }

    pub fn build(self) -> McpError {
        McpError::ConfigurationRequired {
            server_name: self.server_name,
            missing_fields: self.missing_fields,
            field_descriptions: self.field_descriptions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod unit {
        use super::*;

        mod missing_dependency_builder {
            use super::*;

            #[test]
            fn basic() {
                let error = ErrorBuilder::missing_dependency("nodejs").build();
                match error {
                    McpError::MissingDependency {
                        dependency,
                        required_version,
                        install_instructions: _,
                    } => {
                        assert_eq!(dependency, "nodejs");
                        assert_eq!(required_version, None);
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_version() {
                let error = ErrorBuilder::missing_dependency("python")
                    .version("3.8")
                    .build();
                match error {
                    McpError::MissingDependency {
                        dependency,
                        required_version,
                        install_instructions: _,
                    } => {
                        assert_eq!(dependency, "python");
                        assert_eq!(required_version, Some("3.8".to_string()));
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_instructions() {
                let instructions = InstallInstructions {
                    windows: vec![crate::deps::InstallMethod {
                        name: "winget".to_string(),
                        command: "winget install Node.js".to_string(),
                        description: None,
                    }],
                    macos: vec![crate::deps::InstallMethod {
                        name: "brew".to_string(),
                        command: "brew install node".to_string(),
                        description: None,
                    }],
                    linux: vec![crate::deps::InstallMethod {
                        name: "apt".to_string(),
                        command: "sudo apt install nodejs".to_string(),
                        description: None,
                    }],
                };
                let error = ErrorBuilder::missing_dependency("nodejs")
                    .version("18.0")
                    .instructions(instructions.clone())
                    .build();
                match error {
                    McpError::MissingDependency {
                        dependency,
                        required_version,
                        install_instructions,
                    } => {
                        assert_eq!(dependency, "nodejs");
                        assert_eq!(required_version, Some("18.0".to_string()));
                        assert_eq!(
                            install_instructions.windows.len(),
                            instructions.windows.len()
                        );
                    }
                    _ => panic!("Unexpected error type"),
                }
            }
        }

        mod version_mismatch_builder {
            use super::*;

            #[test]
            fn basic() {
                let error = ErrorBuilder::version_mismatch("git").build();
                match error {
                    McpError::VersionMismatch {
                        dependency,
                        current_version,
                        required_version,
                        upgrade_instructions: _,
                    } => {
                        assert_eq!(dependency, "git");
                        assert_eq!(current_version, "");
                        assert_eq!(required_version, "");
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_versions() {
                let error = ErrorBuilder::version_mismatch("docker")
                    .installed("20.10.0")
                    .required("24.0.0")
                    .build();
                match error {
                    McpError::VersionMismatch {
                        dependency,
                        current_version,
                        required_version,
                        upgrade_instructions: _,
                    } => {
                        assert_eq!(dependency, "docker");
                        assert_eq!(current_version, "20.10.0");
                        assert_eq!(required_version, "24.0.0");
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_instructions() {
                let instructions = InstallInstructions {
                    windows: vec![crate::deps::InstallMethod {
                        name: "manual".to_string(),
                        command: "Update Docker Desktop".to_string(),
                        description: None,
                    }],
                    macos: vec![crate::deps::InstallMethod {
                        name: "brew".to_string(),
                        command: "brew upgrade docker".to_string(),
                        description: None,
                    }],
                    linux: vec![crate::deps::InstallMethod {
                        name: "apt".to_string(),
                        command: "sudo apt update && sudo apt upgrade docker".to_string(),
                        description: None,
                    }],
                };
                let error = ErrorBuilder::version_mismatch("docker")
                    .installed("20.10.0")
                    .required("24.0.0")
                    .instructions(instructions.clone())
                    .build();
                match error {
                    McpError::VersionMismatch {
                        dependency,
                        current_version,
                        required_version,
                        upgrade_instructions,
                    } => {
                        assert_eq!(dependency, "docker");
                        assert_eq!(current_version, "20.10.0");
                        assert_eq!(required_version, "24.0.0");
                        assert_eq!(upgrade_instructions.macos.len(), instructions.macos.len());
                    }
                    _ => panic!("Unexpected error type"),
                }
            }
        }

        mod config_required_builder {
            use super::*;

            #[test]
            fn basic() {
                let error = ErrorBuilder::config_required("test-server").build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "test-server");
                        assert!(missing_fields.is_empty());
                        assert!(field_descriptions.is_empty());
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_field() {
                let error = ErrorBuilder::config_required("api-server")
                    .field("api_key", "API key for authentication")
                    .build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "api-server");
                        assert_eq!(missing_fields, vec!["api_key"]);
                        assert_eq!(
                            field_descriptions,
                            vec![(
                                "api_key".to_string(),
                                "API key for authentication".to_string()
                            )]
                        );
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_multiple_fields() {
                let error = ErrorBuilder::config_required("db-server")
                    .field("host", "Database host")
                    .field("port", "Database port")
                    .field("username", "Database username")
                    .build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "db-server");
                        assert_eq!(missing_fields, vec!["host", "port", "username"]);
                        assert_eq!(field_descriptions.len(), 3);
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_fields_iterator() {
                let fields = vec![
                    ("url", "Server URL"),
                    ("token", "Auth token"),
                    ("timeout", "Request timeout"),
                ];
                let error = ErrorBuilder::config_required("web-server")
                    .fields(fields.clone())
                    .build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "web-server");
                        assert_eq!(missing_fields, vec!["url", "token", "timeout"]);
                        assert_eq!(field_descriptions.len(), 3);
                        assert_eq!(field_descriptions[0].0, "url");
                        assert_eq!(field_descriptions[0].1, "Server URL");
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn mixed_fields() {
                let error = ErrorBuilder::config_required("complex-server")
                    .field("primary", "Primary config")
                    .fields(vec![
                        ("secondary", "Secondary config"),
                        ("tertiary", "Tertiary config"),
                    ])
                    .field("final", "Final config")
                    .build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "complex-server");
                        assert_eq!(
                            missing_fields,
                            vec!["primary", "secondary", "tertiary", "final"]
                        );
                        assert_eq!(field_descriptions.len(), 4);
                    }
                    _ => panic!("Unexpected error type"),
                }
            }

            #[test]
            fn with_string_types() {
                // Test that various string types work with the fields method
                let fields: Vec<(String, String)> = vec![
                    ("field1".to_string(), "desc1".to_string()),
                    ("field2".to_string(), "desc2".to_string()),
                ];
                let error = ErrorBuilder::config_required("string-server")
                    .fields(fields)
                    .build();
                match error {
                    McpError::ConfigurationRequired {
                        server_name,
                        missing_fields,
                        field_descriptions,
                    } => {
                        assert_eq!(server_name, "string-server");
                        assert_eq!(missing_fields.len(), 2);
                        assert_eq!(field_descriptions.len(), 2);
                    }
                    _ => panic!("Unexpected error type"),
                }
            }
        }
    }
}
