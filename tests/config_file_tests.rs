use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_read_empty_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    // Write empty JSON object
    fs::write(&config_path, "{}").unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed, json!({}));
}

#[test]
fn test_read_config_with_mcp_servers() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let config_content = json!({
        "mcpServers": {
            "filesystem": {
                "command": "npx",
                "args": ["@modelcontextprotocol/server-filesystem"],
                "env": {
                    "PATH": "/home/user"
                }
            }
        }
    });

    fs::write(&config_path, config_content.to_string()).unwrap();

    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert!(parsed.get("mcpServers").is_some());
    assert!(parsed["mcpServers"].get("filesystem").is_some());
}

#[test]
fn test_add_server_to_existing_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    // Create initial config with one server
    let mut initial_config = json!({
        "mcpServers": {
            "existing-server": {
                "command": "node",
                "args": ["server.js"]
            }
        }
    });

    fs::write(&config_path, initial_config.to_string()).unwrap();

    // Add new server
    if let Some(servers) = initial_config["mcpServers"].as_object_mut() {
        servers.insert(
            "new-server".to_string(),
            json!({
                "command": "python",
                "args": ["-m", "server"],
                "env": {
                    "API_KEY": "test123"
                }
            }),
        );
    }

    // Write back
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&initial_config).unwrap(),
    )
    .unwrap();

    // Read and verify
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["mcpServers"]["existing-server"].is_object());
    assert!(parsed["mcpServers"]["new-server"].is_object());
    assert_eq!(parsed["mcpServers"]["new-server"]["command"], "python");
}

#[test]
fn test_preserve_json_formatting() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    // Write config with specific formatting
    let formatted_json = r#"{
  "theme": "dark",
  "mcpServers": {
    "test": {
      "command": "test",
      "args": []
    }
  },
  "fontSize": 14
}"#;

    fs::write(&config_path, formatted_json).unwrap();

    // Read and parse
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    // Serialize with pretty formatting
    let pretty = serde_json::to_string_pretty(&parsed).unwrap();

    // Should maintain structure (though exact formatting may differ)
    assert!(pretty.contains("\"theme\": \"dark\""));
    assert!(pretty.contains("\"fontSize\": 14"));
}

#[test]
fn test_update_existing_server() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let mut config = json!({
        "mcpServers": {
            "test-server": {
                "command": "old-command",
                "args": ["old-arg"]
            }
        }
    });

    fs::write(&config_path, config.to_string()).unwrap();

    // Update the server
    config["mcpServers"]["test-server"]["command"] = json!("new-command");
    config["mcpServers"]["test-server"]["args"] = json!(["new-arg1", "new-arg2"]);

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Verify update
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(
        parsed["mcpServers"]["test-server"]["command"],
        "new-command"
    );
    assert_eq!(parsed["mcpServers"]["test-server"]["args"][0], "new-arg1");
    assert_eq!(parsed["mcpServers"]["test-server"]["args"][1], "new-arg2");
}

#[test]
fn test_remove_server_from_config() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let mut config = json!({
        "mcpServers": {
            "server1": {
                "command": "cmd1",
                "args": []
            },
            "server2": {
                "command": "cmd2",
                "args": []
            }
        }
    });

    fs::write(&config_path, config.to_string()).unwrap();

    // Remove server2
    if let Some(servers) = config["mcpServers"].as_object_mut() {
        servers.remove("server2");
    }

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Verify removal
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert!(parsed["mcpServers"]["server1"].is_object());
    assert!(parsed["mcpServers"]["server2"].is_null());
}

#[test]
fn test_handle_malformed_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    // Write malformed JSON
    fs::write(&config_path, "{ invalid json }").unwrap();

    // Attempt to parse
    let content = fs::read_to_string(&config_path).unwrap();
    let result: Result<Value, _> = serde_json::from_str(&content);

    assert!(result.is_err());
}

#[test]
fn test_create_config_if_not_exists() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    // Ensure file doesn't exist
    assert!(!config_path.exists());

    // Create new config
    let new_config = json!({
        "mcpServers": {}
    });

    fs::write(
        &config_path,
        serde_json::to_string_pretty(&new_config).unwrap(),
    )
    .unwrap();

    // Verify creation
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();
    assert_eq!(parsed, new_config);
}

#[test]
fn test_environment_variable_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let config = json!({
        "mcpServers": {
            "env-test": {
                "command": "test",
                "args": [],
                "env": {
                    "STRING_VAR": "hello world",
                    "NUMBER_VAR": "12345",
                    "BOOL_VAR": "true",
                    "PATH_VAR": "/usr/local/bin:/usr/bin",
                    "EMPTY_VAR": ""
                }
            }
        }
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Read and verify all env vars are strings
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();
    let env = &parsed["mcpServers"]["env-test"]["env"];

    assert!(env["STRING_VAR"].is_string());
    assert!(env["NUMBER_VAR"].is_string());
    assert!(env["BOOL_VAR"].is_string());
    assert!(env["PATH_VAR"].is_string());
    assert!(env["EMPTY_VAR"].is_string());
}

#[test]
fn test_preserve_unknown_fields() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let config = json!({
        "mcpServers": {
            "test": {
                "command": "test",
                "args": [],
                "unknownField": "should be preserved",
                "anotherUnknown": {
                    "nested": true
                }
            }
        },
        "globalUnknown": "also preserved"
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Read back and verify unknown fields are preserved
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(
        parsed["mcpServers"]["test"]["unknownField"],
        "should be preserved"
    );
    assert_eq!(
        parsed["mcpServers"]["test"]["anotherUnknown"]["nested"],
        true
    );
    assert_eq!(parsed["globalUnknown"], "also preserved");
}

#[test]
fn test_large_config_file() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let mut servers = serde_json::Map::new();

    // Create 100 servers
    for i in 0..100 {
        servers.insert(
            format!("server-{i}"),
            json!({
                "command": format!("command-{i}"),
                "args": vec![format!("arg-{i}-1"), format!("arg-{i}-2")],
                "env": {
                    "VAR1": format!("value-{i}-1"),
                    "VAR2": format!("value-{i}-2"),
                }
            }),
        );
    }

    let config = json!({
        "mcpServers": servers
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Read and verify
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["mcpServers"].as_object().unwrap().len(), 100);
    assert!(parsed["mcpServers"]["server-50"].is_object());
    assert_eq!(parsed["mcpServers"]["server-50"]["command"], "command-50");
}

#[test]
fn test_unicode_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("claude_desktop_config.json");

    let config = json!({
        "mcpServers": {
            "unicode-test": {
                "command": "test",
                "args": ["你好", "世界", "🌍", "émojis"],
                "env": {
                    "UNICODE_VAR": "Special chars: ñ é ü ß 中文"
                }
            }
        }
    });

    fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();

    // Read and verify unicode is preserved
    let content = fs::read_to_string(&config_path).unwrap();
    let parsed: Value = serde_json::from_str(&content).unwrap();

    assert_eq!(parsed["mcpServers"]["unicode-test"]["args"][0], "你好");
    assert_eq!(parsed["mcpServers"]["unicode-test"]["args"][2], "🌍");
    assert!(parsed["mcpServers"]["unicode-test"]["env"]["UNICODE_VAR"]
        .as_str()
        .unwrap()
        .contains("中文"));
}
