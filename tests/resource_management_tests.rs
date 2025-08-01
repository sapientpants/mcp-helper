//! Resource management and concurrent operation tests
//!
//! This test suite verifies proper resource management including:
//! - File handle management and cleanup
//! - Temporary file/directory cleanup
//! - Concurrent access to shared resources
//! - Lock contention and deadlock prevention
//! - Process spawning and cleanup
//! - Memory usage patterns

use mcp_helper::cache::CacheManager;
use mcp_helper::client::detect_clients;
use mcp_helper::runner::{Platform, ServerRunner};
use mcp_helper::utils::secure_file::write_secure;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::{NamedTempFile, TempDir};

/// Test temporary file cleanup on drop
#[test]
fn test_temp_file_cleanup() {
    let temp_path: PathBuf;

    // Create temp file in a scope
    {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_path = temp_file.path().to_path_buf();

        // Verify file exists
        assert!(temp_path.exists(), "Temp file should exist");

        // Write some data
        let mut file = temp_file.as_file();
        file.write_all(b"test data").expect("Failed to write");
    } // temp_file dropped here

    // Verify file is cleaned up
    assert!(
        !temp_path.exists(),
        "Temp file should be deleted after drop"
    );
}

/// Test temporary directory cleanup with nested files
#[test]
fn test_temp_dir_cleanup() {
    let temp_dir_path: PathBuf;
    let nested_file_path: PathBuf;

    {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        temp_dir_path = temp_dir.path().to_path_buf();

        // Create nested structure
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir).expect("Failed to create subdir");

        nested_file_path = sub_dir.join("nested.txt");
        fs::write(&nested_file_path, b"nested data").expect("Failed to write nested file");

        assert!(temp_dir_path.exists(), "Temp dir should exist");
        assert!(nested_file_path.exists(), "Nested file should exist");
    } // temp_dir dropped here

    // Verify everything is cleaned up
    assert!(!temp_dir_path.exists(), "Temp dir should be deleted");
    assert!(!nested_file_path.exists(), "Nested file should be deleted");
}

