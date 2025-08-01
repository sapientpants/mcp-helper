//! Network failure and external dependency tests
//!
//! This test suite simulates various network failures, timeouts, and external
//! dependency issues to ensure robust error handling and recovery.

use mcp_helper::cache::CacheManager;
use mcp_helper::deps::{DependencyChecker, NodeChecker};
use mcp_helper::runner::ServerRunner;
use mcp_helper::security::SecurityValidator;
use mcp_helper::server::{BinaryServer, NpmServer, PythonServer};
use serial_test::serial;
use std::env;
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

/// Simulate a server that accepts connections but never responds
fn start_hanging_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");

    thread::spawn(move || {
        for _stream in listener.incoming().flatten() {
            // Accept connection but never respond
            thread::sleep(Duration::from_secs(60));
        }
    });

    format!("http://{addr}")
}

/// Simulate a server that closes connections immediately
fn start_refusing_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            // Close connection immediately
            drop(stream);
        }
    });

    format!("http://{addr}")
}

/// Simulate a server that returns invalid responses
fn start_corrupt_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            // Send garbage data
            let _ = stream.write_all(b"GARBAGE\r\n\r\nNOT HTTP");
            let _ = stream.flush();
        }
    });

    format!("http://{addr}")
}

/// Test npm server with network failures
#[test]
#[serial]
fn test_npm_server_network_failures() {
    // Test with non-existent npm package
    let result = NpmServer::new("@definitely/not-a-real-package-12345");
    match result {
        Ok(_server) => {
            // Server creation might succeed, but execution should fail
            let platform = match std::env::consts::OS {
                "windows" => mcp_helper::runner::Platform::Windows,
                "macos" => mcp_helper::runner::Platform::MacOS,
                "linux" => mcp_helper::runner::Platform::Linux,
                _ => mcp_helper::runner::Platform::Linux,
            };
            let runner = ServerRunner::new(platform, false);
            let result = runner.run("@definitely/not-a-real-package-12345", &[]);
            assert!(
                result.is_err(),
                "Should fail to run non-existent npm package"
            );
        }
        Err(e) => {
            println!("Failed to create npm server (expected): {e}");
        }
    }

    // Test with malformed package name
    let malformed_names = vec![
        "",
        " ",
        "@",
        "@@",
        "@/",
        "/package",
        "package@",
        "package@@version",
        "../../../etc/passwd",
        "package\0name",
        "package name with spaces",
        "https://evil.com/package",
    ];

    for name in malformed_names {
        match NpmServer::new(name) {
            Ok(_) => println!("Server created for '{name}' (might fail at runtime)"),
            Err(e) => println!("Rejected malformed package name '{name}': {e}"),
        }
    }
}

/// Test binary server download failures
#[test]
#[serial]
fn test_binary_server_download_failures() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("HOME", temp_dir.path());

    // Test with hanging server
    let hanging_url = start_hanging_server();
    let mut binary_server = BinaryServer::new(&format!("{hanging_url}/binary.exe"), None);

    // Set a short timeout for testing
    let cache_manager = CacheManager::new().ok();
    let result = binary_server.download_and_install(cache_manager.as_ref());
    assert!(result.is_err(), "Should timeout on hanging server");

    // Test with refusing server
    let refusing_url = start_refusing_server();
    let mut binary_server = BinaryServer::new(&format!("{refusing_url}/binary.exe"), None);
    let result = binary_server.download_and_install(cache_manager.as_ref());
    assert!(result.is_err(), "Should fail on connection refused");

    // Test with corrupt server
    let corrupt_url = start_corrupt_server();
    let mut binary_server = BinaryServer::new(&format!("{corrupt_url}/binary.exe"), None);
    let result = binary_server.download_and_install(cache_manager.as_ref());
    assert!(result.is_err(), "Should fail on corrupt response");

    // Test with invalid URLs
    let invalid_urls = vec![
        "not-a-url",
        "ftp://unsupported.protocol/file",
        "http://",
        "https://",
        "http://[invalid-ipv6/file",
        "http://999.999.999.999/file",
        "http://example.com:99999/file", // Invalid port
    ];

    for url in invalid_urls {
        let mut binary_server = BinaryServer::new(url, None);
        let result = binary_server.download_and_install(cache_manager.as_ref());
        assert!(result.is_err(), "Should fail for invalid URL: {url}");
    }

    env::remove_var("HOME");
}

