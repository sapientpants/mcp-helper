//! Custom assertion helpers for common test patterns
//!
//! This module provides custom assertion functions that make tests
//! more expressive and reduce boilerplate code.

use std::collections::HashMap;
use std::path::Path;

/// Asserts that a path exists and is a file
pub fn assert_file_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "Expected file to exist at path: {}",
        path.display()
    );
    assert!(
        path.is_file(),
        "Expected path to be a file, but it was not: {}",
        path.display()
    );
}

/// Asserts that a path exists and is a directory
pub fn assert_dir_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "Expected directory to exist at path: {}",
        path.display()
    );
    assert!(
        path.is_dir(),
        "Expected path to be a directory, but it was not: {}",
        path.display()
    );
}

/// Asserts that a path does not exist
pub fn assert_not_exists<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        !path.exists(),
        "Expected path to not exist, but it does: {}",
        path.display()
    );
}

/// Asserts that two HashMaps have the same keys (values may differ)
pub fn assert_same_keys<K, V1, V2>(map1: &HashMap<K, V1>, map2: &HashMap<K, V2>)
where
    K: Eq + std::hash::Hash + std::fmt::Debug,
{
    let keys1: std::collections::HashSet<_> = map1.keys().collect();
    let keys2: std::collections::HashSet<_> = map2.keys().collect();

    assert_eq!(
        keys1, keys2,
        "HashMaps have different keys.\nMap1 keys: {keys1:?}\nMap2 keys: {keys2:?}"
    );
}

/// Asserts that a Result is Ok and returns the value
pub fn assert_ok<T, E>(result: Result<T, E>) -> T
where
    E: std::fmt::Debug,
{
    match result {
        Ok(value) => value,
        Err(e) => panic!("Expected Ok result, but got Err: {e:?}"),
    }
}

/// Asserts that a Result is Err and returns the error
pub fn assert_err<T, E>(result: Result<T, E>) -> E
where
    T: std::fmt::Debug,
{
    match result {
        Ok(value) => panic!("Expected Err result, but got Ok: {value:?}"),
        Err(e) => e,
    }
}

/// Asserts that a string contains all of the given substrings
pub fn assert_contains_all(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "Expected string to contain '{needle}', but it didn't.\nString: {haystack}"
        );
    }
}

/// Asserts that a string contains none of the given substrings
pub fn assert_contains_none(haystack: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            !haystack.contains(needle),
            "Expected string to NOT contain '{needle}', but it did.\nString: {haystack}"
        );
    }
}

/// Asserts that a vector contains a specific element
pub fn assert_vec_contains<T>(vec: &[T], item: &T)
where
    T: PartialEq + std::fmt::Debug,
{
    assert!(
        vec.contains(item),
        "Expected vector to contain {item:?}, but it didn't.\nVector: {vec:?}"
    );
}

/// Asserts that two paths are equivalent (handles platform differences)
pub fn assert_paths_equal<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();

    // Normalize paths for comparison
    let normalized1 = path1.to_string_lossy().replace('\\', "/");
    let normalized2 = path2.to_string_lossy().replace('\\', "/");

    assert_eq!(
        normalized1,
        normalized2,
        "Paths are not equal.\nPath1: {}\nPath2: {}",
        path1.display(),
        path2.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_assert_same_keys() {
        let mut map1 = HashMap::new();
        map1.insert("key1", 1);
        map1.insert("key2", 2);

        let mut map2 = HashMap::new();
        map2.insert("key1", "different");
        map2.insert("key2", "values");

        assert_same_keys(&map1, &map2);
    }

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok(result);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        let error = assert_err(result);
        assert_eq!(error, "error");
    }

    #[test]
    fn test_assert_contains_all() {
        let text = "The quick brown fox jumps over the lazy dog";
        assert_contains_all(text, &["quick", "fox", "dog"]);
    }

    #[test]
    fn test_assert_contains_none() {
        let text = "The quick brown fox";
        assert_contains_none(text, &["cat", "mouse", "bird"]);
    }

    #[test]
    fn test_assert_vec_contains() {
        let vec = vec![1, 2, 3, 4, 5];
        assert_vec_contains(&vec, &3);
    }

    #[test]
    fn test_assert_paths_equal() {
        assert_paths_equal("/home/user/file.txt", "/home/user/file.txt");

        // Test platform-specific path separators
        #[cfg(windows)]
        assert_paths_equal("C:\\Users\\file.txt", "C:/Users/file.txt");
    }
}