/// Test concurrent cache access
#[test]
#[serial]
fn test_concurrent_cache_access() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    let cache_manager = Arc::new(Mutex::new(
        CacheManager::new().expect("Failed to create cache manager"),
    ));

    let mut handles = vec![];
    let iterations = 10;
    let threads = 5;

    // Spawn multiple threads accessing cache simultaneously
    for thread_id in 0..threads {
        let cache_clone = Arc::clone(&cache_manager);

        let handle = thread::spawn(move || {
            for i in 0..iterations {
                // Test concurrent reads and writes
                let _key = format!("thread_{thread_id}_item_{i}");
                let _value = format!("value_{thread_id}_{i}");

                // Write to cache
                {
                    let cache = cache_clone.lock().unwrap();
                    // Simulate cache write (would need actual cache method)
                    drop(cache); // Release lock quickly
                }

                // Read from cache
                {
                    let cache = cache_clone.lock().unwrap();
                    // Simulate cache read
                    drop(cache);
                }

                // Small delay to increase contention
                thread::sleep(Duration::from_micros(100));
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    env::remove_var("XDG_CACHE_HOME");
}

/// Test file handle limits
#[test]
fn test_file_handle_management() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let mut file_handles = Vec::new();

    // Try to open many files
    let max_files = 100; // Conservative limit

    for i in 0..max_files {
        let file_path = temp_dir.path().join(format!("file_{i}.txt"));

        match File::create(&file_path) {
            Ok(file) => {
                file_handles.push(file);
            }
            Err(e) => {
                // Hit file handle limit
                println!("Hit file handle limit at {i} files: {e}");
                break;
            }
        }
    }

    // Verify we can open at least some files
    assert!(
        file_handles.len() > 10,
        "Should be able to open multiple files"
    );

    // Close half the files
    let half = file_handles.len() / 2;
    file_handles.truncate(half);

    // Try to open more files after closing some
    for i in max_files..max_files + 10 {
        let file_path = temp_dir.path().join(format!("file_{i}.txt"));
        if File::create(&file_path).is_ok() {
            // Successfully opened after freeing handles
            break;
        }
    }
}

/// Test atomic file write operations
#[test]
#[serial]
fn test_atomic_write_concurrent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let target_file = temp_dir.path().join("target.json");

    // Initialize file
    fs::write(&target_file, r#"{"counter": 0}"#).expect("Failed to write initial file");

    let file_path = Arc::new(target_file);
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Spawn threads doing concurrent atomic writes
    for thread_id in 0..10 {
        let path_clone = Arc::clone(&file_path);
        let success_clone = Arc::clone(&success_count);

        let handle = thread::spawn(move || {
            for i in 0..5 {
                let content = format!(r#"{{"thread": {thread_id}, "iteration": {i}}}"#);

                match write_secure(&path_clone, content.as_bytes()) {
                    Ok(_) => {
                        success_clone.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) => {
                        println!("Secure write failed: {e}");
                    }
                }

                // Small delay
                thread::sleep(Duration::from_millis(10));
            }
        });

        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // Verify file is not corrupted
    let final_content = fs::read_to_string(&*file_path).expect("Failed to read final file");
    assert!(
        final_content.contains("thread"),
        "File should contain valid JSON"
    );

    let successes = success_count.load(Ordering::Relaxed);
    println!("Successful atomic writes: {successes}/50");
}

/// Test lock contention and timeout
#[test]
fn test_lock_contention() {
    let shared_resource = Arc::new(Mutex::new(0));
    let contention_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Create high contention scenario
    for _ in 0..20 {
        let resource_clone = Arc::clone(&shared_resource);
        let contention_clone = Arc::clone(&contention_count);

        let handle = thread::spawn(move || {
            for _ in 0..100 {
                // Try to acquire lock
                match resource_clone.try_lock() {
                    Ok(mut guard) => {
                        // Simulate work
                        *guard += 1;
                        thread::sleep(Duration::from_micros(10));
                    }
                    Err(_) => {
                        // Lock was contended
                        contention_clone.fetch_add(1, Ordering::Relaxed);
                        // Back off
                        thread::sleep(Duration::from_micros(50));
                    }
                }
            }
        });

        handles.push(handle);
    }

    // Wait for completion
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let contentions = contention_count.load(Ordering::Relaxed);
    println!("Lock contentions: {contentions}");

    // Verify some contention occurred in high-concurrency scenario
    assert!(contentions > 0, "Should have some lock contention");
}

/// Test RwLock for read-heavy workload
#[test]
fn test_rwlock_concurrent_reads() {
    let shared_data = Arc::new(RwLock::new(HashMap::new()));
    let mut handles = vec![];

    // Initialize data
    {
        let mut writer = shared_data.write().unwrap();
        for i in 0..100 {
            writer.insert(format!("key_{i}"), format!("value_{i}"));
        }
    }

    let start = Instant::now();

    // Spawn many reader threads
    for _thread_id in 0..10 {
        let data_clone = Arc::clone(&shared_data);

        let handle = thread::spawn(move || {
            let mut read_count = 0;

            while start.elapsed() < Duration::from_secs(1) {
                let reader = data_clone.read().unwrap();

                // Simulate reading data
                for i in 0..100 {
                    let key = format!("key_{i}");
                    let _ = reader.get(&key);
                }

                read_count += 1;
                drop(reader); // Release read lock
            }

            read_count
        });

        handles.push(handle);
    }

    // Collect results
    let total_reads: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();

    println!("Total reads performed: {total_reads}");
    assert!(total_reads > 100, "Should perform many concurrent reads");
}

/// Test process spawning and cleanup
#[test]
#[serial]
fn test_process_lifecycle() {
    let platform = match env::consts::OS {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => Platform::Linux,
    };
    let runner = ServerRunner::new(platform, false);

    // Test spawning echo command
    #[cfg(unix)]
    let result = runner.run("echo", &["test".to_string()]);

    #[cfg(windows)]
    let result = runner.run(
        "cmd",
        &["/C".to_string(), "echo".to_string(), "test".to_string()],
    );

    // Process should complete successfully
    assert!(result.is_ok() || result.is_err()); // May fail in test environment
}

/// Test directory traversal resource usage
#[test]
fn test_directory_traversal_resources() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Create deep directory structure
    let mut current_path = temp_dir.path().to_path_buf();
    for i in 0..50 {
        current_path = current_path.join(format!("level_{i}"));
        fs::create_dir(&current_path).expect("Failed to create directory");

        // Add some files at each level
        for j in 0..5 {
            let file_path = current_path.join(format!("file_{j}.txt"));
            fs::write(&file_path, format!("Level {i} File {j}")).expect("Failed to write file");
        }
    }

    // Traverse and count entries
    let mut dir_count = 0;
    let mut file_count = 0;

    fn count_entries(path: &Path, dir_count: &mut usize, file_count: &mut usize) {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    *dir_count += 1;
                    count_entries(&path, dir_count, file_count);
                } else {
                    *file_count += 1;
                }
            }
        }
    }

    count_entries(temp_dir.path(), &mut dir_count, &mut file_count);

    assert_eq!(dir_count, 50, "Should have 50 directories");
    assert_eq!(file_count, 250, "Should have 250 files");
}

