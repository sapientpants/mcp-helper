//! Common validation utilities for MCP server configurations

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

use super::{ConfigField, ConfigFieldType};

/// Common validation utilities for server configurations
pub struct ConfigValidation;

impl ConfigValidation {
    /// Validate working directory path if provided in config
    pub fn validate_working_directory(config: &HashMap<String, String>) -> Result<()> {
        if let Some(working_dir) = config.get("working_directory") {
            let path = Path::new(working_dir);
            if !path.exists() {
                anyhow::bail!("Working directory does not exist: {}", working_dir);
            }
            if !path.is_dir() {
                anyhow::bail!("Working directory is not a directory: {}", working_dir);
            }
        }
        Ok(())
    }

    /// Validate timeout value if provided in config
    pub fn validate_timeout(config: &HashMap<String, String>) -> Result<()> {
        if let Some(timeout_str) = config.get("timeout") {
            timeout_str
                .parse::<u64>()
                .with_context(|| format!("Invalid timeout value: {timeout_str}"))?;
        }
        Ok(())
    }

    /// Validate port number if provided in config
    pub fn validate_port(config: &HashMap<String, String>, field_name: &str) -> Result<()> {
        if let Some(port_str) = config.get(field_name) {
            let port: u16 = port_str
                .parse()
                .with_context(|| format!("Invalid {field_name} value: {port_str}"))?;

            if port == 0 {
                anyhow::bail!("{field_name} cannot be 0");
            }
        }
        Ok(())
    }

    /// Validate URL format if provided in config
    pub fn validate_url(config: &HashMap<String, String>, field_name: &str) -> Result<()> {
        if let Some(url_str) = config.get(field_name) {
            url::Url::parse(url_str)
                .with_context(|| format!("Invalid {field_name} URL: {url_str}"))?;
        }
        Ok(())
    }

    /// Validate positive integer if provided in config
    pub fn validate_positive_integer(
        config: &HashMap<String, String>,
        field_name: &str,
    ) -> Result<()> {
        if let Some(value_str) = config.get(field_name) {
            let value: u64 = value_str
                .parse()
                .with_context(|| format!("Invalid {field_name} value: {value_str}"))?;

            if value == 0 {
                anyhow::bail!("{field_name} must be greater than 0");
            }
        }
        Ok(())
    }

    /// Validate boolean value if provided in config
    pub fn validate_boolean(config: &HashMap<String, String>, field_name: &str) -> Result<()> {
        if let Some(bool_str) = config.get(field_name) {
            bool_str
                .parse::<bool>()
                .with_context(|| format!("Invalid {field_name} value: {bool_str}"))?;
        }
        Ok(())
    }

    /// Validate that a required field is present
    pub fn validate_required_field(
        config: &HashMap<String, String>,
        field_name: &str,
    ) -> Result<()> {
        if !config.contains_key(field_name) || config.get(field_name).unwrap().is_empty() {
            anyhow::bail!("Missing required field: {field_name}");
        }
        Ok(())
    }

    /// Validate file path exists if provided in config
    pub fn validate_file_path(config: &HashMap<String, String>, field_name: &str) -> Result<()> {
        if let Some(file_path) = config.get(field_name) {
            let path = Path::new(file_path);
            if !path.exists() {
                anyhow::bail!("{field_name} file does not exist: {}", file_path);
            }
            if !path.is_file() {
                anyhow::bail!("{field_name} is not a file: {}", file_path);
            }
        }
        Ok(())
    }

    /// Validate a configuration field based on its type definition
    pub fn validate_field_type(field: &ConfigField, value: &str) -> Result<()> {
        match field.field_type {
            ConfigFieldType::Number => {
                value
                    .parse::<f64>()
                    .map_err(|_| anyhow::anyhow!("Field '{}' must be a number", field.name))?;
            }
            ConfigFieldType::Boolean => {
                value
                    .parse::<bool>()
                    .map_err(|_| anyhow::anyhow!("Field '{}' must be true or false", field.name))?;
            }
            ConfigFieldType::Path => {
                if value.is_empty() {
                    anyhow::bail!("Field '{}' (path) cannot be empty", field.name);
                }
            }
            ConfigFieldType::Url => {
                Self::validate_url_format(value)
                    .with_context(|| format!("Field '{}' must be a valid URL", field.name))?;
            }
            ConfigFieldType::String => {
                // String fields are always valid (basic type)
            }
        }
        Ok(())
    }

    /// Validate required configuration fields are present and valid
    pub fn validate_required_fields(
        config: &HashMap<String, String>,
        required_fields: &[ConfigField],
    ) -> Result<()> {
        for field in required_fields {
            match config.get(&field.name) {
                Some(value) => Self::validate_field_type(field, value)?,
                None => anyhow::bail!("Missing required configuration field: {}", field.name),
            }
        }
        Ok(())
    }

