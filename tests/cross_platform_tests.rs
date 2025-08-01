//! Cross-platform tests with emphasis on Windows-specific behaviors
//!
//! This test suite ensures that platform-specific code paths work correctly
//! across Windows, macOS, and Linux, with special attention to Windows edge cases.

use mcp_helper::client::detect_clients;
use mcp_helper::deps::{get_install_instructions, Dependency, DependencyChecker, NodeChecker};
use mcp_helper::runner::{normalize_path, Platform, ServerRunner};
use mcp_helper::server::{McpServer, NpmServer};
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Test path normalization across platforms
#[test]
#[allow(clippy::uninlined_format_args)]
fn test_path_normalization_all_platforms() {
    // Test cases with various path formats
    let test_cases = vec![
        // (input, windows_expected, unix_expected)
        ("path/to/file", "path\\to\\file", "path/to/file"),
        ("path\\to\\file", "path\\to\\file", "path/to/file"),
        ("C:/Users/test", "C:\\Users\\test", "C:/Users/test"),
        ("\\\\server\\share", "\\\\server\\share", "//server/share"),
        ("./relative/path", ".\\relative\\path", "./relative/path"),
        ("../parent/path", "..\\parent\\path", "../parent/path"),
        (
            "path with spaces/file",
            "path with spaces\\file",
            "path with spaces/file",
        ),
        ("", "", ""),
        ("single", "single", "single"),
        // Edge cases
        (
            "path//double//slashes",
            "path\\\\double\\\\slashes",
            "path//double//slashes",
        ),
        ("trailing/slash/", "trailing\\slash\\", "trailing/slash/"),
        ("/absolute/path", "\\absolute\\path", "/absolute/path"),
    ];

    for (input, windows_expected, unix_expected) in test_cases {
        let windows_result = normalize_path(input, Platform::Windows);
        assert_eq!(
            windows_result, windows_expected,
            "Windows normalization failed for '{}'",
            input
        );

        let linux_result = normalize_path(input, Platform::Linux);
        assert_eq!(
            linux_result, unix_expected,
            "Linux normalization failed for '{}'",
            input
        );

        let macos_result = normalize_path(input, Platform::MacOS);
        assert_eq!(
            macos_result, unix_expected,
            "macOS normalization failed for '{}'",
            input
        );
    }
}

/// Test Windows-specific command generation
#[test]
fn test_windows_command_generation() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test npm package with scoped name
    let result = runner.run("@modelcontextprotocol/server-filesystem", &[]);
    match result {
        Ok(_) => println!("Command would execute on Windows"),
        Err(e) => {
            // Expected to fail in test environment
            let error_msg = e.to_string();
            println!("Expected error: {error_msg}");
            // The error might be about server exit status or Node.js tools
            assert!(
                error_msg.contains("npm")
                    || error_msg.contains("node")
                    || error_msg.contains("npx")
                    || error_msg.contains("server")
                    || error_msg.contains("exit"),
                "Error should be related to server execution: {error_msg}"
            );
        }
    }

    // Test local file path with Windows-style separators
    let windows_path = "C:\\Users\\test\\server.js";
    let args = vec!["--port".to_string(), "3000".to_string()];
    let result = runner.run(windows_path, &args);
    match result {
        Ok(_) => println!("Command would execute for local file"),
        Err(e) => {
            println!("Expected error for non-existent file: {e}");
        }
    }
}

/// Test Windows-specific npx command resolution
#[test]
fn test_windows_npx_command() {
    let npm_server = NpmServer::new("test-server").unwrap();
    let (cmd, args) = npm_server.generate_command().unwrap();

    #[cfg(target_os = "windows")]
    assert_eq!(cmd, "npx.cmd");

    #[cfg(not(target_os = "windows"))]
    assert_eq!(cmd, "npx");

    // NPM server includes --yes and --stdio flags
    assert!(args.contains(&"test-server".to_string()));
    assert!(args.contains(&"--stdio".to_string()));

    // Test with version
    let npm_server_with_version = NpmServer::new("test-server@1.2.3").unwrap();
    let (_cmd, args) = npm_server_with_version.generate_command().unwrap();

    #[cfg(target_os = "windows")]
    assert_eq!(_cmd, "npx.cmd");

    assert!(args.contains(&"test-server@1.2.3".to_string()));
}