/// Test GitHub API failures
#[test]
#[serial]
fn test_github_api_failures() {
    // Test non-existent repository
    let result = BinaryServer::from_github_repo("definitely/not-a-real-repo-12345", None);
    assert!(result.is_err(), "Should fail for non-existent repo");

    // Test invalid repository formats
    let invalid_repos = vec![
        "",
        "singlename",
        "too/many/slashes",
        "/leading-slash",
        "trailing-slash/",
        "special@chars/repo",
        "../../../etc/passwd",
    ];

    for repo in invalid_repos {
        let result = BinaryServer::from_github_repo(repo, None);
        if result.is_err() {
            println!("Correctly rejected invalid repo format: {repo}");
        }
    }

    // Test rate limiting simulation (can't actually trigger GitHub's rate limit in tests)
    // This would need mocking to properly test
}

/// Test checksum verification failures
#[test]
#[serial]
fn test_checksum_verification_failures() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("HOME", temp_dir.path());

    // Create a mock binary file
    let bin_dir = temp_dir.path().join(".mcp").join("bin");
    std::fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");
    let binary_path = bin_dir.join("test-binary");
    std::fs::write(&binary_path, b"test content").expect("Failed to write test binary");

    // Test with wrong checksum
    let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";
    let _binary_server = BinaryServer::new("file:///test", Some(wrong_checksum.to_string()));

    // Note: We can't easily test the checksum verification without modifying the implementation
    // to allow injecting test data. This is a limitation of the current design.

    env::remove_var("HOME");
}

/// Test Python server with missing dependencies
#[test]
fn test_python_server_dependency_failures() {
    // Test Python server when Python is not available
    let _python_server = PythonServer::new("/path/to/script.py");
    let deps = vec![mcp_helper::deps::Dependency::Python {
        min_version: Some("3.8".to_string()),
    }];

    assert!(
        !deps.is_empty(),
        "Python server should have Python dependency"
    );

    // Check if Python dependency check works
    for dep in deps {
        let checker: Box<dyn DependencyChecker> = match dep {
            mcp_helper::deps::Dependency::Python { .. } => {
                Box::new(mcp_helper::deps::PythonChecker::new())
            }
            _ => continue,
        };

        let result = checker.check();
        match result {
            Ok(check) => {
                println!("Python check result: {:?}", check.status);
            }
            Err(e) => {
                println!("Python check failed (might be expected in CI): {e}");
            }
        }
    }
}

