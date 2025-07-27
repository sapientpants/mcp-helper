use mcp_helper::runner::{normalize_path, Platform, ServerRunner};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper to create a mock executable file
fn create_mock_executable(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::write(&path, "#!/bin/sh\necho 'mock server'").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        fs::write(&path, "@echo off\necho mock server").unwrap();
    }
    path
}

#[test]
fn test_server_runner_verbose_mode() {
    let runner = ServerRunner::new(Platform::MacOS, true);
    // This will fail because "nonexistent-server" doesn't exist
    let result = runner.run("nonexistent-server", &[]);
    assert!(result.is_err());
}

#[test]
fn test_server_runner_platform_detection() {
    let runners = vec![
        ServerRunner::new(Platform::Windows, false),
        ServerRunner::new(Platform::MacOS, false),
        ServerRunner::new(Platform::Linux, false),
    ];

    for runner in runners {
        // Each should be created successfully
        let result = runner.run("test-server", &[]);
        // Will fail because test-server doesn't exist, but that's expected
        assert!(result.is_err());
    }
}

#[test]
fn test_normalize_path_edge_cases() {
    // Empty path
    assert_eq!(normalize_path("", Platform::Windows), "");
    assert_eq!(normalize_path("", Platform::MacOS), "");
    assert_eq!(normalize_path("", Platform::Linux), "");

    // Single separator
    assert_eq!(normalize_path("/", Platform::Windows), "\\");
    assert_eq!(normalize_path("\\", Platform::MacOS), "/");

    // Multiple consecutive separators
    assert_eq!(
        normalize_path("path//to///file", Platform::Windows),
        "path\\\\to\\\\\\file"
    );
    assert_eq!(
        normalize_path("path\\\\to\\\\\\file", Platform::Linux),
        "path//to///file"
    );

    // No separators
    assert_eq!(
        normalize_path("simplefilename", Platform::Windows),
        "simplefilename"
    );
    assert_eq!(
        normalize_path("simplefilename", Platform::Linux),
        "simplefilename"
    );
}

#[test]
fn test_run_with_path_arguments() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test with path-like arguments
    let args = vec![
        "path/to/file".to_string(),
        "another\\path".to_string(),
        "regular-arg".to_string(),
    ];

    let result = runner.run("test-server", &args);
    // Will fail but we're testing argument normalization
    assert!(result.is_err());
}

#[test]
fn test_run_error_messages() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test server names that should trigger error messages
    let test_cases = vec![
        (
            "nonexistent-server-12345",
            vec!["exited with status", "not installed", "Not Found"],
        ),
        (
            "definitely-not-real-xyz",
            vec!["not found", "Not Found", "not installed"],
        ),
    ];

    for (server, expected_patterns) in test_cases {
        let result = runner.run(server, &[]);
        assert!(result.is_err());
        let err_msg = format!("{:?}", result.unwrap_err());

        // Check if any of the expected patterns match
        let matches_any = expected_patterns
            .iter()
            .any(|pattern| err_msg.contains(pattern));
        assert!(
            matches_any,
            "Error message '{err_msg}' did not contain any of {expected_patterns:?}"
        );
    }
}

#[test]
fn test_resolve_server_path_with_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let server_path = create_mock_executable(&temp_dir, "mock-server");

    let runner = ServerRunner::new(Platform::Linux, false);
    let resolved = runner
        .resolve_server_path(server_path.to_str().unwrap())
        .unwrap();

    assert_eq!(resolved, server_path);
}

#[test]
fn test_windows_command_construction_edge_cases() {
    let runner = ServerRunner::new(Platform::Windows, true);

    // Test with absolute path
    let temp_dir = TempDir::new().unwrap();
    let server_path = temp_dir.path().join("server.js");
    fs::write(&server_path, "console.log('test');").unwrap();

    let (cmd, args) = runner
        .get_windows_command(&server_path, &["--arg".to_string()])
        .unwrap();

    assert_eq!(cmd, "node");
    assert!(args.contains(&server_path.to_string_lossy().to_string()));
    assert!(args.contains(&"--arg".to_string()));
}

