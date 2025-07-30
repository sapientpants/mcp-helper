use crate::server::{ConfigField, ConfigFieldType};
use anyhow::{bail, Result};
use std::collections::HashMap;
use std::path::Path;
use url::Url;

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

pub type ValidationResult = Result<(), Vec<ValidationError>>;

pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate a configuration against server requirements
    pub fn validate_config(
        config: &HashMap<String, String>,
        required_fields: &[ConfigField],
        optional_fields: &[ConfigField],
    ) -> ValidationResult {
        let mut errors = Vec::new();

        // Check required fields
        for field in required_fields {
            match config.get(&field.name) {
                Some(value) => {
                    if let Err(e) =
                        Self::validate_field_value(&field.name, value, &field.field_type)
                    {
                        errors.push(ValidationError {
                            field: field.name.clone(),
                            message: e.to_string(),
                        });
                    }
                }
                None => {
                    errors.push(ValidationError {
                        field: field.name.clone(),
                        message: "Required field is missing".to_string(),
                    });
                }
            }
        }

        // Check optional fields if present
        for field in optional_fields {
            if let Some(value) = config.get(&field.name) {
                if let Err(e) = Self::validate_field_value(&field.name, value, &field.field_type) {
                    errors.push(ValidationError {
                        field: field.name.clone(),
                        message: e.to_string(),
                    });
                }
            }
        }

        // Validate extra fields (not in schema)
        for (key, value) in config {
            let is_known = required_fields.iter().any(|f| f.name == *key)
                || optional_fields.iter().any(|f| f.name == *key);

            if !is_known && value.is_empty() {
                errors.push(ValidationError {
                    field: key.clone(),
                    message: "Unknown field has empty value".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate environment variables in config
    pub fn validate_env_vars(env_vars: &HashMap<String, String>) -> ValidationResult {
        let mut errors = Vec::new();

        for (key, value) in env_vars {
            // Check for valid environment variable names
            if !Self::is_valid_env_var_name(key) {
                errors.push(ValidationError {
                    field: format!("env.{key}"),
                    message: format!("Invalid environment variable name: {key}"),
                });
            }

            // Check for empty values
            if value.trim().is_empty() {
                errors.push(ValidationError {
                    field: format!("env.{key}"),
                    message: "Environment variable value cannot be empty".to_string(),
                });
            }

            // Check for potentially dangerous values
            if value.contains("$(") || value.contains("${") || value.contains("`") {
                errors.push(ValidationError {
                    field: format!("env.{key}"),
                    message: "Environment variable contains potentially dangerous shell expansion characters".to_string(),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Test if a command is available in the system
    pub fn test_command_availability(command: &str, _args: &[String]) -> Result<()> {
        use std::process::Command;

        // Special handling for common commands
        let test_args = match command {
            "node" | "python" | "python3" | "docker" => vec!["--version"],
            "npx" => vec!["--version"],
            _ => {
                // For unknown commands, try --help first
                vec!["--help"]
            }
        };

        let output = Command::new(command)
            .args(&test_args)
            .output()
            .map_err(|e| anyhow::anyhow!("Command '{command}' not found: {e}"))?;

        if !output.status.success() && output.status.code() != Some(1) {
            // Exit code 1 is often used for --help, so we allow it
            bail!(
                "Command '{command}' failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    fn validate_field_value(
        _field_name: &str,
        value: &str,
        field_type: &ConfigFieldType,
    ) -> Result<()> {
        if value.trim().is_empty() {
            bail!("Value cannot be empty");
        }

        match field_type {
            ConfigFieldType::String => Ok(()), // Any non-empty string is valid
            ConfigFieldType::Number => value
                .parse::<f64>()
                .map(|_| ())
                .map_err(|_| anyhow::anyhow!("Invalid number format")),
            ConfigFieldType::Boolean => match value.to_lowercase().as_str() {
                "true" | "false" | "yes" | "no" | "1" | "0" => Ok(()),
                _ => bail!("Invalid boolean value. Use true/false, yes/no, or 1/0"),
            },
            ConfigFieldType::Path => {
                let _path = Path::new(value);
                if value.contains("..") {
                    bail!("Path cannot contain '..' for security reasons");
                }
                // We don't check if path exists, as it might be created later
                Ok(())
            }
            ConfigFieldType::Url => Url::parse(value)
                .map(|_| ())
                .map_err(|e| anyhow::anyhow!("Invalid URL: {e}")),
        }
    }

    fn is_valid_env_var_name(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Must start with letter or underscore
        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        // Rest must be alphanumeric or underscore
        name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_fields() {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "test123".to_string());

        let required = vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }];

        let result = ConfigValidator::validate_config(&config, &required, &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_missing_required_field() {
        let config = HashMap::new();

        let required = vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }];

        let result = ConfigValidator::validate_config(&config, &required, &[]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "api_key");
    }

    #[test]
    fn test_validate_field_types() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "not_a_number".to_string());
        config.insert("enabled".to_string(), "maybe".to_string());
        config.insert("url".to_string(), "not a url".to_string());

        let fields = vec![
            ConfigField {
                name: "port".to_string(),
                field_type: ConfigFieldType::Number,
                description: None,
                default: None,
            },
            ConfigField {
                name: "enabled".to_string(),
                field_type: ConfigFieldType::Boolean,
                description: None,
                default: None,
            },
            ConfigField {
                name: "url".to_string(),
                field_type: ConfigFieldType::Url,
                description: None,
                default: None,
            },
        ];

        let result = ConfigValidator::validate_config(&config, &fields, &[]);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 3);
    }

    #[test]
    fn test_validate_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("VALID_VAR".to_string(), "value".to_string());
        env_vars.insert("_ALSO_VALID".to_string(), "value".to_string());

        let result = ConfigValidator::validate_env_vars(&env_vars);
        assert!(result.is_ok());

        // Test invalid names
        let mut bad_env_vars = HashMap::new();
        bad_env_vars.insert("123INVALID".to_string(), "value".to_string());
        bad_env_vars.insert("INVALID-VAR".to_string(), "value".to_string());
        bad_env_vars.insert("".to_string(), "value".to_string());

        let result = ConfigValidator::validate_env_vars(&bad_env_vars);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2); // Empty key might not be in the iteration
    }

    #[test]
    fn test_is_valid_env_var_name() {
        assert!(ConfigValidator::is_valid_env_var_name("VALID"));
        assert!(ConfigValidator::is_valid_env_var_name("_VALID"));
        assert!(ConfigValidator::is_valid_env_var_name("VALID_123"));
        assert!(!ConfigValidator::is_valid_env_var_name("123INVALID"));
        assert!(!ConfigValidator::is_valid_env_var_name("INVALID-VAR"));
        assert!(!ConfigValidator::is_valid_env_var_name(""));
    }
}