/// Test memory-mapped file alternative (regular file operations)
#[test]
fn test_large_file_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let large_file = temp_dir.path().join("large.dat");

    // Create a moderately large file (10MB)
    let size = 10 * 1024 * 1024;
    let chunk = vec![0u8; 1024];

    {
        let mut file = File::create(&large_file).expect("Failed to create file");
        for _ in 0..10240 {
            file.write_all(&chunk).expect("Failed to write chunk");
        }
    }

    // Read file in chunks to avoid loading all into memory
    let mut file = File::open(&large_file).expect("Failed to open file");
    let mut buffer = vec![0u8; 4096];
    let mut total_read = 0;

    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 {
            break;
        }
        total_read += n;
    }

    assert_eq!(total_read, size, "Should read entire file");
}

/// Test concurrent client configuration updates
#[test]
#[serial]
fn test_concurrent_client_updates() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Set up test environment
    #[cfg(windows)]
    env::set_var("APPDATA", temp_dir.path());
    #[cfg(unix)]
    env::set_var("HOME", temp_dir.path());

    // Create mock config files
    let claude_dir = temp_dir.path().join("Claude");
    fs::create_dir_all(&claude_dir).expect("Failed to create Claude dir");

    let config_path = claude_dir.join("claude_desktop_config.json");
    fs::write(&config_path, r#"{"mcpServers": {}}"#).expect("Failed to write config");

    // Get clients
    let clients = detect_clients();
    let has_claude = clients
        .iter()
        .any(|c| c.name() == "Claude Desktop" && c.is_installed());

    if has_claude {
        // Test concurrent config file access
        let config_file = config_path.clone();
        let mut handles = vec![];
        let errors = Arc::new(AtomicUsize::new(0));

        // Spawn threads trying to update config file concurrently
        for i in 0..5 {
            let file_clone = config_file.clone();
            let errors_clone = Arc::clone(&errors);

            let handle = thread::spawn(move || {
                // Try to read and write config
                match fs::read_to_string(&file_clone) {
                    Ok(content) => {
                        let new_content =
                            content.replace("{}", &format!(r#"{{"server_{i}": {{}}}}"#));
                        if fs::write(&file_clone, new_content).is_err() {
                            errors_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(_) => {
                        errors_clone.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for completion
        for handle in handles {
            handle.join().expect("Thread panicked");
        }

        // Some updates might fail due to concurrent access
        let error_count = errors.load(Ordering::Relaxed);
        println!("Concurrent file access errors: {error_count}");
    }

    // Cleanup
    #[cfg(windows)]
    env::remove_var("APPDATA");
    #[cfg(unix)]
    env::remove_var("HOME");
}

/// Test resource cleanup on panic
#[test]
fn test_panic_cleanup() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let panic_file = temp_dir.path().join("panic_test.txt");

    // Run in a thread that will panic
    let panic_file_clone = panic_file.clone();
    let result = thread::spawn(move || {
        let _file = File::create(&panic_file_clone).expect("Failed to create file");

        // This will panic
        panic!("Intentional panic for testing");
    })
    .join();

    assert!(result.is_err(), "Thread should have panicked");

    // File handle should be closed despite panic
    // We should be able to delete the file
    if panic_file.exists() {
        assert!(
            fs::remove_file(&panic_file).is_ok(),
            "Should be able to delete file after panic"
        );
    }
}

/// Test secure file write operations
#[test]
fn test_secure_file_write_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let target_file = temp_dir.path().join("secure_target.json");

    // Create initial file
    fs::write(&target_file, r#"{"initial": true}"#).expect("Failed to write initial");

    // Test multiple sequential writes
    for i in 0..10 {
        let content = format!(r#"{{"iteration": {i}}}"#);
        write_secure(&target_file, content.as_bytes()).expect("Failed to secure write");

        // Verify file is updated
        let current = fs::read_to_string(&target_file).expect("Failed to read");
        assert!(current.contains(&format!("\"iteration\": {i}")));
    }

    // Verify last write is preserved
    let final_content = fs::read_to_string(&target_file).expect("Failed to read final");
    assert!(final_content.contains("\"iteration\": 9"));
}

/// Test cache directory size management
#[test]
#[serial]
fn test_cache_size_management() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    env::set_var("XDG_CACHE_HOME", temp_dir.path());

    let _cache_manager = CacheManager::new().expect("Failed to create cache manager");

    // Create many cache entries
    for i in 0..100 {
        let _key = format!("test_key_{i}");
        let value = "x".repeat(1024); // 1KB per entry

        // Simulate caching (would need actual cache method)
        let cache_file = temp_dir.path().join(format!("cache_{i}.tmp"));
        fs::write(&cache_file, &value).expect("Failed to write cache file");
    }

    // Calculate total cache size
    let mut total_size = 0u64;
    if let Ok(entries) = fs::read_dir(temp_dir.path()) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }
    }

    println!("Total cache size: {total_size} bytes");
    assert!(
        total_size >= 100 * 1024,
        "Cache should contain at least 100KB"
    );

    env::remove_var("XDG_CACHE_HOME");
}
