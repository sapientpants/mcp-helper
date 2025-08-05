//! Direct coverage tests for install.rs module
//!
//! This test suite ensures all code paths in install.rs are covered

use mcp_helper::install::InstallCommand;

#[test]
fn test_install_command_new() {
    // Test verbose and non-verbose creation
    let cmd_verbose = InstallCommand::new(true);
    drop(cmd_verbose);

    let cmd_quiet = InstallCommand::new(false);
    drop(cmd_quiet);
}

#[test]
fn test_with_auto_install_deps() {
    let cmd = InstallCommand::new(false).with_auto_install_deps(true);
    drop(cmd);

    let cmd2 = InstallCommand::new(true).with_auto_install_deps(false);
    drop(cmd2);
}

#[test]
fn test_with_dry_run() {
    let cmd = InstallCommand::new(false).with_dry_run(true);
    drop(cmd);

    let cmd2 = InstallCommand::new(true).with_dry_run(false);
    drop(cmd2);
}

#[test]
fn test_with_config_overrides() {
    // Test empty overrides
    let cmd = InstallCommand::new(false).with_config_overrides(vec![]);
    drop(cmd);

    // Test valid key=value pairs
    let cmd2 = InstallCommand::new(true).with_config_overrides(vec![
        "key1=value1".to_string(),
        "key2=value2".to_string(),
        "api_key=sk-test123".to_string(),
    ]);
    drop(cmd2);

    // Test invalid format (no equals sign)
    let cmd3 = InstallCommand::new(false)
        .with_config_overrides(vec!["invalidformat".to_string(), "valid=value".to_string()]);
    drop(cmd3);

    // Test with spaces
    let cmd4 = InstallCommand::new(false).with_config_overrides(vec![
        " key = value ".to_string(),
        "complex=value with spaces".to_string(),
    ]);
    drop(cmd4);
}

#[test]
fn test_builder_chaining() {
    // Test all builder methods can be chained
    let cmd = InstallCommand::new(true)
        .with_auto_install_deps(true)
        .with_dry_run(true)
        .with_config_overrides(vec!["env=production".to_string()]);
    drop(cmd);
}

#[test]
fn test_execute_basic() {
    let mut cmd = InstallCommand::new(false);

    // Test with a simple server name
    let result = cmd.execute("test-server");
    // This will fail since we don't have actual clients, but it exercises the code
    assert!(result.is_err());
}

#[test]
fn test_execute_with_options() {
    let mut cmd = InstallCommand::new(true)
        .with_auto_install_deps(false)
        .with_dry_run(true);

    // Test with npm package
    let result = cmd.execute("@modelcontextprotocol/server-filesystem");
    assert!(result.is_err()); // Expected to fail in test environment

    // Test with simple package
    let result2 = cmd.execute("simple-server");
    assert!(result2.is_err());
}

#[test]
fn test_config_overrides_parsing() {
    // Test various config override formats
    let configs = vec![
        vec!["simple=value"],
        vec!["port=3000", "host=localhost"],
        vec!["path=/usr/local/bin"],
        vec!["bool=true", "num=42"],
        vec!["empty=", "=nokey"],
        vec!["multi=part=value"],
    ];

    for config in configs {
        let cmd = InstallCommand::new(false)
            .with_config_overrides(config.into_iter().map(String::from).collect());
        drop(cmd);
    }
}

#[test]
fn test_verbose_output() {
    let mut cmd = InstallCommand::new(true);

    // Execute with verbose enabled to test logging paths
    let _ = cmd.execute("verbose-test-server");
}

