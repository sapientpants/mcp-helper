//! Property-based tests for configuration logic
//!
//! This module contains property tests that verify configuration functions work correctly
//! for all possible inputs within defined constraints.

#[cfg(test)]
mod tests {
    use crate::core::config::*;
    use crate::server::{ConfigField, ConfigFieldType};
    use proptest::prelude::*;
    use std::collections::HashMap;

    // Strategy for generating config field names
    fn field_name() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_]{0,30}"
    }

    // Strategy for generating config field values
    fn field_value() -> impl Strategy<Value = String> {
        prop_oneof![
            // Strings
            "[a-zA-Z0-9 ._-]{0,100}",
            // Numbers
            "[0-9]{1,10}",
            // Booleans
            "true|false|yes|no|1|0",
            // URLs
            "https://[a-zA-Z0-9.-]+\\.[a-z]{2,}/[a-zA-Z0-9/]*",
            // Paths
            "/[a-zA-Z0-9/._-]+",
        ]
    }

    // Strategy for generating ConfigField
    prop_compose! {
        fn config_field()(
            name in field_name(),
            field_type in prop_oneof![
                Just(ConfigFieldType::String),
                Just(ConfigFieldType::Number),
                Just(ConfigFieldType::Boolean),
                Just(ConfigFieldType::Url),
                Just(ConfigFieldType::Path),
            ],
            has_default in prop::bool::ANY,
            default_value in field_value(),
        ) -> ConfigField {
            ConfigField {
                name,
                field_type,
                description: Some("Test field".to_string()),
                default: if has_default { Some(default_value) } else { None },
            }
        }
    }

    proptest! {
        #[test]
        fn test_validate_required_fields_always_accepts_when_all_present(
            fields in prop::collection::vec(config_field(), 1..10),
        ) {
            let mut config = HashMap::new();

            // Add all required fields to config
            for field in &fields {
                config.insert(field.name.clone(), "test_value".to_string());
            }

            let result = validate_required_fields(&config, &fields);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn test_validate_required_fields_rejects_when_missing(
            fields in prop::collection::vec(config_field(), 2..10),
            missing_index in 0usize..10,
        ) {
            prop_assume!(!fields.is_empty());
            let missing_index = missing_index % fields.len();

            let mut config = HashMap::new();

            // Add all fields except one
            for (i, field) in fields.iter().enumerate() {
                if i != missing_index {
                    config.insert(field.name.clone(), "test_value".to_string());
                }
            }

            let result = validate_required_fields(&config, &fields);
            prop_assert!(result.is_err());
            prop_assert!(result.unwrap_err().contains(&fields[missing_index].name));
        }

        #[test]
        fn test_validate_field_types_number(
            field_name in field_name(),
            valid_number in "[0-9]{1,10}",
            invalid_number in "[a-zA-Z]+",
        ) {
            let field = ConfigField {
                name: field_name.clone(),
                field_type: ConfigFieldType::Number,
                description: None,
                default: None,
            };

            let mut config = HashMap::new();

            // Test valid number
            config.insert(field_name.clone(), valid_number);
            let result = validate_field_types(&config, &[field.clone()]);
            prop_assert!(result.is_ok());

            // Test invalid number
            config.insert(field_name.clone(), invalid_number);
            let result = validate_field_types(&config, &[field]);
            prop_assert!(result.is_err());
        }

        #[test]
        fn test_validate_field_types_boolean(
            field_name in field_name(),
            valid_bool in "true|false|yes|no|1|0|TRUE|FALSE|Yes|No",
            invalid_bool in "[2-9][0-9]*|maybe|perhaps",
        ) {
            let field = ConfigField {
                name: field_name.clone(),
                field_type: ConfigFieldType::Boolean,
                description: None,
                default: None,
            };

            let mut config = HashMap::new();

            // Test valid boolean
            config.insert(field_name.clone(), valid_bool);
            let result = validate_field_types(&config, &[field.clone()]);
            prop_assert!(result.is_ok());

            // Test invalid boolean
            config.insert(field_name.clone(), invalid_bool);
            let result = validate_field_types(&config, &[field]);
            prop_assert!(result.is_err());
        }

        #[test]
        fn test_validate_field_types_url(
            field_name in field_name(),
            valid_protocol in "https?",
            host in "[a-zA-Z0-9.-]+\\.[a-z]{2,}",
            path in "/[a-zA-Z0-9/._-]*",
        ) {
            let field = ConfigField {
                name: field_name.clone(),
                field_type: ConfigFieldType::Url,
                description: None,
                default: None,
            };

            let mut config = HashMap::new();

            // Test valid URL
            let valid_url = format!("{valid_protocol}://{host}{path}");
            config.insert(field_name.clone(), valid_url);
            let result = validate_field_types(&config, &[field.clone()]);
            prop_assert!(result.is_ok());

            // Test invalid URL (no protocol)
            config.insert(field_name.clone(), format!("{host}{path}"));
            let result = validate_field_types(&config, &[field]);
            prop_assert!(result.is_err());
        }

        #[test]
        fn test_merge_with_defaults_preserves_user_values(
            fields in prop::collection::vec(config_field(), 1..10),
            user_values in prop::collection::vec((field_name(), field_value()), 1..10),
        ) {
            let mut user_config = HashMap::new();
            for (name, value) in user_values {
                user_config.insert(name, value);
            }

            let original_user_config = user_config.clone();
            let merged = merge_with_defaults(user_config, &fields);

            // All original user values should be preserved
            for (key, value) in &original_user_config {
                prop_assert_eq!(merged.get(key), Some(value));
            }
        }

        #[test]
        fn test_merge_with_defaults_adds_missing_defaults(
            fields in prop::collection::vec(config_field(), 1..10),
        ) {
            let empty_config = HashMap::new();
            let merged = merge_with_defaults(empty_config, &fields);

            // All fields with defaults should be in the merged config
            for field in &fields {
                if let Some(default) = &field.default {
                    prop_assert_eq!(merged.get(&field.name), Some(default));
                }
            }
        }

        #[test]
        fn test_transform_to_server_config_preserves_all_data(
            command in "[a-zA-Z0-9_-]+",
            args in prop::collection::vec("[a-zA-Z0-9_-]+", 0..5),
            env_entries in prop::collection::vec((field_name(), field_value()), 0..10),
        ) {
            let mut env = HashMap::new();
            for (key, value) in env_entries {
                env.insert(key, value);
            }

            let config = transform_to_server_config(
                command.clone(),
                args.clone(),
                env.clone(),
            );

            prop_assert_eq!(config.command, command);
            prop_assert_eq!(config.args, args);
            prop_assert_eq!(config.env, env);
        }

        #[test]
        fn test_validate_all_field_types(
            fields in prop::collection::vec(config_field(), 1..20),
        ) {
            let mut config = HashMap::new();
            let mut seen_names = std::collections::HashSet::new();
            let mut unique_fields = Vec::new();

            // Filter out duplicate field names
            for field in fields {
                if seen_names.insert(field.name.clone()) {
                    unique_fields.push(field);
                }
            }

            // If no unique fields, nothing to test
            if unique_fields.is_empty() {
                return Ok(());
            }

            // Generate valid values for each field type
            for field in &unique_fields {
                let value = match field.field_type {
                    ConfigFieldType::String => "test_string".to_string(),
                    ConfigFieldType::Number => "12345".to_string(),
                    ConfigFieldType::Boolean => "true".to_string(),
                    ConfigFieldType::Url => "https://example.com".to_string(),
                    ConfigFieldType::Path => "/valid/path".to_string(),
                };
                config.insert(field.name.clone(), value);
            }

            let result = validate_field_types(&config, &unique_fields);
            if result.is_err() {
                println!("Validation failed for fields: {unique_fields:?}");
                println!("Config: {config:?}");
                println!("Error: {result:?}");
            }
            prop_assert!(result.is_ok());
        }
    }
}
