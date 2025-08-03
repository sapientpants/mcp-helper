//! Comprehensive tests for platform detection and platform-specific behavior

use mcp_helper::runner::Platform;
use std::env;

// Helper function to simulate platform detection
fn detect_platform_from_os(os: &str) -> Platform {
    match os {
        "windows" => Platform::Windows,
        "macos" => Platform::MacOS,
        "linux" => Platform::Linux,
        _ => Platform::Linux, // Default fallback
    }
}

#[test]
fn test_platform_enum_equality() {
    assert_eq!(Platform::Windows, Platform::Windows);
    assert_eq!(Platform::MacOS, Platform::MacOS);
    assert_eq!(Platform::Linux, Platform::Linux);

    assert_ne!(Platform::Windows, Platform::MacOS);
    assert_ne!(Platform::Windows, Platform::Linux);
    assert_ne!(Platform::MacOS, Platform::Linux);
}

#[test]
fn test_platform_detection_all_platforms() {
    assert_eq!(detect_platform_from_os("windows"), Platform::Windows);
    assert_eq!(detect_platform_from_os("macos"), Platform::MacOS);
    assert_eq!(detect_platform_from_os("linux"), Platform::Linux);

    // Test unknown platforms default to Linux
    assert_eq!(detect_platform_from_os("freebsd"), Platform::Linux);
    assert_eq!(detect_platform_from_os("unknown"), Platform::Linux);
}

#[test]
#[cfg(target_os = "windows")]
fn test_current_platform_windows() {
    // On Windows, this should match
    assert_eq!(env::consts::OS, "windows");
    let platform = detect_platform_from_os(env::consts::OS);
    assert_eq!(platform, Platform::Windows);
}

#[test]
#[cfg(target_os = "macos")]
fn test_current_platform_macos() {
    // On macOS, this should match
    assert_eq!(env::consts::OS, "macos");
    let platform = detect_platform_from_os(env::consts::OS);
    assert_eq!(platform, Platform::MacOS);
}

#[test]
#[cfg(target_os = "linux")]
fn test_current_platform_linux() {
    // On Linux, this should match
    assert_eq!(env::consts::OS, "linux");
    let platform = detect_platform_from_os(env::consts::OS);
    assert_eq!(platform, Platform::Linux);
}

#[test]
fn test_platform_debug_display() {
    // Test Debug trait implementation
    let windows = format!("{:?}", Platform::Windows);
    let macos = format!("{:?}", Platform::MacOS);
    let linux = format!("{:?}", Platform::Linux);

    assert_eq!(windows, "Windows");
    assert_eq!(macos, "MacOS");
    assert_eq!(linux, "Linux");
}

#[test]
fn test_platform_copy_clone() {
    // Test that Platform implements Copy and Clone
    let platform1 = Platform::Windows;
    let platform2 = platform1; // Copy
    let platform3 = platform1; // Copy

    assert_eq!(platform1, platform2);
    assert_eq!(platform1, platform3);
    assert_eq!(platform2, platform3);
}

#[test]
fn test_platform_pattern_matching() {
    let platforms = vec![Platform::Windows, Platform::MacOS, Platform::Linux];

    for platform in platforms {
        let name = match platform {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
        };

        match platform {
            Platform::Windows => assert_eq!(name, "Windows"),
            Platform::MacOS => assert_eq!(name, "macOS"),
            Platform::Linux => assert_eq!(name, "Linux"),
        }
    }
}

#[test]
fn test_platform_in_collections() {
    // Platform doesn't implement Hash/Eq, so we can't use it as HashMap key
    // But we can use it in a Vec
    let platforms = vec![
        (Platform::Windows, "Microsoft Windows"),
        (Platform::MacOS, "Apple macOS"),
        (Platform::Linux, "GNU/Linux"),
    ];

    for (platform, name) in &platforms {
        match platform {
            Platform::Windows => assert_eq!(*name, "Microsoft Windows"),
            Platform::MacOS => assert_eq!(*name, "Apple macOS"),
            Platform::Linux => assert_eq!(*name, "GNU/Linux"),
        }
    }
}

#[test]
fn test_platform_conditional_compilation() {
    #[cfg(target_os = "windows")]
    {
        let expected = Platform::Windows;
        let actual = detect_platform_from_os("windows");
        assert_eq!(actual, expected);
    }

    #[cfg(target_os = "macos")]
    {
        let expected = Platform::MacOS;
        let actual = detect_platform_from_os("macos");
        assert_eq!(actual, expected);
    }

    #[cfg(target_os = "linux")]
    {
        let expected = Platform::Linux;
        let actual = detect_platform_from_os("linux");
        assert_eq!(actual, expected);
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        // Other platforms should default to Linux
        let actual = detect_platform_from_os(env::consts::OS);
        assert_eq!(actual, Platform::Linux);
    }
}

#[test]
fn test_platform_thread_safety() {
    // Platform should be Send + Sync
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Platform>();

    // Test across threads
    use std::thread;

    let platform = Platform::Windows;

    let handles: Vec<_> = (0..3)
        .map(|_| {
            thread::spawn(move || {
                assert_eq!(platform, Platform::Windows);
                platform
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert_eq!(result, Platform::Windows);
    }
}

#[test]
fn test_platform_match_exhaustiveness() {
    // This test ensures all platform variants are handled
    fn get_platform_name(platform: Platform) -> &'static str {
        match platform {
            Platform::Windows => "Windows",
            Platform::MacOS => "macOS",
            Platform::Linux => "Linux",
            // If a new variant is added, this will fail to compile
        }
    }

    assert_eq!(get_platform_name(Platform::Windows), "Windows");
    assert_eq!(get_platform_name(Platform::MacOS), "macOS");
    assert_eq!(get_platform_name(Platform::Linux), "Linux");
}

#[test]
fn test_platform_size_and_alignment() {
    // Ensure Platform is efficiently represented
    use std::mem;

    // Platform enum should be small (likely 1 byte)
    assert!(mem::size_of::<Platform>() <= 8);

    // Should have reasonable alignment
    assert!(mem::align_of::<Platform>() <= 8);

    // Should be zero-cost to pass around
    assert_eq!(
        mem::size_of::<Platform>(),
        mem::size_of::<Option<Platform>>()
    );
}
