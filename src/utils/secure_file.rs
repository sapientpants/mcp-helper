//! Secure file operations with appropriate permissions.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

/// Write data to a file with secure permissions (0600 on Unix).
///
/// This function ensures that configuration files are written with
/// appropriate permissions to prevent unauthorized access.
///
/// # Arguments
/// * `path` - The target path for the file
/// * `contents` - The contents to write to the file
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::utils::secure_file;
///
/// secure_file::write_secure("/path/to/config.json", b"{ \"key\": \"value\" }")?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn write_secure(path: &Path, contents: &[u8]) -> Result<()> {
    // Create a temporary file in the same directory
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let temp_file = NamedTempFile::new_in(parent).context("Failed to create temporary file")?;

    // Write contents to temporary file
    fs::write(temp_file.path(), contents).context("Failed to write to temporary file")?;

    // Set secure permissions on Unix-like systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(temp_file.path())?.permissions();
        perms.set_mode(0o600); // Read/write for owner only
        fs::set_permissions(temp_file.path(), perms).context("Failed to set file permissions")?;
    }

    // Persist the temporary file to the target path
    temp_file
        .persist(path)
        .with_context(|| format!("Failed to persist file to {}", path.display()))?;

    Ok(())
}

/// Write a JSON string to a file with secure permissions.
///
/// This is a convenience wrapper around `write_secure` for JSON data.
///
/// # Arguments
/// * `path` - The target path for the JSON file
/// * `json` - The JSON string to write
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::utils::secure_file;
///
/// let json = serde_json::json!({"key": "value"}).to_string();
/// secure_file::write_json_secure("/path/to/config.json", &json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn write_json_secure(path: &Path, json: &str) -> Result<()> {
    write_secure(path, json.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_secure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        write_secure(&file_path, b"test content").unwrap();

        // Verify file was created
        assert!(file_path.exists());

        // Verify content
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");

        // Verify permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path).unwrap();
            let mode = metadata.permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
    }

    #[test]
    fn test_write_json_secure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.json");

        let json = r#"{"key": "value"}"#;
        write_json_secure(&file_path, json).unwrap();

        // Verify file was created
        assert!(file_path.exists());

        // Verify content
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, json);
    }
}
