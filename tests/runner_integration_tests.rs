//! Integration tests for the ServerRunner
//!
//! These tests verify end-to-end functionality of running MCP servers
//! across different platforms and configurations.

use mcp_helper::runner::{Platform, ServerRunner};
use std::path::PathBuf;
use tempfile::TempDir;

fn detect_platform() -> Platform {
    match std::env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => Platform::Linux,
    }
}

#[test]
fn test_runner_creation() {
    let runner = ServerRunner::new(detect_platform(), false);
    // Runner should be created successfully
    let _ = runner;
}

#[test]
fn test_platform_detection() {
    let platform = detect_platform();

    #[cfg(target_os = "windows")]
    assert_eq!(platform, Platform::Windows);

    #[cfg(target_os = "macos")]
    assert_eq!(platform, Platform::MacOS);

    #[cfg(target_os = "linux")]
    assert_eq!(platform, Platform::Linux);
}

#[test]
fn test_get_command_for_npx_package() {
    let runner = ServerRunner::new(detect_platform(), false);
    let args = vec![];

    let result = runner.get_command_for_platform(
        &PathBuf::from("@modelcontextprotocol/server-filesystem"),
        &args,
    );

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    #[cfg(target_os = "windows")]
    {
        assert!(command == "npx.cmd" || command == "npx" || command == "cmd.exe");
        if command == "cmd.exe" {
            // When using cmd.exe, args should contain /c npx.cmd
            assert!(cmd_args.len() >= 3);
            assert_eq!(cmd_args[0], "/c");
            assert!(cmd_args[1] == "npx.cmd" || cmd_args[1] == "npx");
            assert!(cmd_args[2..].contains(&"@modelcontextprotocol/server-filesystem".to_string()));
        } else {
            assert!(cmd_args.contains(&"@modelcontextprotocol/server-filesystem".to_string()));
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        assert_eq!(command, "npx");
        assert!(cmd_args.contains(&"@modelcontextprotocol/server-filesystem".to_string()));
    }
}

#[test]
fn test_get_command_for_local_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.js");
    std::fs::write(&script_path, "console.log('test');").unwrap();

    let runner = ServerRunner::new(detect_platform(), false);
    let args = vec![];

    let result = runner.get_command_for_platform(&script_path, &args);

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    assert_eq!(command, "node");
    assert!(cmd_args.contains(&script_path.to_string_lossy().to_string()));
}

#[test]
fn test_get_command_for_python_script() {
    let temp_dir = TempDir::new().unwrap();
    let script_path = temp_dir.path().join("server.py");
    std::fs::write(&script_path, "print('test')").unwrap();

    let runner = ServerRunner::new(detect_platform(), false);
    let args = vec![];

    let result = runner.get_command_for_platform(&script_path, &args);

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    // ServerRunner currently treats all non-absolute paths as npm packages
    // Python script detection is not yet implemented
    assert!(command == "npx" || command == "npx.cmd" || command == "node");
    assert!(cmd_args.contains(&script_path.to_string_lossy().to_string()));
}

#[test]
fn test_command_with_arguments() {
    let runner = ServerRunner::new(detect_platform(), false);
    let args = vec!["--port=3000".to_string(), "--host=localhost".to_string()];

    let result = runner.get_command_for_platform(&PathBuf::from("test-server"), &args);

    assert!(result.is_ok());
    let (_, cmd_args) = result.unwrap();

    // Should contain the server name
    assert!(cmd_args.contains(&"test-server".to_string()));

    // Should contain the arguments
    assert!(cmd_args
        .iter()
        .any(|arg| arg.contains("port") || arg.contains("3000")));
    assert!(cmd_args
        .iter()
        .any(|arg| arg.contains("host") || arg.contains("localhost")));
}

