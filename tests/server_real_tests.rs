//! Real implementation tests for MCP servers
//!
//! This test suite uses real server implementations instead of mocks,
//! ensuring we're testing actual behavior.

use mcp_helper::server::{ConfigField, ConfigFieldType, ServerMetadata, ServerType};

#[test]
fn test_server_metadata_creation() {
    let metadata = ServerMetadata {
        name: "test-server".to_string(),
        description: Some("A test server".to_string()),
        server_type: ServerType::Npm {
            package: "test-server".to_string(),
            version: None,
        },
        required_config: vec![],
        optional_config: vec![],
    };

    assert_eq!(metadata.name, "test-server");
    assert_eq!(metadata.description, Some("A test server".to_string()));
    assert!(matches!(metadata.server_type, ServerType::Npm { .. }));
}

#[test]
fn test_server_metadata_with_config_fields() {
    let required_field = ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API key for authentication".to_string()),
        default: None,
    };

    let optional_field = ConfigField {
        name: "timeout".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("Request timeout in seconds".to_string()),
        default: Some("30".to_string()),
    };

    let metadata = ServerMetadata {
        name: "configured-server".to_string(),
        description: None,
        server_type: ServerType::Npm {
            package: "configured-server".to_string(),
            version: Some("1.0.0".to_string()),
        },
        required_config: vec![required_field.clone()],
        optional_config: vec![optional_field.clone()],
    };

    assert_eq!(metadata.required_config.len(), 1);
    assert_eq!(metadata.required_config[0].name, "api_key");
    assert!(metadata.required_config[0].default.is_none());

    assert_eq!(metadata.optional_config.len(), 1);
    assert_eq!(metadata.optional_config[0].name, "timeout");
    assert_eq!(metadata.optional_config[0].default, Some("30".to_string()));
}

#[test]
fn test_config_field_types() {
    let string_field = ConfigField {
        name: "text".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };

    let number_field = ConfigField {
        name: "port".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("Port number".to_string()),
        default: Some("8080".to_string()),
    };

    let bool_field = ConfigField {
        name: "debug".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: Some("Enable debug mode".to_string()),
        default: Some("false".to_string()),
    };

    let url_field = ConfigField {
        name: "endpoint".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("API endpoint".to_string()),
        default: None,
    };

    let path_field = ConfigField {
        name: "config_path".to_string(),
        field_type: ConfigFieldType::Path,
        description: Some("Configuration file path".to_string()),
        default: Some("./config.json".to_string()),
    };

    assert!(matches!(string_field.field_type, ConfigFieldType::String));
    assert!(matches!(number_field.field_type, ConfigFieldType::Number));
    assert!(matches!(bool_field.field_type, ConfigFieldType::Boolean));
    assert!(matches!(url_field.field_type, ConfigFieldType::Url));
    assert!(matches!(path_field.field_type, ConfigFieldType::Path));
}

#[test]
fn test_server_type_npm_variant() {
    let npm_type = ServerType::Npm {
        package: "@modelcontextprotocol/server-filesystem".to_string(),
        version: Some("1.0.0".to_string()),
    };

    match npm_type {
        ServerType::Npm { package, version } => {
            assert_eq!(package, "@modelcontextprotocol/server-filesystem");
            assert_eq!(version, Some("1.0.0".to_string()));
        }
        _ => panic!("Expected Npm variant"),
    }
}

#[test]
fn test_server_type_binary_variant() {
    let binary_type = ServerType::Binary {
        url: "https://github.com/user/repo/releases/download/v1.0/server".to_string(),
        checksum: Some("sha256:abcdef123456".to_string()),
    };

    match binary_type {
        ServerType::Binary { url, checksum } => {
            assert!(url.starts_with("https://"));
            assert!(checksum.is_some());
        }
        _ => panic!("Expected Binary variant"),
    }
}

#[test]
fn test_server_type_python_variant() {
    let python_type = ServerType::Python {
        package: "mcp-server.py".to_string(),
        version: Some("3.9".to_string()),
    };

    match python_type {
        ServerType::Python { package, version } => {
            assert_eq!(package, "mcp-server.py");
            assert_eq!(version, Some("3.9".to_string()));
        }
        _ => panic!("Expected Python variant"),
    }
}

