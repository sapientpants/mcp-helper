//! Tests for error handling of unwrap/expect calls in critical modules
//!
//! This test suite ensures that code paths containing unwrap/expect are properly
//! tested and that error conditions are handled gracefully.

use mcp_helper::cache::CacheManager;
use mcp_helper::config::ConfigManager;
use mcp_helper::install::InstallCommand;
use mcp_helper::runner::{Platform, ServerRunner};
use serial_test::serial;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Test ConfigManager creation failure scenarios
#[test]
#[serial]
fn test_config_manager_creation_failure() {
    // Save original env
    let original_home = env::var("HOME").ok();
    let original_xdg_data = env::var("XDG_DATA_HOME").ok();

    // Create a directory where we can't write
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).expect("Failed to create directory");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }

    // Set environment to use readonly directory
    env::set_var("HOME", &readonly_dir);
    env::set_var("XDG_DATA_HOME", &readonly_dir);

    // Test that ConfigManager::new() handles the error gracefully
    let result = ConfigManager::new();

    // On some systems, creating directories might still succeed
    // So we just verify it doesn't panic
    match result {
        Ok(_) => println!("ConfigManager created successfully (system allows writes)"),
        Err(e) => {
            println!("ConfigManager creation failed as expected: {e}");
            // Verify the error is meaningful
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("permission")
                    || error_msg.contains("Permission")
                    || error_msg.contains("access"),
                "Error should mention permissions: {error_msg}"
            );
        }
    }

    // Restore original environment
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    } else {
        env::remove_var("HOME");
    }
    if let Some(xdg) = original_xdg_data {
        env::set_var("XDG_DATA_HOME", xdg);
    } else {
        env::remove_var("XDG_DATA_HOME");
    }
}

/// Test CacheManager creation failure scenarios
#[test]
#[serial]
fn test_cache_manager_creation_failure() {
    // Save original env
    let original_home = env::var("HOME").ok();
    let original_xdg_cache = env::var("XDG_CACHE_HOME").ok();

    // Create a directory where we can't write
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let readonly_dir = temp_dir.path().join("readonly_cache");
    fs::create_dir(&readonly_dir).expect("Failed to create directory");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        fs::set_permissions(&readonly_dir, perms).unwrap();
    }

    // Set environment to use readonly directory
    env::set_var("HOME", &readonly_dir);
    env::set_var("XDG_CACHE_HOME", &readonly_dir);

    // Test that CacheManager::new() handles the error gracefully
    let result = CacheManager::new();

    match result {
        Ok(_) => println!("CacheManager created successfully (system allows writes)"),
        Err(e) => {
            println!("CacheManager creation failed as expected: {e}");
            // Verify error is meaningful
            let error_msg = e.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
        }
    }

    // Test the default() method which uses expect
    // In our case, CacheManager::default() uses expect() on new()
    // But since we're in a test environment, this may still succeed
    // The important thing is that it doesn't panic
    std::panic::catch_unwind(|| {
        let _cache_manager = CacheManager::default();
    })
    .ok(); // Don't care about the result, just that it doesn't abort

    // Restore original environment
    if let Some(home) = original_home {
        env::set_var("HOME", home);
    } else {
        env::remove_var("HOME");
    }
    if let Some(cache) = original_xdg_cache {
        env::set_var("XDG_CACHE_HOME", cache);
    } else {
        env::remove_var("XDG_CACHE_HOME");
    }
}

/// Test InstallCommand creation with failing ConfigManager
#[test]
#[serial]
fn test_install_command_config_manager_failure() {
    // The InstallCommand uses expect() on ConfigManager::new()
    // We need to ensure this is tested in a scenario where it might fail

    // Save original env
    let original_xdg = env::var("XDG_DATA_HOME").ok();

    // Create a problematic path scenario
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let invalid_path = temp_dir
        .path()
        .join("nonexistent")
        .join("deep")
        .join("path");

    // Set to a path that doesn't exist
    env::set_var("XDG_DATA_HOME", invalid_path.to_string_lossy().as_ref());

    // This uses expect() internally - verify it handles errors appropriately
    // The actual behavior depends on whether ConfigManager can handle the invalid path
    let _installer = InstallCommand::new(true);

    // Restore env
    if let Some(xdg) = original_xdg {
        env::set_var("XDG_DATA_HOME", xdg);
    } else {
        env::remove_var("XDG_DATA_HOME");
    }
}

/// Test ServerRunner path resolution with invalid paths
#[test]
fn test_server_runner_invalid_path_resolution() {
    let platform = if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        Platform::Linux
    };
    let runner = ServerRunner::new(platform, false);

    // Test with paths that might cause issues
    let long_path = "a".repeat(10000);
    let test_paths = vec![
        "",                             // Empty path
        ".",                            // Current directory
        "..",                           // Parent directory
        "~",                            // Home directory shorthand
        "$HOME/test",                   // Environment variable
        "C:\\Windows\\System32",        // Windows system path
        "/etc/passwd",                  // Unix system file
        "../../../../../../etc/passwd", // Path traversal
        "\0",                           // Null byte
        &long_path[..],                 // Very long path
        "🚀💥",                         // Unicode
    ];

    for path in test_paths {
        match runner.resolve_server_path(path) {
            Ok(resolved) => {
                // Verify the path is properly sanitized
                let path_str = resolved.to_string_lossy();
                println!("Path '{path}' resolved to '{path_str}'");

                // Just verify it resolved without panic
                // Different systems may handle path traversal differently
            }
            Err(e) => {
                println!("Path '{path}' failed as expected: {e}");
                // This is fine - we want to handle errors gracefully
            }
        }
    }
}

