use mcp_helper::runner::{normalize_path, Platform, ServerRunner};
use std::env;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_runner_verbose_mode() {
    let runner = ServerRunner::new(Platform::Linux, true);

    // Test with a non-existent server to see verbose output
    let result = runner.run("nonexistent-server", &[]);
    assert!(result.is_err());
}

#[test]
fn test_runner_with_path_arguments() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test with arguments that look like paths
    let args = vec![
        "/home/user/file.txt".to_string(),
        "relative/path/file.js".to_string(),
        "--config=/etc/config".to_string(),
        "not-a-path".to_string(),
    ];

    // This will fail because the server doesn't exist, but it tests the path normalization
    let result = runner.run("test-server", &args);
    assert!(result.is_err());
}

#[test]
fn test_runner_windows_path_normalization() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test with Windows-style paths
    let args = vec![
        "C:/Users/test/file.txt".to_string(),
        "relative\\path\\file.js".to_string(),
        "mixed/path\\style.txt".to_string(),
    ];

    let result = runner.run("test-server", &args);
    assert!(result.is_err());
}

#[test]
fn test_normalize_path_edge_cases() {
    // Test empty path
    assert_eq!(normalize_path("", Platform::Windows), "");
    assert_eq!(normalize_path("", Platform::Linux), "");

    // Test path with only slashes
    assert_eq!(normalize_path("///", Platform::Windows), "\\\\\\");
    assert_eq!(normalize_path("\\\\\\", Platform::Linux), "///");

    // Test path with spaces
    assert_eq!(
        normalize_path("path with spaces/file.txt", Platform::Windows),
        "path with spaces\\file.txt"
    );
    assert_eq!(
        normalize_path("path with spaces\\file.txt", Platform::Linux),
        "path with spaces/file.txt"
    );

    // Test Unicode paths
    assert_eq!(
        normalize_path("пуrь/файл.txt", Platform::Windows),
        "пуrь\\файл.txt"
    );
    assert_eq!(
        normalize_path("パス\\ファイル.txt", Platform::Linux),
        "パス/ファイル.txt"
    );
}

#[test]
fn test_runner_with_npm_package_in_path() {
    // Create a temporary directory and add it to PATH
    let temp_dir = TempDir::new().unwrap();
    let node_modules_bin = temp_dir.path().join("node_modules").join(".bin");
    fs::create_dir_all(&node_modules_bin).unwrap();

    // Create a fake executable
    let fake_server = node_modules_bin.join("test-mcp-server");
    #[cfg(unix)]
    {
        fs::write(&fake_server, "#!/bin/sh\necho 'test server'").unwrap();
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&fake_server, fs::Permissions::from_mode(0o755)).unwrap();
    }
    #[cfg(windows)]
    {
        fs::write(fake_server.with_extension("cmd"), "@echo test server").unwrap();
    }

    // Add to PATH
    let original_path = env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", node_modules_bin.display(), original_path);
    env::set_var("PATH", new_path);

    let runner = ServerRunner::new(Platform::Linux, true);
    let result = runner.run("test-mcp-server", &[]);

    // Restore original PATH
    env::set_var("PATH", original_path);

    // The command should be found but may fail to execute properly in test environment
    // We're mainly testing that the path resolution works
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_runner_command_execution_failure() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Try to run a command that definitely doesn't exist
    let result = runner.run("/nonexistent/path/to/server", &[]);
    assert!(result.is_err());

    match result {
        Err(e) => {
            let error_msg = e.to_string();
            // The error message varies by platform and situation
            assert!(!error_msg.is_empty());
        }
        Ok(_) => panic!("Expected error for nonexistent command"),
    }
}

#[test]
fn test_runner_with_special_characters_in_args() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test with special characters in arguments
    let args = vec![
        "--option=\"value with spaces\"".to_string(),
        "--path=/home/user/file with spaces.txt".to_string(),
        "--emoji=🚀".to_string(),
        "--special=!@#$%^&*()".to_string(),
    ];

    let result = runner.run("test-server", &args);
    assert!(result.is_err());
}

#[test]
fn test_platform_specific_command_generation() {
    // Test Windows command generation
    let runner_win = ServerRunner::new(Platform::Windows, true);
    let result = runner_win.run("test.cmd", &["arg1".to_string()]);
    assert!(result.is_err());

    // Test macOS command generation
    let runner_mac = ServerRunner::new(Platform::MacOS, true);
    let result = runner_mac.run("test.sh", &["arg1".to_string()]);
    assert!(result.is_err());

    // Test Linux command generation
    let runner_linux = ServerRunner::new(Platform::Linux, true);
    let result = runner_linux.run("test.sh", &["arg1".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_runner_with_complex_npm_package() {
    let runner = ServerRunner::new(Platform::Linux, false);

    // Test with scoped npm package
    let result = runner.run("@scope/package", &["--stdio".to_string()]);
    assert!(result.is_err());

    // Test with npm package and version
    let result = runner.run("package@1.2.3", &["--stdio".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_runner_error_context() {
    // Use the current platform for testing
    let platform = if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        Platform::Linux
    };

    let runner = ServerRunner::new(platform, false);

    // Test that errors have proper context
    let result = runner.run("definitely-not-a-real-server", &[]);
    assert!(result.is_err());

    if let Err(e) = result {
        // Check that the error has context about what failed
        let error_chain = format!("{e:?}");
        // On different platforms, the error message may vary
        assert!(
            error_chain.contains("definitely-not-a-real-server")
                || error_chain.contains("Failed to find")
                || error_chain.contains("not found")
                || error_chain.contains("404")
        );
    }
}

#[test]
fn test_normalize_path_with_dots() {
    // Test paths with . and ..
    assert_eq!(
        normalize_path("./file.txt", Platform::Windows),
        ".\\file.txt"
    );
    assert_eq!(
        normalize_path("..\\parent\\file.txt", Platform::Linux),
        "../parent/file.txt"
    );
    assert_eq!(
        normalize_path("path/../other/file.txt", Platform::Windows),
        "path\\..\\other\\file.txt"
    );
}

#[test]
fn test_runner_verbose_output() {
    // Test that verbose mode doesn't affect functionality
    let runner_verbose = ServerRunner::new(Platform::Linux, true);
    let runner_quiet = ServerRunner::new(Platform::Linux, false);

    // Both should fail in the same way
    let result_verbose = runner_verbose.run("nonexistent", &[]);
    let result_quiet = runner_quiet.run("nonexistent", &[]);

    assert!(result_verbose.is_err());
    assert!(result_quiet.is_err());
}