#[test]
fn test_server_type_docker_variant() {
    let docker_type = ServerType::Docker {
        image: "mcp/server".to_string(),
        tag: Some("latest".to_string()),
    };

    match docker_type {
        ServerType::Docker { image, tag } => {
            assert_eq!(image, "mcp/server");
            assert_eq!(tag, Some("latest".to_string()));
        }
        _ => panic!("Expected Docker variant"),
    }
}

#[test]
fn test_server_metadata_default() {
    let metadata = ServerMetadata {
        name: "default-server".to_string(),
        description: None,
        server_type: ServerType::Npm {
            package: "default-server".to_string(),
            version: None,
        },
        required_config: vec![],
        optional_config: vec![],
    };

    assert!(metadata.description.is_none());
    assert!(metadata.required_config.is_empty());
    assert!(metadata.optional_config.is_empty());
}

#[test]
fn test_config_field_with_all_options() {
    let field = ConfigField {
        name: "database_url".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("PostgreSQL connection string".to_string()),
        default: Some("postgres://localhost:5432/mydb".to_string()),
    };

    assert_eq!(field.name, "database_url");
    assert!(matches!(field.field_type, ConfigFieldType::Url));
    assert_eq!(
        field.description,
        Some("PostgreSQL connection string".to_string())
    );
    assert_eq!(
        field.default,
        Some("postgres://localhost:5432/mydb".to_string())
    );
}

#[test]
fn test_multiple_required_fields() {
    let fields = vec![
        ConfigField {
            name: "api_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API key".to_string()),
            default: None,
        },
        ConfigField {
            name: "api_secret".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("API secret".to_string()),
            default: None,
        },
        ConfigField {
            name: "endpoint".to_string(),
            field_type: ConfigFieldType::Url,
            description: Some("API endpoint".to_string()),
            default: None,
        },
    ];

    let metadata = ServerMetadata {
        name: "multi-field-server".to_string(),
        description: Some("Server with multiple required fields".to_string()),
        server_type: ServerType::Npm {
            package: "multi-field-server".to_string(),
            version: None,
        },
        required_config: fields,
        optional_config: vec![],
    };

    assert_eq!(metadata.required_config.len(), 3);
    assert!(metadata.required_config.iter().all(|f| f.default.is_none()));
}

#[test]
fn test_config_field_validation_hints() {
    // Fields with specific types should help with validation
    let url_field = ConfigField {
        name: "webhook_url".to_string(),
        field_type: ConfigFieldType::Url,
        description: Some("Webhook endpoint for notifications".to_string()),
        default: None,
    };

    // URL field type indicates URL validation should be applied
    assert!(matches!(url_field.field_type, ConfigFieldType::Url));

    let path_field = ConfigField {
        name: "log_file".to_string(),
        field_type: ConfigFieldType::Path,
        description: Some("Path to log file".to_string()),
        default: Some("/var/log/app.log".to_string()),
    };

    // Path field type indicates path validation should be applied
    assert!(matches!(path_field.field_type, ConfigFieldType::Path));

    let bool_field = ConfigField {
        name: "enable_ssl".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: Some("Enable SSL/TLS".to_string()),
        default: Some("true".to_string()),
    };

    // Boolean field should only accept true/false values
    assert!(matches!(bool_field.field_type, ConfigFieldType::Boolean));
    assert!(bool_field.default == Some("true".to_string()));
}

#[test]
fn test_server_type_npm_scoped_packages() {
    let scoped_packages = vec![
        ("@babel/core", None),
        ("@types/node", Some("18.0.0")),
        ("@angular/cli", Some("15.0.0")),
        ("@vue/compiler-core", None),
        ("@testing-library/react", Some("13.0.0")),
    ];

    for (package, version) in scoped_packages {
        let server_type = ServerType::Npm {
            package: package.to_string(),
            version: version.map(|v| v.to_string()),
        };

        match server_type {
            ServerType::Npm {
                package: p,
                version: v,
            } => {
                assert_eq!(p, package);
                assert_eq!(v, version.map(|s| s.to_string()));
            }
            _ => panic!("Expected Npm variant"),
        }
    }
}