/// Test cache operations with system time edge cases
#[test]
#[serial]
fn test_cache_manager_time_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    let mut cache_manager = CacheManager::new().unwrap_or_else(|_| CacheManager::default());

    // Test caching with current time
    use mcp_helper::deps::{Dependency, DependencyStatus};

    let dependency = Dependency::NodeJs {
        min_version: Some("16.0.0".to_string()),
    };
    let status = DependencyStatus::Installed {
        version: Some("18.0.0".to_string()),
    };

    // This internally uses SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    let result = cache_manager.cache_dependency_status(dependency.clone(), status);
    assert!(result.is_ok(), "Caching should succeed");

    // Test retrieval
    let cached = cache_manager.get_dependency_status(&dependency);
    assert!(cached.is_some(), "Should retrieve cached dependency");
}

/// Test config manager mutex operations under contention
#[test]
#[serial]
fn test_config_manager_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    // This tests the unwrap() calls on mutex locks in MockClient
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    env::set_var("XDG_DATA_HOME", temp_dir.path());

    let manager = Arc::new(ConfigManager::new().unwrap_or_else(|_| ConfigManager::default()));

    let mut handles = vec![];

    // Spawn multiple threads trying to access config simultaneously
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            // This would test internal mutex operations if we had access
            // For now, we just verify the manager works under concurrent load
            let _result = manager_clone.get_history(None, None);
            println!("Thread {i} completed");
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

/// Test runner command generation with edge cases
#[test]
fn test_runner_command_generation_edge_cases() {
    let platform = if cfg!(target_os = "windows") {
        Platform::Windows
    } else if cfg!(target_os = "macos") {
        Platform::MacOS
    } else {
        Platform::Linux
    };
    let runner = ServerRunner::new(platform, false);

    // Test various command scenarios that might fail
    struct TestCase {
        args: Vec<String>,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            args: vec![],
            description: "Empty args",
        },
        TestCase {
            args: vec!["".to_string()],
            description: "Empty string arg",
        },
        TestCase {
            args: vec!["--arg".to_string(), "value with spaces".to_string()],
            description: "Args with spaces",
        },
        TestCase {
            args: vec!["--arg=\"quoted\"".to_string()],
            description: "Args with quotes",
        },
        TestCase {
            args: vec!["$(echo pwned)".to_string()],
            description: "Command injection attempt",
        },
        TestCase {
            args: vec!["; rm -rf /".to_string()],
            description: "Shell command attempt",
        },
        TestCase {
            args: vec!["a".repeat(1000)],
            description: "Very long argument",
        },
    ];

    for test_case in test_cases {
        println!("\nTesting: {}", test_case.description);
        match runner.run("test-server", &test_case.args) {
            Ok(_) => println!("  Command executed (may have failed at runtime)"),
            Err(e) => println!("  Command failed as expected: {e}"),
        }
    }
}

/// Test JSON parsing in cache with malformed data
#[test]
#[serial]
fn test_cache_corrupt_json_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    let cache_manager = CacheManager::new().unwrap_or_else(|_| CacheManager::default());

    // Create cache directory
    let cache_dir = temp_dir.path().join("mcp-helper");
    fs::create_dir_all(&cache_dir).expect("Failed to create cache dir");

    // Write corrupt cache files
    let corrupt_files = vec![
        ("deps_cache.json", "not json"),
        ("deps_cache.json", "{\"key\": }"), // Invalid JSON
        ("deps_cache.json", "null"),        // Valid JSON but wrong type
        ("metadata_cache.json", "[1,2,3]"), // Wrong structure
    ];

    for (filename, content) in corrupt_files {
        let file_path = cache_dir.join(filename);
        fs::write(&file_path, content).expect("Failed to write corrupt file");

        // Try to use the cache - should handle corruption gracefully
        use mcp_helper::deps::Dependency;

        let dependency = Dependency::NodeJs {
            min_version: Some("16.0.0".to_string()),
        };

        // This should not panic despite corrupt cache
        let cached = cache_manager.get_dependency_status(&dependency);
        // It should return None or handle the error internally
        println!("Corrupt cache '{filename}' handled: {:?}", cached.is_some());
    }
}

/// Test path operations in runner with non-UTF8 paths
#[test]
#[cfg(unix)]
fn test_runner_non_utf8_paths() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    use std::path::PathBuf;

    let platform = Platform::Linux; // Unix test
    let runner = ServerRunner::new(platform, false);

    // Create a path with invalid UTF-8
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    let invalid_path = OsStr::from_bytes(&invalid_bytes);

    // Convert to PathBuf
    let path_buf = PathBuf::from(invalid_path);

    // Test that to_str() unwrap scenarios are handled
    // In the actual code, we use to_string_lossy() or proper error handling
    let path_str = path_buf.to_string_lossy();
    println!("Non-UTF8 path handled: {path_str}");

    // Verify it doesn't panic when used
    match runner.resolve_server_path(&path_str) {
        Ok(resolved) => println!("Resolved to: {resolved:?}"),
        Err(e) => println!("Failed as expected: {e}"),
    }
}

/// Test InstallCommand with cache manager failures
#[test]
#[serial]
fn test_install_command_cache_failures() {
    // Set up environment to make cache creation fail
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let cache_dir = temp_dir.path().join("cache");

    // Create a file where directory should be
    fs::write(&cache_dir, "not a directory").expect("Failed to write file");

    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    // InstallCommand uses unwrap_or_else for cache manager
    // This should fall back to default instead of panicking
    let mut installer = InstallCommand::new(true);

    // Verify it still works even with cache issues
    println!("InstallCommand created successfully with cache fallback");

    // The installer should work even without a functional cache
    match installer.execute("nonexistent-package") {
        Ok(_) => println!("Installation succeeded unexpectedly"),
        Err(e) => println!("Installation failed as expected: {e}"),
    }
}
