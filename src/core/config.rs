//! Pure configuration transformation and validation logic
//!
//! This module contains pure functions for transforming and validating configuration data
//! without performing I/O operations.

use crate::client::ServerConfig;
use crate::server::{ConfigField, ConfigFieldType};
use std::collections::HashMap;

/// Validates that all required configuration fields are present
pub fn validate_required_fields(
    config: &HashMap<String, String>,
    required_fields: &[ConfigField],
) -> Result<(), String> {
    for field in required_fields {
        if !config.contains_key(&field.name) {
            return Err(format!("Missing required field: {}", field.name));
        }
    }
    Ok(())
}

/// Validates configuration field types and formats
pub fn validate_field_types(
    config: &HashMap<String, String>,
    fields: &[ConfigField],
) -> Result<(), String> {
    for field in fields {
        if let Some(value) = config.get(&field.name) {
            match field.field_type {
                ConfigFieldType::Number => {
                    if value.parse::<f64>().is_err() {
                        return Err(format!("Field '{}' must be a valid number", field.name));
                    }
                }
                ConfigFieldType::Boolean => {
                    if !matches!(
                        value.to_lowercase().as_str(),
                        "true" | "false" | "1" | "0" | "yes" | "no"
                    ) {
                        return Err(format!("Field '{}' must be a boolean value", field.name));
                    }
                }
                ConfigFieldType::Url => {
                    if !value.starts_with("http://") && !value.starts_with("https://") {
                        return Err(format!("Field '{}' must be a valid URL", field.name));
                    }
                }
                ConfigFieldType::Path => {
                    if value.is_empty() {
                        return Err(format!("Field '{}' must be a non-empty path", field.name));
                    }
                }
                ConfigFieldType::String => {
                    // String fields are always valid if present
                }
            }
        }
    }
    Ok(())
}

/// Merges user configuration with default values
pub fn merge_with_defaults(
    user_config: HashMap<String, String>,
    fields: &[ConfigField],
) -> HashMap<String, String> {
    let mut merged = user_config;

    for field in fields {
        if !merged.contains_key(&field.name) {
            if let Some(default_value) = &field.default {
                merged.insert(field.name.clone(), default_value.clone());
            }
        }
    }

    merged
}

/// Transforms configuration into ServerConfig format
pub fn transform_to_server_config(
    command: String,
    args: Vec<String>,
    config: HashMap<String, String>,
) -> ServerConfig {
    ServerConfig {
        command,
        args,
        env: config,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_required_fields_success() {
        let mut config = HashMap::new();
        config.insert("api_key".to_string(), "test-key".to_string());

        let fields = vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }];

        assert!(validate_required_fields(&config, &fields).is_ok());
    }

    #[test]
    fn test_validate_required_fields_missing() {
        let config = HashMap::new();

        let fields = vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }];

        let result = validate_required_fields(&config, &fields);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Missing required field: api_key"));
    }

    #[test]
    fn test_validate_field_types_number() {
        let mut config = HashMap::new();
        config.insert("port".to_string(), "8080".to_string());

        let fields = vec![ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        }];

        assert!(validate_field_types(&config, &fields).is_ok());

        config.insert("port".to_string(), "not-a-number".to_string());
        let result = validate_field_types(&config, &fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a valid number"));
    }

    #[test]
    fn test_validate_field_types_boolean() {
        let mut config = HashMap::new();
        config.insert("debug".to_string(), "true".to_string());

        let fields = vec![ConfigField {
            name: "debug".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: None,
        }];

        assert!(validate_field_types(&config, &fields).is_ok());

        config.insert("debug".to_string(), "invalid".to_string());
        let result = validate_field_types(&config, &fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a boolean value"));
    }

    #[test]
    fn test_validate_field_types_url() {
        let mut config = HashMap::new();
        config.insert(
            "endpoint".to_string(),
            "https://api.example.com".to_string(),
        );

        let fields = vec![ConfigField {
            name: "endpoint".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        }];

        assert!(validate_field_types(&config, &fields).is_ok());

        config.insert("endpoint".to_string(), "not-a-url".to_string());
        let result = validate_field_types(&config, &fields);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a valid URL"));
    }

    #[test]
    fn test_merge_with_defaults() {
        let mut user_config = HashMap::new();
        user_config.insert("api_key".to_string(), "user-key".to_string());

        let fields = vec![
            ConfigField {
                name: "api_key".to_string(),
                field_type: ConfigFieldType::String,
                description: None,
                default: Some("default-key".to_string()),
            },
            ConfigField {
                name: "timeout".to_string(),
                field_type: ConfigFieldType::Number,
                description: None,
                default: Some("30".to_string()),
            },
        ];

        let merged = merge_with_defaults(user_config, &fields);

        assert_eq!(merged.get("api_key"), Some(&"user-key".to_string()));
        assert_eq!(merged.get("timeout"), Some(&"30".to_string()));
    }

    #[test]
    fn test_transform_to_server_config() {
        let mut env = HashMap::new();
        env.insert("API_KEY".to_string(), "test-key".to_string());

        let config = transform_to_server_config(
            "npx".to_string(),
            vec!["test-server".to_string()],
            env.clone(),
        );

        assert_eq!(config.command, "npx");
        assert_eq!(config.args, vec!["test-server"]);
        assert_eq!(config.env, env);
    }
}