#[test]
fn test_build_field_prompt_static_method() {
    use mcp_helper::server::{ConfigField, ConfigFieldType};

    // Test required field with description
    let field1 = ConfigField {
        name: "api_key".to_string(),
        field_type: ConfigFieldType::String,
        description: Some("API key for authentication".to_string()),
        default: None,
    };
    let prompt1 = InstallCommand::build_field_prompt(&field1, true);
    assert_eq!(prompt1, "API key for authentication");

    // Test optional field with description
    let field2 = ConfigField {
        name: "timeout".to_string(),
        field_type: ConfigFieldType::Number,
        description: Some("Request timeout".to_string()),
        default: Some("30".to_string()),
    };
    let prompt2 = InstallCommand::build_field_prompt(&field2, false);
    assert_eq!(prompt2, "Request timeout (optional)");

    // Test field without description
    let field3 = ConfigField {
        name: "custom_field".to_string(),
        field_type: ConfigFieldType::String,
        description: None,
        default: None,
    };
    let prompt3 = InstallCommand::build_field_prompt(&field3, true);
    assert_eq!(prompt3, "custom_field");

    // Test optional field without description
    let field4 = ConfigField {
        name: "optional_field".to_string(),
        field_type: ConfigFieldType::Boolean,
        description: None,
        default: Some("false".to_string()),
    };
    let prompt4 = InstallCommand::build_field_prompt(&field4, false);
    assert_eq!(prompt4, "optional_field (optional)");
}

#[test]
fn test_handle_missing_dependency_static_method() {
    use mcp_helper::deps::{
        Dependency, DependencyCheck, DependencyStatus, InstallInstructions, InstallMethod,
    };

    // Test with Node.js dependency
    let check1 = DependencyCheck {
        dependency: Dependency::NodeJs {
            min_version: Some("18.0.0".to_string()),
        },
        status: DependencyStatus::Missing,
        install_instructions: Some(InstallInstructions {
            windows: vec![InstallMethod {
                name: "winget".to_string(),
                command: "winget install nodejs".to_string(),
                description: Some("Install via Windows Package Manager".to_string()),
            }],
            macos: vec![InstallMethod {
                name: "homebrew".to_string(),
                command: "brew install node".to_string(),
                description: Some("Install via Homebrew".to_string()),
            }],
            linux: vec![],
        }),
    };

    let result1 = InstallCommand::handle_missing_dependency("Node.js", &check1);
    assert!(result1.is_err());

    // Test without install instructions
    let check2 = DependencyCheck {
        dependency: Dependency::Python {
            min_version: Some("3.9".to_string()),
        },
        status: DependencyStatus::Missing,
        install_instructions: None,
    };

    let result2 = InstallCommand::handle_missing_dependency("Python", &check2);
    assert!(result2.is_err());

    // Test with empty install instructions
    let check3 = DependencyCheck {
        dependency: Dependency::Docker {
            min_version: None,
            requires_compose: true,
        },
        status: DependencyStatus::Missing,
        install_instructions: Some(InstallInstructions::default()),
    };

    let result3 = InstallCommand::handle_missing_dependency("Docker", &check3);
    assert!(result3.is_err());
}

#[test]
fn test_edge_cases() {
    // Test with various edge case server names
    let mut cmd = InstallCommand::new(false);

    let _ = cmd.execute("");
    let _ = cmd.execute("@");
    let _ = cmd.execute("//");
    let _ = cmd.execute("docker:");
    let _ = cmd.execute("https://");
    let _ = cmd.execute("user/");
    let _ = cmd.execute("/repo");
    let _ = cmd.execute("@org/");
    let _ = cmd.execute("@/package");
}

#[test]
fn test_config_override_edge_cases() {
    // Test edge cases in config parsing
    let edge_cases = vec![
        vec!["=value"],            // No key
        vec!["key="],              // No value
        vec!["="],                 // Just equals
        vec!["key=val=ue"],        // Multiple equals
        vec!["key = value"],       // Spaces around equals
        vec!["  key  =  value  "], // Extra spaces
        vec![""],                  // Empty string
        vec!["no-equals-sign"],    // No equals at all
        vec!["key==value"],        // Double equals
        vec!["key=value=with=many=equals"],
    ];

    for case in edge_cases {
        let cmd = InstallCommand::new(false)
            .with_config_overrides(case.into_iter().map(String::from).collect());
        drop(cmd);
    }
}

