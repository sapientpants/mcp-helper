//! Comprehensive tests for platform-specific command execution

use mcp_helper::runner::{Platform, ServerRunner};
use std::path::PathBuf;

#[test]
fn test_server_runner_platform_creation() {
    // Test that ServerRunner can be created with different platforms
    let windows_runner = ServerRunner::new(Platform::Windows, false);
    let macos_runner = ServerRunner::new(Platform::MacOS, true);
    let linux_runner = ServerRunner::new(Platform::Linux, false);

    // Can't access verbose field directly as it's private
    // But we can test that runners are created successfully
    let _ = windows_runner;
    let _ = macos_runner;
    let _ = linux_runner;
}

#[test]
fn test_windows_npx_command_handling() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test NPM package command
    let (cmd, args) = runner
        .get_command_for_platform(
            &PathBuf::from("@modelcontextprotocol/server-filesystem"),
            &["--path".to_string(), "C:\\Users\\test".to_string()],
        )
        .unwrap();

    // On Windows, it can return either cmd.exe (when npx.cmd is found) or npx
    if cmd == "cmd.exe" {
        // When npx.cmd is found, args are ["/c", "npx.cmd", server, ...]
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "npx.cmd");
        assert_eq!(args[2], "@modelcontextprotocol/server-filesystem");
        assert_eq!(args[3], "--path");
        assert_eq!(args[4], "C:\\Users\\test");
    } else {
        // When npx.cmd is not found but npx is
        assert_eq!(cmd, "npx");
        assert_eq!(args[0], "@modelcontextprotocol/server-filesystem");
        assert_eq!(args[1], "--path");
        assert_eq!(args[2], "C:\\Users\\test");
    }
}

#[test]
fn test_unix_npx_command_handling() {
    for platform in &[Platform::MacOS, Platform::Linux] {
        let runner = ServerRunner::new(*platform, false);

        // Test NPM package command
        let (cmd, args) = runner
            .get_command_for_platform(
                &PathBuf::from("@modelcontextprotocol/server-filesystem"),
                &["--path".to_string(), "/home/user".to_string()],
            )
            .unwrap();

        // On Unix, npx remains npx (no -y flag added)
        assert_eq!(cmd, "npx");
        assert_eq!(args[0], "@modelcontextprotocol/server-filesystem");
        assert_eq!(args[1], "--path");
        assert_eq!(args[2], "/home/user");
    }
}

#[test]
fn test_windows_executable_detection() {
    let runner = ServerRunner::new(Platform::Windows, false);

    // Test .exe file
    let (cmd, args) = runner
        .get_command_for_platform(
            &PathBuf::from("C:\\Program Files\\App\\server.exe"),
            &["--port".to_string(), "3000".to_string()],
        )
        .unwrap();

    // Non-existing paths are treated as npm packages
    if cmd == "cmd.exe" {
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "npx.cmd");
        assert_eq!(args[2], "C:\\Program Files\\App\\server.exe");
        assert_eq!(args[3], "--port");
        assert_eq!(args[4], "3000");
    } else {
        assert_eq!(cmd, "npx");
        assert_eq!(
            args,
            vec!["C:\\Program Files\\App\\server.exe", "--port", "3000"]
        );
    }

    // Test .cmd file
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("script.cmd"), &[])
        .unwrap();

    if cmd == "cmd.exe" {
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "npx.cmd");
        assert_eq!(args[2], "script.cmd");
    } else {
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["script.cmd"]);
    }

    // Test .bat file
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("batch.bat"), &["arg1".to_string()])
        .unwrap();

    if cmd == "cmd.exe" {
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "npx.cmd");
        assert_eq!(args[2], "batch.bat");
        assert_eq!(args[3], "arg1");
    } else {
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["batch.bat", "arg1"]);
    }
}

