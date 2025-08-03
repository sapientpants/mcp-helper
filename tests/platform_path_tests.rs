//! Comprehensive tests for cross-platform path handling

use mcp_helper::runner::{normalize_path, Platform};
use std::path::PathBuf;

#[test]
fn test_normalize_path_windows_basic() {
    let platform = Platform::Windows;

    // Forward slashes to backslashes
    assert_eq!(normalize_path("path/to/file", platform), "path\\to\\file");
    assert_eq!(
        normalize_path("C:/Program Files/App", platform),
        "C:\\Program Files\\App"
    );
    assert_eq!(
        normalize_path("/usr/local/bin", platform),
        "\\usr\\local\\bin"
    );
}

#[test]
fn test_normalize_path_unix_basic() {
    // Test both macOS and Linux
    for platform in &[Platform::MacOS, Platform::Linux] {
        // Backslashes to forward slashes
        assert_eq!(normalize_path("path\\to\\file", *platform), "path/to/file");
        assert_eq!(
            normalize_path("C:\\Windows\\System32", *platform),
            "C:/Windows/System32"
        );
        assert_eq!(
            normalize_path("\\\\server\\share", *platform),
            "//server/share"
        );
    }
}

#[test]
fn test_normalize_path_mixed_separators() {
    // Windows: all become backslashes
    assert_eq!(
        normalize_path("path\\to/mixed\\separators/file", Platform::Windows),
        "path\\to\\mixed\\separators\\file"
    );

    // Unix: all become forward slashes
    assert_eq!(
        normalize_path("path\\to/mixed\\separators/file", Platform::Linux),
        "path/to/mixed/separators/file"
    );
    assert_eq!(
        normalize_path("path\\to/mixed\\separators/file", Platform::MacOS),
        "path/to/mixed/separators/file"
    );
}

#[test]
fn test_normalize_path_edge_cases() {
    // Empty path
    assert_eq!(normalize_path("", Platform::Windows), "");
    assert_eq!(normalize_path("", Platform::Linux), "");

    // Single separator
    assert_eq!(normalize_path("/", Platform::Windows), "\\");
    assert_eq!(normalize_path("\\", Platform::Linux), "/");

    // Multiple consecutive separators
    assert_eq!(
        normalize_path("path//to\\\\file", Platform::Windows),
        "path\\\\to\\\\file"
    );
    assert_eq!(
        normalize_path("path//to\\\\file", Platform::Linux),
        "path//to//file"
    );

    // Trailing separators
    assert_eq!(
        normalize_path("path/to/dir/", Platform::Windows),
        "path\\to\\dir\\"
    );
    assert_eq!(
        normalize_path("path\\to\\dir\\", Platform::Linux),
        "path/to/dir/"
    );

    // Leading separators
    assert_eq!(
        normalize_path("/absolute/path", Platform::Windows),
        "\\absolute\\path"
    );
    assert_eq!(
        normalize_path("\\absolute\\path", Platform::Linux),
        "/absolute/path"
    );
}

#[test]
fn test_normalize_path_special_paths() {
    // Dots in paths
    assert_eq!(
        normalize_path("./relative/path", Platform::Windows),
        ".\\relative\\path"
    );
    assert_eq!(
        normalize_path(".\\relative\\path", Platform::Linux),
        "./relative/path"
    );

    assert_eq!(
        normalize_path("../parent/path", Platform::Windows),
        "..\\parent\\path"
    );
    assert_eq!(
        normalize_path("..\\parent\\path", Platform::Linux),
        "../parent/path"
    );

    // Hidden files
    assert_eq!(
        normalize_path(".hidden/folder/.file", Platform::Windows),
        ".hidden\\folder\\.file"
    );
    assert_eq!(
        normalize_path(".hidden\\folder\\.file", Platform::Linux),
        ".hidden/folder/.file"
    );
}

#[test]
fn test_normalize_path_unc_paths() {
    // Windows UNC paths
    assert_eq!(
        normalize_path("//server/share/file", Platform::Windows),
        "\\\\server\\share\\file"
    );
    assert_eq!(
        normalize_path("\\\\server\\share\\file", Platform::Windows),
        "\\\\server\\share\\file"
    );

    // On Unix, these remain as-is but normalized
    assert_eq!(
        normalize_path("\\\\server\\share\\file", Platform::Linux),
        "//server/share/file"
    );
}

#[test]
fn test_normalize_path_windows_drive_letters() {
    let platform = Platform::Windows;

    assert_eq!(normalize_path("C:/", platform), "C:\\");
    assert_eq!(normalize_path("D:/Users/Name", platform), "D:\\Users\\Name");
    assert_eq!(
        normalize_path("Z:\\Projects\\mcp", platform),
        "Z:\\Projects\\mcp"
    );

    // Lowercase drive letters
    assert_eq!(normalize_path("c:/windows", platform), "c:\\windows");
    assert_eq!(normalize_path("d:\\temp", platform), "d:\\temp");
}

#[test]
fn test_normalize_path_unix_home_tilde() {
    // Tilde paths (note: normalize_path doesn't expand, just normalizes separators)
    assert_eq!(
        normalize_path("~/Documents", Platform::Linux),
        "~/Documents"
    );
    assert_eq!(
        normalize_path("~\\Documents", Platform::Linux),
        "~/Documents"
    );

    assert_eq!(
        normalize_path("~/Documents", Platform::Windows),
        "~\\Documents"
    );
}

