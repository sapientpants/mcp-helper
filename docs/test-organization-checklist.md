# Test Organization Checklist

This checklist ensures consistent test organization across the MCP Helper codebase.

## Before Writing Tests

- [ ] Identify which type of test you're writing:
  - [ ] **Unit Test**: Testing a single function/method in isolation
  - [ ] **Integration Test**: Testing module interactions
  - [ ] **E2E Test**: Testing the complete user workflow

- [ ] Choose the correct location:
  - [ ] **Inline tests** (`#[cfg(test)]` in source files) for unit tests
  - [ ] **Integration tests** (`tests/` directory) for broader scenarios

## Inline Test Module Structure

Use this pattern for all inline test modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit {
        use super::*;
        
        mod feature_or_struct_name {
            use super::*;
            
            #[test]
            fn descriptive_test_name() {
                // Test implementation
            }
        }
    }
    
    // Optional: only if you have integration-style tests inline
    mod integration {
        use super::*;
        // Tests that use filesystem, network, etc.
    }
    
    // Optional: only if you need module-specific fixtures
    mod fixtures {
        use super::*;
        // Test data specific to this module
    }
}
```

## Test Naming Conventions

- [ ] Use descriptive names that explain the scenario
- [ ] Follow pattern: `test_[function]_[scenario]_[expected_result]`
- [ ] Examples:
  - [ ] `test_parse_version_with_valid_input_returns_some()`
  - [ ] `test_parse_version_with_invalid_input_returns_none()`
  - [ ] `test_install_server_with_missing_dependency_fails()`

## Test Structure Checklist

For each test:

- [ ] **Arrange**: Set up test data and mocks
- [ ] **Act**: Execute the code under test
- [ ] **Assert**: Verify the results

```rust
#[test]
fn test_example() {
    // Arrange
    let input = create_test_input();
    let expected = ExpectedResult::new();
    
    // Act
    let result = function_under_test(input);
    
    // Assert
    assert_eq!(result, expected);
}
```

## Using Shared Test Utilities

- [ ] Import utilities at the top of your test module:
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      use crate::test_utils::{mocks::*, fixtures::*, assertions::*};
  ```

- [ ] Use builders for complex test data:
  ```rust
  let server = MockServerBuilder::new("test-server")
      .with_dependency(Dependency::NodeJs { min_version: None })
      .build();
  ```

- [ ] Use fixtures for common data:
  ```rust
  let config = sample_server_config();
  ```

- [ ] Use custom assertions:
  ```rust
  assert_file_exists(path);
  assert_contains_all(output, &["expected", "text"]);
  ```

## Platform-Specific Testing

- [ ] Use conditional compilation for platform-specific tests:
  ```rust
  #[test]
  #[cfg(target_os = "windows")]
  fn test_windows_specific_feature() {
      // Windows-only test
  }
  
  #[test]
  #[cfg(unix)]
  fn test_unix_feature() {
      // Unix-only test
  }
  ```

- [ ] Test cross-platform behavior:
  ```rust
  #[test]
  fn test_cross_platform_paths() {
      #[cfg(windows)]
      let expected = "C:\\path\\to\\file";
      
      #[cfg(unix)]
      let expected = "/path/to/file";
      
      assert_paths_equal(result, expected);
  }
  ```

## Error Testing

- [ ] Test both success and failure paths
- [ ] Use `assert_err()` helper for error cases:
  ```rust
  let error = assert_err(function_that_should_fail());
  assert!(error.to_string().contains("expected message"));
  ```

- [ ] Test error display formatting:
  ```rust
  let error = create_test_error();
  let display = format!("{}", error);
  assert!(display.contains("✗")); // Error symbol
  ```

## Mock Usage Guidelines

- [ ] Use existing mock builders when possible
- [ ] Keep mocks simple - avoid complex business logic
- [ ] Document mock behavior:
  ```rust
  // Mock that always returns success
  let mock = MockServerBuilder::new("test")
      .with_config_validator(|_| Ok(()))
      .build();
  ```

## Coverage Considerations

- [ ] Ensure critical paths have >90% coverage
- [ ] Document coverage exemptions:
  ```rust
  #[cfg(target_os = "windows")]
  fn platform_specific() {
      // COVERAGE: Cannot test on non-Windows platforms
  }
  ```

- [ ] Test all enum variants and match arms
- [ ] Test boundary conditions (empty input, max values, etc.)

## Integration Test Considerations

For tests in the `tests/` directory:

- [ ] Create realistic test scenarios
- [ ] Use temporary directories for file operations
- [ ] Clean up test artifacts
- [ ] Test complete workflows, not individual functions

## Documentation

- [ ] Add module-level documentation for complex test suites:
  ```rust
  //! Tests for the server installation module
  //! 
  //! These tests cover:
  //! - NPM server installation
  //! - Dependency validation
  //! - Configuration management
  ```

- [ ] Document test setup when complex:
  ```rust
  /// Sets up a mock server environment with:
  /// - Node.js dependency available
  /// - Valid configuration
  /// - Temporary directory for installation
  fn setup_test_environment() -> TestEnvironment {
      // Setup code
  }
  ```

## Before Submitting

- [ ] Run all tests: `cargo test`
- [ ] Check coverage: `cargo tarpaulin --lib --bins`
- [ ] Verify tests pass on different platforms (CI)
- [ ] Review test names for clarity
- [ ] Ensure no test artifacts remain in working directory
- [ ] Run with `--release` flag to catch release-only issues

## Common Anti-Patterns to Avoid

- [ ] ❌ Don't put unrelated tests in the same test function
- [ ] ❌ Don't use magic numbers without explanation
- [ ] ❌ Don't test implementation details, test behavior
- [ ] ❌ Don't create tests that depend on external services
- [ ] ❌ Don't ignore platform differences in path testing
- [ ] ❌ Don't use `unwrap()` in tests without good reason
- [ ] ❌ Don't create tests that require specific execution order

## Quality Checklist

Before considering tests complete:

- [ ] All happy paths tested
- [ ] All error paths tested
- [ ] Edge cases covered (empty, null, boundary values)
- [ ] Platform-specific behavior tested
- [ ] Mocks are realistic and focused
- [ ] Test names clearly describe the scenario
- [ ] No flaky tests (run multiple times to verify)
- [ ] Tests are fast (<100ms each for unit tests)
- [ ] No external dependencies in unit tests