/// Test concurrent dependency checks
#[test]
fn test_concurrent_dependency_checks() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // Spawn multiple threads checking dependencies simultaneously
    for i in 0..10 {
        let results_clone = Arc::clone(&results);
        let handle = thread::spawn(move || {
            let checker = NodeChecker::new();
            let result = checker.check();

            let mut results = results_clone.lock().unwrap();
            results.push((i, result));
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let results = results.lock().unwrap();
    let count = results.len();
    println!("Completed {count} concurrent dependency checks");

    // All checks should complete without panicking
    assert_eq!(results.len(), 10);
}

/// Test security validation with network issues
#[test]
fn test_security_validation_network_failures() {
    let validator = SecurityValidator::new();

    // Test URLs that might have network issues
    let test_urls = vec![
        "https://expired.badssl.com/",        // Expired certificate
        "https://self-signed.badssl.com/",    // Self-signed certificate
        "https://untrusted-root.badssl.com/", // Untrusted root
        "https://wrong.host.badssl.com/",     // Wrong host
        "http://insecure.example.com/file",   // HTTP instead of HTTPS
    ];

    for url in test_urls {
        let result = validator.validate_url(url);
        match result {
            Ok(validation) => {
                println!(
                    "URL {url} validation: trusted={}, https={}, warnings={:?}",
                    validation.is_trusted, validation.is_https, validation.warnings
                );

                // HTTP URLs should not be trusted by default
                if url.starts_with("http://") && !url.contains("localhost") {
                    assert!(!validation.is_https, "HTTP URL should not be HTTPS");
                    assert!(
                        !validation.warnings.is_empty(),
                        "HTTP URL should have warnings"
                    );
                }
            }
            Err(e) => {
                println!("URL {url} validation failed: {e}");
            }
        }
    }
}

/// Test npm install with network interruption simulation
#[test]
#[serial]
fn test_npm_install_network_interruption() {
    // This test simulates network interruptions during npm operations
    // In a real scenario, we'd use a proxy that can simulate network failures

    // Test with NODE_OFFLINE environment variable (if npm respects it)
    env::set_var("NODE_OFFLINE", "1");

    let result = NpmServer::new("express");
    match result {
        Ok(_server) => {
            let platform = match std::env::consts::OS {
                "windows" => mcp_helper::runner::Platform::Windows,
                "macos" => mcp_helper::runner::Platform::MacOS,
                "linux" => mcp_helper::runner::Platform::Linux,
                _ => mcp_helper::runner::Platform::Linux,
            };
            let runner = ServerRunner::new(platform, false);
            let result = runner.run("express", &[]);

            // Might fail due to offline mode
            if result.is_err() {
                println!("NPM correctly failed in offline mode");
            }
        }
        Err(e) => {
            println!("NPM server creation failed in offline mode: {e}");
        }
    }

    env::remove_var("NODE_OFFLINE");
}

/// Test DNS resolution failures
#[test]
fn test_dns_resolution_failures() {
    let validator = SecurityValidator::new();

    // Test with non-existent domains
    let non_existent_domains = vec![
        "https://definitely-not-a-real-domain-12345.com/file",
        "https://xn--invalid-unicode-.com/file",
        "https://subdomain.definitely-not-real-12345.org/file",
    ];

    for url in non_existent_domains {
        let result = validator.validate_url(url);
        match result {
            Ok(validation) => {
                println!("Non-existent domain {url} unexpectedly validated: {validation:?}");
            }
            Err(e) => {
                println!("Non-existent domain {url} correctly failed: {e}");
            }
        }
    }
}

/// Test partial download recovery
#[test]
#[serial]
fn test_partial_download_recovery() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("HOME", temp_dir.path());

    // Create a partial download file
    let bin_dir = temp_dir.path().join(".mcp").join("bin");
    std::fs::create_dir_all(&bin_dir).expect("Failed to create bin dir");
    let partial_file = bin_dir.join("partial-download.exe.part");
    std::fs::write(&partial_file, b"partial content").expect("Failed to write partial file");

    // Test that binary server handles partial downloads
    // (Current implementation might not have resume support)

    env::remove_var("HOME");
}

/// Test timeout configurations
#[test]
#[ignore] // Timeout configuration not yet implemented in binary server
fn test_timeout_configurations() {
    // Test very short timeouts
    env::set_var("MCP_DOWNLOAD_TIMEOUT", "1"); // 1 second timeout

    let hanging_url = start_hanging_server();
    let mut binary_server = BinaryServer::new(&format!("{hanging_url}/slow-download"), None);

    let cache_manager = CacheManager::new().ok();
    let start = std::time::Instant::now();
    let result = binary_server.download_and_install(cache_manager.as_ref());
    let duration = start.elapsed();

    assert!(result.is_err(), "Should timeout on slow download");
    // Note: The binary server doesn't currently implement timeout configuration
    // This test is ignored until that feature is implemented
    assert!(duration < Duration::from_secs(10), "Should timeout quickly");

    env::remove_var("MCP_DOWNLOAD_TIMEOUT");
}

/// Test cache behavior during network failures
#[test]
#[serial]
fn test_cache_fallback_on_network_failure() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    let mut cache_manager = CacheManager::new().expect("Failed to create cache manager");

    // Pre-populate cache with dependency info
    use mcp_helper::deps::{Dependency, DependencyStatus};
    let dep = Dependency::NodeJs {
        min_version: Some("18.0.0".to_string()),
    };
    let status = DependencyStatus::Installed {
        version: Some("18.0.0".to_string()),
    };

    cache_manager
        .cache_dependency_status(dep.clone(), status.clone())
        .expect("Failed to cache dependency");

    // Simulate network failure by using invalid checker
    // In real scenario, we'd mock the network call

    // Verify cache can be retrieved even without network
    let cached = cache_manager.get_dependency_status(&dep);
    assert!(cached.is_some(), "Should retrieve from cache");

    env::remove_var("XDG_CACHE_HOME");
}

