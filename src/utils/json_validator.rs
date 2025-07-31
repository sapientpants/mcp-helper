//! JSON validation utilities to ensure safe parsing and prevent injection.

use anyhow::{Context, Result};
use serde_json::Value;

/// Maximum allowed JSON depth to prevent stack overflow attacks.
const MAX_JSON_DEPTH: usize = 100;

/// Maximum allowed JSON string size (10MB).
const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;

/// Validates JSON input for safety before parsing.
///
/// This function checks for:
/// - Excessive nesting depth (prevents stack overflow)
/// - Large payload sizes (prevents memory exhaustion)
/// - Proper JSON syntax
///
/// # Arguments
/// * `json_str` - The JSON string to validate
///
/// # Returns
/// `Ok(())` if the JSON is safe to parse, or an error describing the issue
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::utils::json_validator;
///
/// let json = r#"{"key": "value"}"#;
/// json_validator::validate_json_input(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn validate_json_input(json_str: &str) -> Result<()> {
    // Check size first (cheapest check)
    if json_str.len() > MAX_JSON_SIZE {
        anyhow::bail!(
            "JSON input too large: {} bytes exceeds maximum of {} bytes",
            json_str.len(),
            MAX_JSON_SIZE
        );
    }

    // Parse and validate depth
    let value: Value = serde_json::from_str(json_str).context("Invalid JSON syntax")?;

    let depth = calculate_json_depth(&value);
    if depth > MAX_JSON_DEPTH {
        anyhow::bail!(
            "JSON nesting too deep: {} levels exceeds maximum of {}",
            depth,
            MAX_JSON_DEPTH
        );
    }

    Ok(())
}

/// Safely parse and validate JSON from a string.
///
/// This combines validation and parsing into a single operation.
///
/// # Arguments
/// * `json_str` - The JSON string to parse
///
/// # Returns
/// The parsed JSON value if valid, or an error
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::utils::json_validator;
/// use serde_json::Value;
///
/// let json = r#"{"key": "value"}"#;
/// let value: Value = json_validator::parse_json_safe(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn parse_json_safe(json_str: &str) -> Result<Value> {
    validate_json_input(json_str)?;
    serde_json::from_str(json_str).context("Failed to parse JSON")
}

/// Safely deserialize JSON into a specific type.
///
/// This validates the JSON before attempting deserialization.
///
/// # Arguments
/// * `json_str` - The JSON string to deserialize
///
/// # Returns
/// The deserialized value if valid, or an error
///
/// # Example
/// ```rust,no_run
/// use mcp_helper::utils::json_validator;
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct Config {
///     key: String,
/// }
///
/// let json = r#"{"key": "value"}"#;
/// let config: Config = json_validator::deserialize_json_safe(json)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn deserialize_json_safe<T>(json_str: &str) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    validate_json_input(json_str)?;
    serde_json::from_str(json_str).context("Failed to deserialize JSON")
}

/// Calculate the maximum nesting depth of a JSON value.
fn calculate_json_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => 1 + map.values().map(calculate_json_depth).max().unwrap_or(0),
        Value::Array(arr) => 1 + arr.iter().map(calculate_json_depth).max().unwrap_or(0),
        _ => 1,
    }
}

/// Sanitize a JSON string by removing potentially dangerous patterns.
///
/// This is a defense-in-depth measure that removes:
/// - JavaScript code patterns
/// - HTML/script tags
/// - Excessive whitespace
///
/// # Arguments
/// * `json_str` - The JSON string to sanitize
///
/// # Returns
/// A sanitized JSON string
pub fn sanitize_json_string(json_str: &str) -> String {
    json_str
        .replace("<script", "&lt;script")
        .replace("</script", "&lt;/script")
        .replace("javascript:", "")
        .replace("eval(", "")
        .replace("Function(", "")
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_json() {
        let json = r#"{"key": "value", "nested": {"inner": "data"}}"#;
        assert!(validate_json_input(json).is_ok());
    }

    #[test]
    fn test_invalid_json_syntax() {
        let json = r#"{"key": "value", invalid}"#;
        assert!(validate_json_input(json).is_err());
    }

    #[test]
    fn test_excessive_nesting() {
        let mut json = String::from("{");
        for _ in 0..MAX_JSON_DEPTH + 10 {
            json.push_str(r#""nested": {"#);
        }
        json.push_str(r#""value": 1"#);
        for _ in 0..MAX_JSON_DEPTH + 10 {
            json.push('}');
        }
        json.push('}');

        let result = validate_json_input(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nesting too deep"));
    }

    #[test]
    fn test_large_json() {
        let large_value = "x".repeat(MAX_JSON_SIZE);
        let json = format!(r#"{{"key": "{large_value}"}}"#);

        let result = validate_json_input(&json);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_parse_json_safe() {
        let json = r#"{"key": "value"}"#;
        let value = parse_json_safe(json).unwrap();
        assert_eq!(value["key"], "value");
    }

    #[test]
    fn test_deserialize_json_safe() {
        #[derive(serde::Deserialize, Debug, PartialEq)]
        struct TestConfig {
            key: String,
        }

        let json = r#"{"key": "value"}"#;
        let config: TestConfig = deserialize_json_safe(json).unwrap();
        assert_eq!(config.key, "value");
    }

    #[test]
    fn test_calculate_depth() {
        let json1 = serde_json::json!({"a": 1});
        assert_eq!(calculate_json_depth(&json1), 2);

        let json2 = serde_json::json!({"a": {"b": {"c": 1}}});
        assert_eq!(calculate_json_depth(&json2), 4);

        let json3 = serde_json::json!([1, 2, [3, 4, [5, 6]]]);
        assert_eq!(calculate_json_depth(&json3), 4);
    }

    #[test]
    fn test_sanitize_json_string() {
        let dangerous =
            r#"{"script": "<script>alert('xss')</script>", "js": "javascript:void(0)"}"#;
        let sanitized = sanitize_json_string(dangerous);

        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("javascript:"));
        assert!(sanitized.contains("&lt;script"));
    }
}