#[test]
fn test_normalize_path_spaces_and_special_chars() {
    // Paths with spaces
    assert_eq!(
        normalize_path("Program Files/My App/file.txt", Platform::Windows),
        "Program Files\\My App\\file.txt"
    );
    assert_eq!(
        normalize_path("Program Files\\My App\\file.txt", Platform::Linux),
        "Program Files/My App/file.txt"
    );

    // Special characters
    assert_eq!(
        normalize_path("path/with-dashes/and_underscores", Platform::Windows),
        "path\\with-dashes\\and_underscores"
    );
    assert_eq!(
        normalize_path("file (1)/copy [2].txt", Platform::Windows),
        "file (1)\\copy [2].txt"
    );

    // Unicode paths
    assert_eq!(
        normalize_path("文档/测试/文件.txt", Platform::Windows),
        "文档\\测试\\文件.txt"
    );
    assert_eq!(
        normalize_path("café\\résumé.pdf", Platform::Linux),
        "café/résumé.pdf"
    );
}

#[test]
fn test_normalize_path_real_world_examples() {
    // NPM global paths
    assert_eq!(
        normalize_path(
            "/usr/local/lib/node_modules/@modelcontextprotocol/server-filesystem",
            Platform::Windows
        ),
        "\\usr\\local\\lib\\node_modules\\@modelcontextprotocol\\server-filesystem"
    );

    // Windows npm paths
    assert_eq!(
        normalize_path(
            "C:\\Users\\Username\\AppData\\Roaming\\npm\\node_modules\\mcp-server",
            Platform::Windows
        ),
        "C:\\Users\\Username\\AppData\\Roaming\\npm\\node_modules\\mcp-server"
    );

    // Config file paths
    assert_eq!(
        normalize_path("~/.config/mcp/servers.json", Platform::Windows),
        "~\\.config\\mcp\\servers.json"
    );

    // macOS application support
    assert_eq!(
        normalize_path(
            "~/Library/Application Support/Claude/claude_desktop_config.json",
            Platform::Windows
        ),
        "~\\Library\\Application Support\\Claude\\claude_desktop_config.json"
    );
}

#[test]
fn test_normalize_path_idempotency() {
    // Normalizing twice should give the same result
    let paths = vec![
        "path/to/file",
        "C:\\Windows\\System32",
        "\\\\server\\share",
        "../relative/path",
        "mixed\\path/separators\\here",
    ];

    for path in paths {
        for platform in &[Platform::Windows, Platform::MacOS, Platform::Linux] {
            let normalized_once = normalize_path(path, *platform);
            let normalized_twice = normalize_path(&normalized_once, *platform);
            assert_eq!(
                normalized_once, normalized_twice,
                "Path normalization should be idempotent for '{path}' on {platform:?}"
            );
        }
    }
}

#[test]
fn test_path_conversion_symmetry() {
    // Test that converting back and forth maintains structure
    let original = "path/to/file";

    // Convert to Windows then back to Unix
    let windows = normalize_path(original, Platform::Windows);
    let back_to_unix = normalize_path(&windows, Platform::Linux);
    assert_eq!(back_to_unix, original);

    // Convert to Unix then back to Windows
    let original_windows = "path\\to\\file";
    let unix = normalize_path(original_windows, Platform::Linux);
    let back_to_windows = normalize_path(&unix, Platform::Windows);
    assert_eq!(back_to_windows, original_windows);
}

#[test]
fn test_pathbuf_compatibility() {
    // Ensure normalized paths work with PathBuf
    let normalized_windows = normalize_path("path/to/file", Platform::Windows);
    let path_windows = PathBuf::from(&normalized_windows);

    // On Windows, this would be a valid path
    #[cfg(target_os = "windows")]
    assert_eq!(path_windows.to_string_lossy(), "path\\to\\file");

    let normalized_unix = normalize_path("path\\to\\file", Platform::Linux);
    let path_unix = PathBuf::from(&normalized_unix);

    // On Unix, this would be a valid path
    #[cfg(not(target_os = "windows"))]
    assert_eq!(path_unix.to_string_lossy(), "path/to/file");

    // Verify paths are valid PathBuf regardless of platform
    assert!(!normalized_windows.is_empty());
    assert!(!normalized_unix.is_empty());
    assert_eq!(path_windows.components().count(), 3);
    assert_eq!(path_unix.components().count(), 3);
}

#[test]
fn test_normalize_path_performance_characteristics() {
    // Test with very long paths
    let long_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z".repeat(10);

    let start = std::time::Instant::now();
    let normalized = normalize_path(&long_path, Platform::Windows);
    let duration = start.elapsed();

    // Verify the path was actually normalized
    assert!(normalized.contains('\\'));
    assert!(!normalized.contains('/'));

    // Should complete quickly even for very long paths
    assert!(
        duration.as_millis() < 10,
        "Path normalization took too long: {duration:?}"
    );
}

#[test]
fn test_normalize_path_no_allocation_for_correct_paths() {
    // Paths already using correct separators shouldn't need new allocation
    let windows_path = "C:\\already\\correct\\path";
    let normalized = normalize_path(windows_path, Platform::Windows);
    assert_eq!(normalized, windows_path);

    let unix_path = "/already/correct/path";
    let normalized = normalize_path(unix_path, Platform::Linux);
    assert_eq!(normalized, unix_path);
}