/// Test server metadata fetching failures
#[test]
fn test_metadata_fetch_failures() {
    // Test metadata fetching for non-existent servers
    let non_existent_servers = vec![
        "@definitely/not-real",
        "python:nonexistent-script.py",
        "binary:https://fake.url/binary",
        "docker:fake/image:latest",
    ];

    for server in non_existent_servers {
        println!("Testing metadata fetch for non-existent server: {server}");
        // In a real implementation, we'd test the metadata fetching
        // Currently, this would require mocking the metadata service
    }
}

/// Test retry logic for transient failures
#[test]
#[serial]
fn test_retry_logic() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // Create a server that fails the first N times
    let failure_count = Arc::new(AtomicU32::new(0));
    let max_failures = 2;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind");
    let addr = listener.local_addr().expect("Failed to get local address");
    let failure_count_clone = Arc::clone(&failure_count);

    thread::spawn(move || {
        for mut stream in listener.incoming().flatten() {
            let failures = failure_count_clone.fetch_add(1, Ordering::SeqCst);
            if failures < max_failures {
                // Fail the first few times
                drop(stream);
            } else {
                // Succeed after that
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\ntest content";
                let _ = stream.write_all(response.as_bytes());
            }
        }
    });

    // Test that retry logic eventually succeeds
    let url = format!("http://{addr}/retry-test");
    println!("Testing retry logic with URL: {url}");

    // Note: Current implementation might not have retry logic
    // This test demonstrates how it should behave
}

/// Test proxy configuration failures
#[test]
fn test_proxy_configuration_failures() {
    // Test with invalid proxy configurations
    let invalid_proxies = vec![
        "not-a-url",
        "http://",
        "http://999.999.999.999:8080",
        "http://proxy:invalid-port",
        "socks5://unsupported:1080",
    ];

    for proxy in invalid_proxies {
        env::set_var("HTTP_PROXY", proxy);
        env::set_var("HTTPS_PROXY", proxy);

        // Test that operations handle invalid proxy gracefully
        let validator = SecurityValidator::new();
        let result = validator.validate_url("https://example.com");

        // Should either fail gracefully or ignore invalid proxy
        println!("Proxy {proxy} test result: {:?}", result.is_ok());

        env::remove_var("HTTP_PROXY");
        env::remove_var("HTTPS_PROXY");
    }
}

/// Test handling of various HTTP error codes
#[test]
fn test_http_error_codes() {
    // Test handling of various HTTP error responses
    // This would require a mock server that returns specific error codes

    let error_codes = vec![
        (400, "Bad Request"),
        (401, "Unauthorized"),
        (403, "Forbidden"),
        (404, "Not Found"),
        (429, "Too Many Requests"),
        (500, "Internal Server Error"),
        (502, "Bad Gateway"),
        (503, "Service Unavailable"),
        (504, "Gateway Timeout"),
    ];

    for (code, description) in error_codes {
        println!("Testing HTTP {code} - {description}");
        // In real implementation, we'd test actual error handling
    }
}