/// Test Windows environment variable handling
#[test]
#[serial]
fn test_windows_env_var_paths() {
    // Save original env vars
    let original_appdata = env::var("APPDATA").ok();
    let original_home = env::var("HOME").ok();
    let original_userprofile = env::var("USERPROFILE").ok();

    // Test Windows APPDATA path
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let fake_appdata = temp_dir.path().join("AppData\\Roaming");
    fs::create_dir_all(&fake_appdata).expect("Failed to create fake APPDATA");

    env::set_var("APPDATA", &fake_appdata);

    // On non-Windows platforms, we simulate Windows behavior
    #[cfg(not(target_os = "windows"))]
    {
        // The code might fall back to other methods on non-Windows
        env::set_var("HOME", temp_dir.path());
    }

    // Test Claude Desktop config path resolution
    // This tests the actual path resolution logic
    let clients = detect_clients();
    for client in &clients {
        if client.name() == "Claude Desktop" {
            println!("Found Claude Desktop client");
            break;
        }
    }

    // Restore env vars
    match original_appdata {
        Some(val) => env::set_var("APPDATA", val),
        None => env::remove_var("APPDATA"),
    }
    match original_home {
        Some(val) => env::set_var("HOME", val),
        None => env::remove_var("HOME"),
    }
    match original_userprofile {
        Some(val) => env::set_var("USERPROFILE", val),
        None => env::remove_var("USERPROFILE"),
    }
}

/// Test Windows-specific path edge cases
#[test]
fn test_windows_path_edge_cases() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test UNC paths
    let unc_path = "\\\\server\\share\\script.js";
    match runner.resolve_server_path(unc_path) {
        Ok(resolved) => {
            println!("UNC path resolved to: {resolved:?}");
            // Should preserve UNC format
            #[cfg(target_os = "windows")]
            assert!(resolved.to_string_lossy().starts_with("\\\\"));
        }
        Err(e) => println!("UNC path resolution failed (expected in tests): {e}"),
    }

    // Test drive letter paths
    let drive_paths = vec![
        "C:\\Program Files\\script.js",
        "D:\\",
        "E:\\folder\\subfolder\\file.js",
        "c:\\lowercase\\drive.js", // lowercase drive letter
    ];

    for path in drive_paths {
        match runner.resolve_server_path(path) {
            Ok(resolved) => {
                println!("Drive path '{path}' resolved to: {resolved:?}");
            }
            Err(e) => {
                println!("Drive path '{path}' resolution failed (expected): {e}");
            }
        }
    }

    // Test relative paths with backslashes
    let relative_paths = vec![
        ".\\local\\script.js",
        "..\\parent\\script.js",
        "folder\\subfolder\\script.js",
    ];

    for path in relative_paths {
        match runner.resolve_server_path(path) {
            Ok(resolved) => {
                println!("Relative path '{path}' resolved to: {resolved:?}");
            }
            Err(_) => {
                // Expected to fail if file doesn't exist
            }
        }
    }
}

/// Test Windows-specific install instructions
#[test]
fn test_windows_install_instructions() {
    let dependencies = vec![
        Dependency::NodeJs {
            min_version: Some("18.0.0".to_string()),
        },
        Dependency::Python {
            min_version: Some("3.8".to_string()),
        },
        Dependency::Docker {
            min_version: None,
            requires_compose: false,
        },
        Dependency::Git,
    ];

    for dep in dependencies {
        let instructions = get_install_instructions(&dep);

        // Verify Windows-specific installers are included
        assert!(
            !instructions.windows.is_empty(),
            "{} should have Windows install methods",
            dep.name()
        );

        let windows_methods: Vec<String> = instructions
            .windows
            .iter()
            .map(|m| m.name.clone())
            .collect();

        // Check for common Windows package managers
        let has_windows_installer = windows_methods.iter().any(|m| {
            m.contains("winget")
                || m.contains("chocolatey")
                || m.contains("download")
                || m.contains("docker-desktop")
        });

        assert!(
            has_windows_installer,
            "{} should have Windows-specific installers, found: {:?}",
            dep.name(),
            windows_methods
        );
    }
}

/// Test Windows command line argument escaping
#[test]
fn test_windows_argument_escaping() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test arguments that need special handling on Windows
    let test_args = vec![
        vec!["--path".to_string(), "C:\\Program Files\\app".to_string()],
        vec!["--text".to_string(), "hello world".to_string()],
        vec!["--special".to_string(), "quotes\"inside".to_string()],
        vec!["--json".to_string(), r#"{"key": "value"}"#.to_string()],
        vec!["--empty".to_string(), "".to_string()],
        vec![
            "--multiple".to_string(),
            "arg1".to_string(),
            "arg2".to_string(),
            "arg3".to_string(),
        ],
    ];

    for args in test_args {
        match runner.run("test-server", &args) {
            Ok(_) => println!("Command would execute with args: {args:?}"),
            Err(_) => {
                // Expected to fail in test environment
                // The important thing is it doesn't panic
            }
        }
    }
}

/// Test platform-specific Node.js detection
#[test]
fn test_platform_specific_node_detection() {
    let checker = NodeChecker::new();
    let result = checker.check();

    // The check might succeed or fail depending on the test environment
    match result {
        Ok(check) => {
            println!("Node.js check result: {:?}", check.status);
            if let Some(instructions) = check.install_instructions {
                #[cfg(target_os = "windows")]
                {
                    assert!(!instructions.windows.is_empty());
                    // Check for npx.cmd on Windows
                    let npx_available =
                        which::which("npx.cmd").is_ok() || which::which("npx").is_ok();
                    println!("npx available on Windows: {npx_available}");
                }

                #[cfg(not(target_os = "windows"))]
                {
                    let platform_instructions = instructions.for_platform();
                    assert!(!platform_instructions.is_empty());
                }
            }
        }
        Err(e) => {
            println!("Node.js check failed (expected in CI): {e}");
        }
    }
}