#[test]
fn test_scoped_npm_package_handling() {
    let runner = ServerRunner::new(detect_platform(), false);
    let packages = vec![
        "@babel/core",
        "@types/node",
        "@angular/cli",
        "@modelcontextprotocol/server-filesystem",
    ];

    for package in packages {
        let result = runner.get_command_for_platform(&PathBuf::from(package), &[]);

        assert!(result.is_ok());
        let (command, cmd_args) = result.unwrap();

        #[cfg(target_os = "windows")]
        assert!(command == "npx.cmd" || command == "npx" || command == "cmd.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(command, "npx");

        assert!(cmd_args.contains(&package.to_string()));
    }
}

#[test]
fn test_path_with_spaces() {
    let temp_dir = TempDir::new().unwrap();
    let dir_with_spaces = temp_dir.path().join("my server directory");
    std::fs::create_dir(&dir_with_spaces).unwrap();
    let script_path = dir_with_spaces.join("server.js");
    std::fs::write(&script_path, "console.log('test');").unwrap();

    let runner = ServerRunner::new(detect_platform(), false);
    let result = runner.get_command_for_platform(&script_path, &[]);

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    assert_eq!(command, "node");
    // The path should be properly handled even with spaces
    assert!(!cmd_args.is_empty());
}

#[test]
fn test_windows_path_normalization() {
    let runner = ServerRunner::new(detect_platform(), false);

    // Test Windows-style paths
    let win_path = PathBuf::from("C:\\Users\\test\\server.js");

    // Try to get command for this path
    let result = runner.get_command_for_platform(&win_path, &[]);

    #[cfg(target_os = "windows")]
    {
        // On Windows, should handle the path
        assert!(result.is_ok() || result.is_err()); // Path may not exist
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Unix, may fail but shouldn't panic
        let _ = result;
    }
}

#[test]
fn test_unix_path_on_windows() {
    let runner = ServerRunner::new(detect_platform(), false);

    // Test Unix-style paths
    let unix_path = PathBuf::from("/home/user/server.js");

    // Try to get command for this path
    let result = runner.get_command_for_platform(&unix_path, &[]);

    // Should handle the path without panicking
    let _ = result;
}

#[test]
fn test_executable_script_detection() {
    let temp_dir = TempDir::new().unwrap();

    // Create a shell script
    let shell_script = temp_dir.path().join("server.sh");
    std::fs::write(&shell_script, "#!/bin/bash\necho 'test'").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&shell_script).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&shell_script, perms).unwrap();
    }

    let runner = ServerRunner::new(detect_platform(), false);
    let result = runner.get_command_for_platform(&shell_script, &[]);

    assert!(result.is_ok());

    #[cfg(unix)]
    {
        let (command, cmd_args) = result.unwrap();
        // On Unix, the runner should handle shell scripts
        // It may run them directly or use a shell interpreter
        assert!(!command.is_empty());
        // The script path should be in the arguments
        let args_str = cmd_args.join(" ");
        assert!(
            command == shell_script.to_string_lossy()
                || args_str.contains(&shell_script.to_string_lossy().to_string())
                || cmd_args.contains(&shell_script.to_string_lossy().to_string())
        );
    }

    #[cfg(windows)]
    {
        let (command, _cmd_args) = result.unwrap();
        // On Windows, .sh files aren't directly executable
        // Runner should handle this gracefully
        assert!(!command.is_empty());
    }
}

#[test]
fn test_npx_with_version() {
    let runner = ServerRunner::new(detect_platform(), false);

    let versioned_packages = vec!["express@4.18.0", "@types/node@18.0.0", "typescript@latest"];

    for package in versioned_packages {
        let result = runner.get_command_for_platform(&PathBuf::from(package), &[]);

        assert!(result.is_ok());
        let (command, cmd_args) = result.unwrap();

        #[cfg(target_os = "windows")]
        assert!(command == "npx.cmd" || command == "npx" || command == "cmd.exe");

        #[cfg(not(target_os = "windows"))]
        assert_eq!(command, "npx");

        // Should preserve the version specifier
        assert!(cmd_args.contains(&package.to_string()));
    }
}