    /// Internal helper to validate URL format
    fn validate_url_format(value: &str) -> Result<()> {
        if !value.starts_with("http://") && !value.starts_with("https://") {
            anyhow::bail!("Must be a valid URL starting with http:// or https://");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_working_directory_valid() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = HashMap::new();
        config.insert(
            "working_directory".to_string(),
            temp_dir.path().to_string_lossy().to_string(),
        );

        assert!(ConfigValidation::validate_working_directory(&config).is_ok());
    }

    #[test]
    fn test_validate_working_directory_missing() {
        let mut config = HashMap::new();
        config.insert(
            "working_directory".to_string(),
            "/nonexistent/path".to_string(),
        );

        assert!(ConfigValidation::validate_working_directory(&config).is_err());
    }

    #[test]
    fn test_validate_working_directory_not_provided() {
        let config = HashMap::new();
        assert!(ConfigValidation::validate_working_directory(&config).is_ok());
    }

    #[test]
    fn test_validate_timeout_valid() {
        let mut config = HashMap::new();
        config.insert("timeout".to_string(), "30".to_string());

        assert!(ConfigValidation::validate_timeout(&config).is_ok());
    }

    #[test]
    fn test_validate_timeout_invalid() {
        let mut config = HashMap::new();
        config.insert("timeout".to_string(), "invalid".to_string());

        assert!(ConfigValidation::validate_timeout(&config).is_err());
    }

    #[test]
    fn test_validate_timeout_not_provided() {
        let config = HashMap::new();
        assert!(ConfigValidation::validate_timeout(&config).is_ok());
    }

    #[test]
    fn test_validate_port_valid() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "8080".to_string());

        assert!(ConfigValidation::validate_port(&config, "port").is_ok());
    }

    #[test]
    fn test_validate_port_invalid() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "invalid".to_string());

        assert!(ConfigValidation::validate_port(&config, "port").is_err());
    }

    #[test]
    fn test_validate_port_zero() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "0".to_string());

        assert!(ConfigValidation::validate_port(&config, "port").is_err());
    }

    #[test]
    fn test_validate_url_valid() {
        let mut config = HashMap::new();
        config.insert("api_url".to_string(), "https://example.com/api".to_string());

        assert!(ConfigValidation::validate_url(&config, "api_url").is_ok());
    }

    #[test]
    fn test_validate_url_invalid() {
        let mut config = HashMap::new();
        config.insert("api_url".to_string(), "not-a-url".to_string());

        assert!(ConfigValidation::validate_url(&config, "api_url").is_err());
    }

    #[test]
    fn test_validate_required_field_present() {
        let mut config = HashMap::new();
        config.insert("required_field".to_string(), "value".to_string());

        assert!(ConfigValidation::validate_required_field(&config, "required_field").is_ok());
    }

    #[test]
    fn test_validate_required_field_missing() {
        let config = HashMap::new();

        assert!(ConfigValidation::validate_required_field(&config, "required_field").is_err());
    }

    #[test]
    fn test_validate_required_field_empty() {
        let mut config = HashMap::new();
        config.insert("required_field".to_string(), "".to_string());

        assert!(ConfigValidation::validate_required_field(&config, "required_field").is_err());
    }

    #[test]
    fn test_validate_file_path_valid() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        fs::write(&file_path, "test content").unwrap();

        let mut config = HashMap::new();
        config.insert(
            "config_file".to_string(),
            file_path.to_string_lossy().to_string(),
        );

        assert!(ConfigValidation::validate_file_path(&config, "config_file").is_ok());
    }

    #[test]
    fn test_validate_file_path_missing() {
        let mut config = HashMap::new();
        config.insert(
            "config_file".to_string(),
            "/nonexistent/file.txt".to_string(),
        );

        assert!(ConfigValidation::validate_file_path(&config, "config_file").is_err());
    }

    #[test]
    fn test_validate_field_type_number_valid() {
        use super::super::{ConfigField, ConfigFieldType};

        let field = ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        assert!(ConfigValidation::validate_field_type(&field, "8080").is_ok());
        assert!(ConfigValidation::validate_field_type(&field, "3.14").is_ok());
    }

    #[test]
    fn test_validate_field_type_number_invalid() {
        use super::super::{ConfigField, ConfigFieldType};

        let field = ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        };

        assert!(ConfigValidation::validate_field_type(&field, "not-a-number").is_err());
    }

    #[test]
    fn test_validate_field_type_boolean_valid() {
        use super::super::{ConfigField, ConfigFieldType};

        let field = ConfigField {
            name: "enabled".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: None,
        };

        assert!(ConfigValidation::validate_field_type(&field, "true").is_ok());
        assert!(ConfigValidation::validate_field_type(&field, "false").is_ok());
    }

    #[test]
    fn test_validate_field_type_url_valid() {
        use super::super::{ConfigField, ConfigFieldType};

        let field = ConfigField {
            name: "api_url".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        };

        assert!(ConfigValidation::validate_field_type(&field, "https://example.com").is_ok());
        assert!(ConfigValidation::validate_field_type(&field, "http://localhost:8080").is_ok());
    }

    #[test]
    fn test_validate_field_type_url_invalid() {
        use super::super::{ConfigField, ConfigFieldType};

        let field = ConfigField {
            name: "api_url".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        };

        assert!(ConfigValidation::validate_field_type(&field, "not-a-url").is_err());
        assert!(ConfigValidation::validate_field_type(&field, "ftp://example.com").is_err());
    }
}