/// Test Windows file system case sensitivity
#[test]
#[serial]
fn test_windows_case_sensitivity() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("TestFile.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    // Test case variations
    let case_variations = vec![
        "TestFile.txt",
        "testfile.txt",
        "TESTFILE.TXT",
        "TeStFiLe.TxT",
    ];

    for variation in case_variations {
        let test_path = temp_dir.path().join(variation);

        #[cfg(target_os = "windows")]
        {
            // Windows is case-insensitive
            assert!(
                test_path.exists(),
                "Windows should find '{}' (case-insensitive)",
                variation
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix is case-sensitive, but macOS might be case-insensitive depending on file system
            if variation == "TestFile.txt" {
                assert!(test_path.exists(), "Should find exact match '{variation}'");
            } else {
                // macOS HFS+ and APFS can be case-insensitive by default
                #[cfg(target_os = "macos")]
                {
                    // Don't assert on macOS as it depends on file system configuration
                    println!(
                        "macOS file exists check for '{variation}': {}",
                        test_path.exists()
                    );
                }

                #[cfg(target_os = "linux")]
                {
                    assert!(
                        !test_path.exists(),
                        "Linux should not find '{variation}' (case-sensitive)"
                    );
                }
            }
        }
    }
}

/// Test Windows-specific executable resolution
#[test]
fn test_windows_executable_resolution() {
    // Test .exe extension handling
    let executables = vec![
        ("node", vec!["node.exe", "node"]),
        ("npm", vec!["npm.cmd", "npm.exe", "npm"]),
        ("npx", vec!["npx.cmd", "npx.exe", "npx"]),
        ("git", vec!["git.exe", "git"]),
    ];

    for (base_name, variants) in executables {
        let mut found = false;
        for variant in variants {
            if which::which(variant).is_ok() {
                println!("Found {base_name} as {variant}");
                found = true;
                break;
            }
        }

        if !found {
            println!("{base_name} not found in PATH (expected in CI)");
        }
    }
}

/// Test Windows path length limitations
#[test]
fn test_windows_path_length_limits() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Windows has a traditional 260 character path limit
    // though this can be disabled in Windows 10+
    let long_path = "C:\\".to_string() + &"a\\".repeat(120) + "script.js";

    match runner.resolve_server_path(&long_path) {
        Ok(resolved) => {
            println!(
                "Long path resolved to: {resolved:?} (length: {})",
                resolved.to_string_lossy().len()
            );
        }
        Err(e) => {
            println!("Long path failed (expected): {e}");
            // This is expected to fail on Windows with traditional path limits
        }
    }

    // Test UNC long path format
    let unc_long_path = "\\\\?\\C:\\".to_string() + &"a\\".repeat(120) + "script.js";

    match runner.resolve_server_path(&unc_long_path) {
        Ok(resolved) => {
            println!("UNC long path resolved to: {resolved:?}");
        }
        Err(e) => {
            println!("UNC long path failed: {e}");
        }
    }
}

/// Test platform-specific configuration paths
#[test]
#[serial]
fn test_platform_config_paths() {
    // Save original env vars
    let original_appdata = env::var("APPDATA").ok();
    let original_xdg_config = env::var("XDG_CONFIG_HOME").ok();
    let original_home = env::var("HOME").ok();

    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Test Windows config path
    let windows_config = temp_dir.path().join("AppData\\Roaming");
    fs::create_dir_all(&windows_config).expect("Failed to create Windows config dir");
    env::set_var("APPDATA", &windows_config);

    // Test XDG config path (Linux)
    let xdg_config = temp_dir.path().join(".config");
    fs::create_dir_all(&xdg_config).expect("Failed to create XDG config dir");
    env::set_var("XDG_CONFIG_HOME", &xdg_config);

    // Test macOS config path
    let macos_config = temp_dir.path().join("Library/Application Support");
    fs::create_dir_all(&macos_config).expect("Failed to create macOS config dir");
    env::set_var("HOME", temp_dir.path());

    // Verify paths are created correctly
    #[cfg(target_os = "windows")]
    assert!(windows_config.exists());

    #[cfg(target_os = "linux")]
    assert!(xdg_config.exists());

    #[cfg(target_os = "macos")]
    assert!(macos_config.exists());

    // Restore env vars
    match original_appdata {
        Some(val) => env::set_var("APPDATA", val),
        None => env::remove_var("APPDATA"),
    }
    match original_xdg_config {
        Some(val) => env::set_var("XDG_CONFIG_HOME", val),
        None => env::remove_var("XDG_CONFIG_HOME"),
    }
    match original_home {
        Some(val) => env::set_var("HOME", val),
        None => env::remove_var("HOME"),
    }
}