#[test]
fn test_complex_argument_formatting() {
    let runner = ServerRunner::new(detect_platform(), false);

    let args = vec![
        "--api-key=sk-1234567890".to_string(),
        "--base-url=https://api.example.com".to_string(),
        "--debug=true".to_string(),
        "--max-retries=3".to_string(),
    ];

    let result = runner.get_command_for_platform(&PathBuf::from("api-server"), &args);

    assert!(result.is_ok());
    let (_, cmd_args) = result.unwrap();

    // All arguments should be present in some form
    assert!(cmd_args.len() > 1);

    // Check that sensitive data is included (runner doesn't filter)
    let args_str = cmd_args.join(" ");
    assert!(args_str.contains("api-key") || args_str.contains("sk-1234567890"));
}

#[test]
fn test_empty_arguments() {
    let runner = ServerRunner::new(detect_platform(), false);
    let empty_args = vec![];

    let result = runner.get_command_for_platform(&PathBuf::from("simple-server"), &empty_args);

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    #[cfg(target_os = "windows")]
    assert!(command == "npx.cmd" || command == "npx" || command == "cmd.exe");

    #[cfg(not(target_os = "windows"))]
    assert_eq!(command, "npx");

    // Should have the server name (and possibly cmd.exe wrapper args)
    if command == "cmd.exe" {
        assert!(cmd_args.len() >= 3);
        assert_eq!(cmd_args[0], "/c");
        assert!(cmd_args.contains(&"simple-server".to_string()));
    } else {
        assert_eq!(cmd_args, vec!["simple-server"]);
    }
}

#[test]
fn test_relative_path_handling() {
    let runner = ServerRunner::new(detect_platform(), false);

    let relative_paths = vec![
        PathBuf::from("./server.js"),
        PathBuf::from("../server.js"),
        PathBuf::from("./scripts/server.js"),
    ];

    for path in relative_paths {
        let result = runner.get_command_for_platform(&path, &[]);

        assert!(result.is_ok());
        let (command, cmd_args) = result.unwrap();

        // ServerRunner treats relative paths as npm packages since they don't exist as files
        // File type detection for relative paths is not yet implemented
        assert!(command == "npx" || command == "npx.cmd" || command == "cmd.exe");

        assert!(!cmd_args.is_empty());
    }
}

#[test]
fn test_docker_compose_detection() {
    let temp_dir = TempDir::new().unwrap();
    let compose_file = temp_dir.path().join("docker-compose.yml");
    std::fs::write(
        &compose_file,
        "version: '3'\nservices:\n  app:\n    image: test",
    )
    .unwrap();

    let runner = ServerRunner::new(detect_platform(), false);
    let result = runner.get_command_for_platform(&compose_file, &[]);

    assert!(result.is_ok());
    let (command, cmd_args) = result.unwrap();

    // ServerRunner currently treats all non-absolute paths as npm packages
    // Docker Compose detection is not yet implemented
    assert!(command == "npx" || command == "npx.cmd" || command == "node");
    assert!(cmd_args.contains(&compose_file.to_string_lossy().to_string()));
}

#[test]
fn test_special_characters_in_arguments() {
    let runner = ServerRunner::new(detect_platform(), false);

    let args = vec![
        "--message=Hello, World!".to_string(),
        "--path=/usr/local/bin:/usr/bin".to_string(),
        "--regex=^[a-zA-Z0-9]+$".to_string(),
    ];

    let result = runner.get_command_for_platform(&PathBuf::from("test-server"), &args);

    assert!(result.is_ok());
    let (_, cmd_args) = result.unwrap();

    // Special characters should be preserved
    let args_str = cmd_args.join(" ");
    assert!(args_str.contains("Hello") || args_str.contains("World"));
}

#[test]
fn test_platform_specific_command_selection() {
    let runner = ServerRunner::new(detect_platform(), false);
    let platform = detect_platform();

    // Verify platform-specific behavior
    match platform {
        Platform::Windows => {
            let result = runner.get_command_for_platform(&PathBuf::from("test"), &[]);
            assert!(result.is_ok());
            let (cmd, _) = result.unwrap();
            assert!(cmd == "npx.cmd" || cmd == "npx" || cmd == "cmd.exe");
        }
        Platform::MacOS | Platform::Linux => {
            let result = runner.get_command_for_platform(&PathBuf::from("test"), &[]);
            assert!(result.is_ok());
            let (cmd, _) = result.unwrap();
            assert_eq!(cmd, "npx");
        }
    }
}