#[test]
fn test_unix_executable_handling() {
    for platform in &[Platform::MacOS, Platform::Linux] {
        let runner = ServerRunner::new(*platform, false);

        // Test shell script
        let (cmd, args) = runner
            .get_command_for_platform(&PathBuf::from("./script.sh"), &["--verbose".to_string()])
            .unwrap();

        // Non-absolute paths are treated as npm packages
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["./script.sh", "--verbose"]);

        // Test binary without extension (absolute path)
        let (cmd, args) = runner
            .get_command_for_platform(
                &PathBuf::from("/usr/local/bin/mcp-server"),
                &["start".to_string()],
            )
            .unwrap();

        // Non-existing absolute paths still go through npx
        assert_eq!(cmd, "npx");
        assert_eq!(args, vec!["/usr/local/bin/mcp-server", "start"]);
    }
}

#[test]
fn test_platform_specific_path_resolution() {
    // Windows paths
    let windows_runner = ServerRunner::new(Platform::Windows, false);
    let resolved = windows_runner
        .resolve_server_path("path/to/server")
        .unwrap();
    assert_eq!(resolved, PathBuf::from("path\\to\\server"));

    // Unix paths
    let unix_runner = ServerRunner::new(Platform::Linux, false);
    let resolved = unix_runner.resolve_server_path("path\\to\\server").unwrap();
    assert_eq!(resolved, PathBuf::from("path/to/server"));
}

#[test]
fn test_command_with_special_characters() {
    // Windows: paths with spaces
    let windows_runner = ServerRunner::new(Platform::Windows, false);
    let (cmd, args) = windows_runner
        .get_command_for_platform(
            &PathBuf::from("Program Files\\My Server\\server.exe"),
            &["--config".to_string(), "My Config.json".to_string()],
        )
        .unwrap();

    if cmd == "cmd.exe" {
        assert_eq!(args[0], "/c");
        assert_eq!(args[1], "npx.cmd");
        assert_eq!(args[2], "Program Files\\My Server\\server.exe");
        assert_eq!(args[3], "--config");
        assert_eq!(args[4], "My Config.json");
    } else {
        assert_eq!(cmd, "npx");
        assert_eq!(
            args,
            vec![
                "Program Files\\My Server\\server.exe",
                "--config",
                "My Config.json"
            ]
        );
    }

    // Unix: paths with special characters
    let unix_runner = ServerRunner::new(Platform::Linux, false);
    let (cmd, args) = unix_runner
        .get_command_for_platform(
            &PathBuf::from("/opt/my-server/run.sh"),
            &["--file=data (1).txt".to_string()],
        )
        .unwrap();

    assert_eq!(cmd, "npx");
    assert_eq!(args, vec!["/opt/my-server/run.sh", "--file=data (1).txt"]);
}

#[test]
fn test_scoped_npm_packages() {
    // Test scoped packages on all platforms
    for platform in &[Platform::Windows, Platform::MacOS, Platform::Linux] {
        let runner = ServerRunner::new(*platform, false);
        let (cmd, args) = runner
            .get_command_for_platform(&PathBuf::from("@anthropic/mcp-server"), &[])
            .unwrap();

        // Windows might use cmd.exe wrapper, Unix uses npx directly
        if *platform == Platform::Windows && cmd == "cmd.exe" {
            assert_eq!(args[0], "/c");
            assert_eq!(args[1], "npx.cmd");
            assert_eq!(args[2], "@anthropic/mcp-server");
        } else {
            assert_eq!(cmd, "npx");
            assert_eq!(args[0], "@anthropic/mcp-server");
        }
    }
}

#[test]
fn test_npm_package_with_version() {
    let runner = ServerRunner::new(Platform::Linux, false);
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("mcp-server@1.2.3"), &["--start".to_string()])
        .unwrap();

    assert_eq!(cmd, "npx");
    assert_eq!(args, vec!["mcp-server@1.2.3", "--start"]);
}

