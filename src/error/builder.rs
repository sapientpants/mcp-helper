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