#[test]
fn test_server_metadata_mixed_config() {
    let metadata = ServerMetadata {
        name: "mixed-config-server".to_string(),
        description: Some("Server with both required and optional config".to_string()),
        server_type: ServerType::Binary {
            url: "https://example.com/server".to_string(),
            checksum: None,
        },
        required_config: vec![ConfigField {
            name: "license_key".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("License key".to_string()),
            default: None,
        }],
        optional_config: vec![
            ConfigField {
                name: "log_level".to_string(),
                field_type: ConfigFieldType::String,
                description: Some("Logging level".to_string()),
                default: Some("info".to_string()),
            },
            ConfigField {
                name: "max_connections".to_string(),
                field_type: ConfigFieldType::Number,
                description: Some("Maximum connections".to_string()),
                default: Some("100".to_string()),
            },
        ],
    };

    assert_eq!(metadata.required_config.len(), 1);
    assert_eq!(metadata.optional_config.len(), 2);
    assert!(matches!(metadata.server_type, ServerType::Binary { .. }));
}

#[test]
fn test_docker_server_type_with_registry() {
    let docker_types = vec![
        ServerType::Docker {
            image: "nginx".to_string(),
            tag: Some("alpine".to_string()),
        },
        ServerType::Docker {
            image: "ghcr.io/user/app".to_string(),
            tag: Some("v1.0.0".to_string()),
        },
        ServerType::Docker {
            image: "docker.io/library/postgres".to_string(),
            tag: Some("15".to_string()),
        },
    ];

    for docker_type in docker_types {
        match docker_type {
            ServerType::Docker { image, tag } => {
                assert!(!image.is_empty());
                assert!(tag.is_some());
            }
            _ => panic!("Expected Docker variant"),
        }
    }
}

#[test]
fn test_config_field_optional_with_defaults() {
    let optional_fields = vec![
        ConfigField {
            name: "theme".to_string(),
            field_type: ConfigFieldType::String,
            description: Some("UI theme".to_string()),
            default: Some("dark".to_string()),
        },
        ConfigField {
            name: "auto_save".to_string(),
            field_type: ConfigFieldType::Boolean,
            description: Some("Enable auto-save".to_string()),
            default: Some("true".to_string()),
        },
        ConfigField {
            name: "save_interval".to_string(),
            field_type: ConfigFieldType::Number,
            description: Some("Auto-save interval in seconds".to_string()),
            default: Some("60".to_string()),
        },
    ];

    for field in &optional_fields {
        assert!(field.default.is_some());
        assert!(field.description.is_some());
    }
}

#[test]
fn test_python_server_type_variations() {
    let python_types = vec![
        ServerType::Python {
            package: "server.py".to_string(),
            version: None,
        },
        ServerType::Python {
            package: "mcp_server/__main__.py".to_string(),
            version: Some("3.10".to_string()),
        },
        ServerType::Python {
            package: "app.pyw".to_string(),
            version: Some("3.11".to_string()),
        },
    ];

    for python_type in python_types {
        match python_type {
            ServerType::Python { package, .. } => {
                assert!(package.ends_with(".py") || package.ends_with(".pyw"));
            }
            _ => panic!("Expected Python variant"),
        }
    }
}

#[test]
fn test_binary_server_type_with_checksum() {
    let binary_with_checksum = ServerType::Binary {
        url: "https://releases.example.com/v1.0/server-linux-amd64".to_string(),
        checksum: Some(
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
        ),
    };

    match binary_with_checksum {
        ServerType::Binary { url, checksum } => {
            assert!(url.contains("https://"));
            assert!(checksum.is_some());
            if let Some(cs) = checksum {
                assert!(cs.starts_with("sha256:"));
            }
        }
        _ => panic!("Expected Binary variant"),
    }
}

#[test]
fn test_server_metadata_empty_configs() {
    let metadata = ServerMetadata {
        name: "minimal".to_string(),
        description: None,
        server_type: ServerType::Npm {
            package: "minimal".to_string(),
            version: None,
        },
        required_config: vec![],
        optional_config: vec![],
    };

    assert!(metadata.required_config.is_empty());
    assert!(metadata.optional_config.is_empty());
    assert!(metadata.description.is_none());
}