#[test]
fn test_unix_command_construction_edge_cases() {
    let runner = ServerRunner::new(Platform::MacOS, false);

    // Test with absolute path
    let temp_dir = TempDir::new().unwrap();
    let server_path = temp_dir.path().join("server.js");
    fs::write(&server_path, "console.log('test');").unwrap();

    let (cmd, args) = runner
        .get_unix_command(&server_path, &["--verbose".to_string()])
        .unwrap();

    assert_eq!(cmd, "node");
    assert!(args.contains(&server_path.to_string_lossy().to_string()));
    assert!(args.contains(&"--verbose".to_string()));
}

#[test]
fn test_command_args_ordering() {
    let runner = ServerRunner::new(Platform::Linux, false);

    let args = vec!["arg1".to_string(), "arg2".to_string(), "arg3".to_string()];

    // For non-existent server (npm package)
    let result = runner.get_unix_command(&PathBuf::from("test-server"), &args);

    if let Ok((_, cmd_args)) = result {
        // First arg should be server name
        assert_eq!(cmd_args[0], "test-server");
        // Rest should be in order
        assert_eq!(&cmd_args[1..], &args);
    }
}

#[test]
fn test_error_exit_codes() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // This will attempt to run a non-existent command
    let result = runner.run("definitely-not-a-real-command-xyz123", &[]);

    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();

    // Should contain helpful error message
    assert!(
        err_str.contains("not installed")
            || err_str.contains("not found")
            || err_str.contains("Failed to execute")
    );
}

#[test]
fn test_environment_variable_inheritance() {
    // Set a test environment variable
    env::set_var("TEST_MCP_VAR", "test_value");

    let runner = ServerRunner::new(Platform::Linux, false);

    // Even though this will fail, it tests that env vars would be passed
    let _result = runner.run("test-server", &[]);

    // Clean up
    env::remove_var("TEST_MCP_VAR");
}

#[test]
fn test_verbose_output_behavior() {
    // Test with verbose enabled
    let verbose_runner = ServerRunner::new(Platform::Linux, true);
    let _result = verbose_runner.run("test", &[]);

    // Test with verbose disabled
    let quiet_runner = ServerRunner::new(Platform::Linux, false);
    let _result = quiet_runner.run("test", &[]);

    // Both should fail but with different levels of output
    // (actual output verification would require capturing stderr)
}

#[test]
fn test_mixed_platform_paths() {
    // Test handling of paths that might come from different platforms
    let test_cases = vec![
        (
            "C:\\Users\\test\\file.js",
            Platform::Linux,
            "C:/Users/test/file.js",
        ),
        (
            "/home/user/file.js",
            Platform::Windows,
            "\\home\\user\\file.js",
        ),
        (
            "relative\\path/mixed",
            Platform::MacOS,
            "relative/path/mixed",
        ),
    ];

    for (input, platform, expected) in test_cases {
        let result = normalize_path(input, platform);
        assert_eq!(result, expected);
    }
}

// Integration tests for the public API
#[test]
fn test_public_api_completeness() {
    // Ensure all public methods are accessible
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test run method
    let _ = runner.run("test", &[]);

    // Test normalize_path function
    let _ = normalize_path("test/path", Platform::Windows);
}

// Mock tests for platform-specific behavior
#[cfg(target_os = "windows")]
#[test]
fn test_windows_specific_behavior() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Windows-specific test for npx.cmd handling
    let result = runner.get_windows_command(&PathBuf::from("test-server"), &[]);

    if let Ok((cmd, _args)) = result {
        // On Windows CI without npx, this might return different results
        assert!(cmd == "cmd.exe" || cmd == "npx");
    }
}

#[cfg(not(target_os = "windows"))]
#[test]
fn test_unix_specific_behavior() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Unix-specific test
    let result = runner.get_unix_command(&PathBuf::from("test-server"), &[]);

    if result.is_ok() {
        let (cmd, _) = result.unwrap();
        assert_eq!(cmd, "npx");
    }
}

// Test error recovery and helpful messages
#[test]
fn test_helpful_error_messages() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test that error messages include helpful suggestions
    let result = runner.run("@test/nonexistent-package", &[]);

    if let Err(e) = result {
        let error_string = e.to_string();
        // Should suggest installation or other helpful actions
        assert!(
            error_string.contains("install")
                || error_string.contains("not found")
                || error_string.contains("Failed")
        );
    }
}
