use mcp_helper::server::{ConfigField, ConfigFieldType, McpServer, NpmServer, ServerType};
use std::collections::HashMap;

#[test]
fn test_npm_server_from_simple_package() {
    let server = NpmServer::new("express").unwrap();
    assert_eq!(server.metadata().name, "express");
    match &server.metadata().server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "express");
            assert!(version.is_none());
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_npm_server_from_scoped_package() {
    let server = NpmServer::new("@modelcontextprotocol/server-filesystem").unwrap();
    assert_eq!(
        server.metadata().name,
        "@modelcontextprotocol/server-filesystem"
    );
    match &server.metadata().server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert!(version.is_none());
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_npm_server_with_version() {
    let server = NpmServer::new("lodash@4.17.21").unwrap();
    assert_eq!(server.metadata().name, "lodash");
    match &server.metadata().server_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "lodash");
            assert_eq!(version.as_ref().unwrap(), "4.17.21");
        }
        _ => panic!("Expected NPM server type"),
    }
}

#[test]
fn test_npm_server_invalid_type() {
    assert!(NpmServer::new("https://github.com/test/repo").is_err());
    assert!(NpmServer::new("docker:nginx").is_err());
    assert!(NpmServer::new("script.py").is_err());
}

#[test]
fn test_npm_server_from_package() {
    let server = NpmServer::from_package("test-package".to_string(), None);
    assert_eq!(server.metadata().name, "test-package");
}

#[test]
fn test_npm_server_with_metadata() {
    let server = NpmServer::from_package("test".to_string(), None).with_metadata(
        "Custom Name".to_string(),
        Some("Custom description".to_string()),
    );

    assert_eq!(server.metadata().name, "Custom Name");
    assert_eq!(
        server.metadata().description,
        Some("Custom description".to_string())
    );
}

#[test]
fn test_npm_server_with_config() {
    let required = vec![ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API Key".to_string()),
        default: None,
    }];

    let optional = vec![ConfigField {
        name: "debug".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: Some("Enable debug mode".to_string()),
        default: Some("false".to_string()),
    }];

    let server = NpmServer::from_package("test".to_string(), None).with_config(required, optional);

    assert_eq!(server.metadata().required_config.len(), 1);
    assert_eq!(server.metadata().optional_config.len(), 1);
}

#[test]
fn test_generate_command_basic() {
    let server = NpmServer::from_package("test-server".to_string(), None);
    let (cmd, args) = server.generate_command().unwrap();

    #[cfg(target_os = "windows")]
    assert_eq!(cmd, "npx.cmd");

    #[cfg(not(target_os = "windows"))]
    assert_eq!(cmd, "npx");

    assert_eq!(args, vec!["--yes", "test-server", "--stdio"]);
}

#[test]
fn test_generate_command_with_version() {
    let server = NpmServer::from_package("test-server".to_string(), Some("1.2.3".to_string()));
    let (_, args) = server.generate_command().unwrap();

    assert_eq!(args[1], "test-server@1.2.3");
}

#[test]
fn test_generate_command_scoped_package() {
    let server = NpmServer::from_package("@org/package".to_string(), None);
    let (_, args) = server.generate_command().unwrap();

    assert_eq!(args[1], "@org/package");
}

#[test]
fn test_validate_config_empty() {
    let server = NpmServer::from_package("test".to_string(), None);
    let config = HashMap::new();

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_missing_required() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "required_field".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }],
        vec![],
    );

    let config = HashMap::new();
    let result = server.validate_config(&config);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("required_field"));
}

#[test]
fn test_validate_config_valid_required() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("api_key".to_string(), "secret123".to_string());

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_invalid_number() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("port".to_string(), "not-a-number".to_string());

    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("must be a number"));
}

#[test]
fn test_validate_config_valid_number() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "port".to_string(),
            field_type: ConfigFieldType::Number,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("port".to_string(), "8080".to_string());

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_invalid_boolean() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "enabled".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("enabled".to_string(), "yes".to_string());

    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be true or false"));
}

#[test]
fn test_validate_config_valid_boolean() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "enabled".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("enabled".to_string(), "true".to_string());
    assert!(server.validate_config(&config).is_ok());

    config.insert("enabled".to_string(), "false".to_string());
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_empty_path() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "config_path".to_string(),
            field_type: ConfigFieldType::Path,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("config_path".to_string(), "".to_string());

    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[test]
fn test_validate_config_valid_path() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "config_path".to_string(),
            field_type: ConfigFieldType::Path,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("config_path".to_string(), "/path/to/config".to_string());

    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_validate_config_invalid_url() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "endpoint".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert("endpoint".to_string(), "not-a-url".to_string());

    let result = server.validate_config(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must be a valid URL"));
}

#[test]
fn test_validate_config_valid_url() {
    let server = NpmServer::from_package("test".to_string(), None).with_config(
        vec![ConfigField {
            name: "endpoint".to_string(),
            field_type: ConfigFieldType::Url,
            description: None,
            default: None,
        }],
        vec![],
    );

    let mut config = HashMap::new();
    config.insert(
        "endpoint".to_string(),
        "https://api.example.com".to_string(),
    );
    assert!(server.validate_config(&config).is_ok());

    config.insert("endpoint".to_string(), "http://localhost:8080".to_string());
    assert!(server.validate_config(&config).is_ok());
}

#[test]
fn test_npm_server_dependency() {
    let server = NpmServer::from_package("test".to_string(), None);
    let dep = server.get_dependency();

    match dep {
        mcp_helper::deps::Dependency::NodeJs { min_version } => {
            assert_eq!(min_version, Some("16.0.0".to_string()));
        }
        _ => panic!("Expected NodeJs dependency"),
    }
}