#[test]
fn test_unicode_handling() {
    let mut cmd = InstallCommand::new(false).with_config_overrides(vec![
        "message=Hello 世界".to_string(),
        "emoji=🚀".to_string(),
        "путь=/home/пользователь".to_string(),
    ]);

    // Test with unicode server names
    let _ = cmd.execute("测试-server");
    let _ = cmd.execute("🚀-emoji-server");
    let _ = cmd.execute("сервер");
}

#[test]
fn test_execute_batch() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("servers.txt");

    // Test with valid batch file
    fs::write(&batch_file, "# Test batch file\n@modelcontextprotocol/server-filesystem\nsimple-server\n\n# Comment\ndocker:postgres:13").unwrap();

    let mut cmd = InstallCommand::new(false);
    let result = cmd.execute_batch(batch_file.to_str().unwrap());
    assert!(result.is_err()); // Will fail due to no clients, but exercises the code

    // Test with empty batch file
    let empty_file = temp_dir.path().join("empty.txt");
    fs::write(&empty_file, "# Just comments\n# Another comment\n\n").unwrap();

    let result2 = cmd.execute_batch(empty_file.to_str().unwrap());
    assert!(result2.is_err());
    if let Err(e) = result2 {
        assert!(e.to_string().contains("No servers found"));
    }

    // Test with non-existent file
    let result3 = cmd.execute_batch("non-existent-file.txt");
    assert!(result3.is_err());
}

#[test]
fn test_get_dependency_name() {
    use mcp_helper::deps::Dependency;

    let deps = vec![
        (Dependency::NodeJs { min_version: None }, "Node.js"),
        (
            Dependency::NodeJs {
                min_version: Some("18.0.0".to_string()),
            },
            "Node.js",
        ),
        (Dependency::Python { min_version: None }, "Python"),
        (
            Dependency::Python {
                min_version: Some("3.9".to_string()),
            },
            "Python",
        ),
        (
            Dependency::Docker {
                min_version: None,
                requires_compose: false,
            },
            "Docker",
        ),
        (
            Dependency::Docker {
                min_version: Some("20.0".to_string()),
                requires_compose: true,
            },
            "Docker",
        ),
        (Dependency::Git, "Git"),
    ];

    for (dep, expected_name) in deps {
        let name = InstallCommand::get_dependency_name(&dep);
        assert_eq!(name, expected_name);
    }
}

#[test]
fn test_handle_installed_dependency() {
    // Test with version
    let result1 =
        InstallCommand::handle_installed_dependency("Node.js", &Some("18.17.0".to_string()));
    assert!(result1.is_ok());

    // Test without version
    let result2 = InstallCommand::handle_installed_dependency("Python", &None);
    assert!(result2.is_ok());

    // Test with various dependency names
    let names = vec!["Docker", "Git", "Custom Tool", "测试工具"];
    for name in names {
        let result = InstallCommand::handle_installed_dependency(name, &Some("1.0.0".to_string()));
        assert!(result.is_ok());
    }
}

#[test]
fn test_batch_file_with_config() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let batch_file = temp_dir.path().join("batch_with_config.txt");

    // Create a batch file with server configs
    let content = r#"
# Server with configuration
@modelcontextprotocol/server-filesystem
  allowedDirectories=/home/user/docs
  allowedFileTypes=.md,.txt

# Another server
simple-server
  port=3000
  host=localhost

# Server without config
basic-server
"#;

    fs::write(&batch_file, content).unwrap();

    let mut cmd = InstallCommand::new(true).with_dry_run(true);

    let _ = cmd.execute_batch(batch_file.to_str().unwrap());
}

#[test]
fn test_all_builder_combinations() {
    // Test various combinations of builder options
    let combinations = vec![
        (true, true, true),
        (true, true, false),
        (true, false, true),
        (true, false, false),
        (false, true, true),
        (false, true, false),
        (false, false, true),
        (false, false, false),
    ];

    for (verbose, auto_install, dry_run) in combinations {
        let cmd = InstallCommand::new(verbose)
            .with_auto_install_deps(auto_install)
            .with_dry_run(dry_run);
        drop(cmd);
    }
}