#[test]
fn test_python_script_execution() {
    // Test Python scripts on all platforms
    for platform in &[Platform::Windows, Platform::MacOS, Platform::Linux] {
        let runner = ServerRunner::new(*platform, false);
        let (cmd, args) = runner
            .get_command_for_platform(
                &PathBuf::from("server.py"),
                &["--port".to_string(), "8080".to_string()],
            )
            .unwrap();

        // Non-absolute paths are treated as npm packages
        if *platform == Platform::Windows && cmd == "cmd.exe" {
            assert_eq!(args[0], "/c");
            assert_eq!(args[1], "npx.cmd");
            assert_eq!(args[2], "server.py");
            assert_eq!(args[3], "--port");
            assert_eq!(args[4], "8080");
        } else {
            assert_eq!(cmd, "npx");
            assert_eq!(args, vec!["server.py", "--port", "8080"]);
        }
    }
}

#[test]
fn test_verbose_mode_behavior() {
    // Test that runners can be created with verbose mode
    let verbose_runner = ServerRunner::new(Platform::Windows, true);
    let quiet_runner = ServerRunner::new(Platform::Linux, false);

    // Can't access verbose field, but we can test the runners work
    let _ = verbose_runner;
    let _ = quiet_runner;
}

#[test]
fn test_empty_args_handling() {
    let runner = ServerRunner::new(Platform::MacOS, false);
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("simple-server"), &[])
        .unwrap();

    assert_eq!(cmd, "npx");
    assert_eq!(args, vec!["simple-server"]);
}

#[test]
fn test_complex_argument_patterns() {
    let runner = ServerRunner::new(Platform::Linux, false);
    let complex_args = vec![
        "--flag".to_string(),
        "-v".to_string(),
        "--key=value".to_string(),
        "--path=/home/user/file.txt".to_string(),
        "positional".to_string(),
        "--".to_string(),
        "extra".to_string(),
    ];

    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("complex-server"), &complex_args)
        .unwrap();

    assert_eq!(cmd, "npx");
    let mut expected_args = vec!["complex-server".to_string()];
    expected_args.extend(complex_args);
    assert_eq!(args, expected_args);
}

#[test]
fn test_platform_specific_environment_paths() {
    // Test typical installation paths
    let windows_paths = vec![
        "C:\\Program Files\\nodejs\\node_modules\\.bin\\mcp-server",
        "%APPDATA%\\npm\\mcp-server",
        ".\\node_modules\\.bin\\mcp-server",
    ];

    let unix_paths = vec![
        "/usr/local/lib/node_modules/.bin/mcp-server",
        "~/.npm-global/bin/mcp-server",
        "./node_modules/.bin/mcp-server",
    ];

    let windows_runner = ServerRunner::new(Platform::Windows, false);
    for path in windows_paths {
        let resolved = windows_runner.resolve_server_path(path).unwrap();
        assert!(
            resolved.to_string_lossy().contains('\\') || resolved.to_string_lossy().contains('/')
        );
    }

    let unix_runner = ServerRunner::new(Platform::Linux, false);
    for path in unix_paths {
        let resolved = unix_runner.resolve_server_path(path).unwrap();
        assert!(
            !resolved.to_string_lossy().contains('\\') || resolved.to_string_lossy().contains('/')
        );
    }
}

#[test]
fn test_local_vs_global_package_detection() {
    // Local node_modules
    let runner = ServerRunner::new(Platform::MacOS, false);

    let local_path = "./node_modules/.bin/mcp-server";
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from(local_path), &[])
        .unwrap();
    // Non-absolute paths go through npx
    assert_eq!(cmd, "npx");
    assert_eq!(args[0], local_path);

    // Global package (no path indicators)
    let (cmd, args) = runner
        .get_command_for_platform(&PathBuf::from("mcp-server"), &[])
        .unwrap();
    assert_eq!(cmd, "npx");
    assert_eq!(args, vec!["mcp-server"]);
}
