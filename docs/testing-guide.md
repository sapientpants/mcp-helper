# Testing Guide for MCP Helper

## Overview

This guide explains the testing strategy for MCP Helper, including our approach to unit tests, integration tests, and the constraints imposed by Rust's coverage tools.

## Testing Philosophy

MCP Helper follows a comprehensive testing strategy designed to ensure reliability across Windows, macOS, and Linux platforms. Our tests are organized to provide maximum coverage while maintaining clarity and maintainability.

### Key Principles

1. **Coverage-Driven Development**: We maintain >80% code coverage for the library
2. **Platform Parity**: All platform-specific code paths must be tested
3. **Real-World Scenarios**: Tests should reflect actual usage patterns
4. **Fast Feedback**: Tests should run quickly to encourage frequent execution

## Test Organization

### Directory Structure

```
mcp-helper/
├── src/                     # Source code with inline unit tests
│   ├── error/
│   │   ├── mod.rs          # Module with inline tests
│   │   └── builder.rs      # Builder pattern with organized test submodules
│   └── test_utils/         # Shared test utilities (test-only)
│       ├── mocks.rs        # Reusable mock implementations
│       ├── fixtures.rs     # Test data builders
│       └── assertions.rs   # Custom assertion helpers
└── tests/                  # Integration tests (separate binaries)
    ├── cli_tests.rs
    ├── install_tests.rs
    └── ...
```

### Inline Test Organization Pattern

Due to coverage tool limitations (only inline tests count toward coverage metrics), we organize inline tests using submodules:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit {
        use super::*;
        
        mod feature_name {
            use super::*;
            
            #[test]
            fn test_basic_functionality() {
                // Pure unit test
            }
        }
    }
    
    mod integration {
        use super::*;
        // Tests that touch filesystem/network
    }
    
    mod fixtures {
        use super::*;
        // Test data and utilities specific to this module
    }
}
```

## Coverage Strategy

### Understanding Coverage Constraints

**Important**: Due to limitations in `cargo-tarpaulin` and similar tools:
- Only tests in `#[cfg(test)]` modules within source files contribute to coverage
- Integration tests in the `tests/` directory do NOT count toward coverage
- This forces us to place more tests inline than would be ideal in other languages

### Coverage Requirements

- **Target**: >80% overall coverage
- **Critical Paths**: >90% coverage for error handling, security validation
- **Platform-Specific Code**: Must have tests for all supported platforms

### Running Coverage

```bash
# Generate coverage report
cargo tarpaulin --lib --bins --out Html

# View coverage
open tarpaulin-report.html
```

### Coverage Exemptions

Some code legitimately cannot be tested:
- Platform-specific code on other platforms
- Code requiring specific hardware/OS features
- Recovery paths for catastrophic failures

Document exemptions with comments:
```rust
// COVERAGE: Cannot test on non-Windows platforms
#[cfg(target_os = "windows")]
fn windows_specific_function() {
    // ...
}
```

## Test Categories

### 1. Unit Tests (Inline)

Located within source files, these test individual functions and modules in isolation.

**Example**:
```rust
#[test]
fn test_parse_version() {
    assert_eq!(parse_version("1.2.3"), Some(Version::new(1, 2, 3)));
    assert_eq!(parse_version("invalid"), None);
}
```

### 2. Integration Tests

Located in the `tests/` directory, these test larger workflows and module interactions.

**Example**:
```rust
// tests/install_flow_tests.rs
#[test]
fn test_complete_installation_flow() {
    let installer = InstallCommand::new(false);
    // Test full installation workflow
}
```

### 3. E2E Tests (Planned)

Will test the actual CLI binary with real commands and filesystem operations.

## Shared Test Utilities

### Mock Builders

Use the mock builders from `test_utils::mocks` for consistent test data:

```rust
use mcp_helper::test_utils::mocks::*;

let server = MockServerBuilder::new("test-server")
    .with_dependency(Dependency::NodeJs { min_version: Some("18.0.0") })
    .with_config_validator(|config| {
        // Custom validation logic
        Ok(())
    })
    .build();
```

### Fixtures

Use fixtures for common test data:

```rust
use mcp_helper::test_utils::fixtures::*;

let config = sample_server_config();
let instructions = sample_install_instructions();
```

### Custom Assertions

Use custom assertions for clearer test failures:

```rust
use mcp_helper::test_utils::assertions::*;

assert_file_exists("/tmp/test.txt");
assert_contains_all(output, &["expected", "strings"]);
assert_paths_equal(path1, path2); // Handles platform differences
```

## Platform-Specific Testing

### Conditional Compilation

Use `#[cfg]` attributes for platform-specific tests:

```rust
#[test]
#[cfg(target_os = "windows")]
fn test_windows_path_handling() {
    // Windows-specific test
}

#[test]
#[cfg(unix)]
fn test_unix_permissions() {
    // Unix-specific test
}
```

### Cross-Platform Path Testing

Always test path handling on all platforms:

```rust
#[test]
fn test_cross_platform_paths() {
    let path = if cfg!(windows) {
        "C:\\Users\\test\\file.txt"
    } else {
        "/home/test/file.txt"
    };
    
    // Test normalization
    assert_eq!(normalize_path(path), expected);
}
```

## Mocking Strategy

### When to Mock

- External dependencies (network, filesystem)
- Complex trait implementations
- Time-sensitive operations
- Platform-specific APIs

### Mock Complexity Guidelines

Keep mocks simple:
- ✅ Return fixed values
- ✅ Track method calls
- ✅ Simulate specific scenarios
- ❌ Complex business logic
- ❌ Stateful behavior beyond basics

If mocks become complex, consider refactoring the code under test.

## Best Practices

### 1. Test Naming

Use descriptive names that explain the scenario:
```rust
#[test]
fn test_parse_npm_package_with_scoped_name_and_version() {
    // Clear what this tests
}
```

### 2. Arrange-Act-Assert

Structure tests clearly:
```rust
#[test]
fn test_server_installation() {
    // Arrange
    let server = create_test_server();
    let config = HashMap::new();
    
    // Act
    let result = server.install(&config);
    
    // Assert
    assert!(result.is_ok());
    assert_file_exists("/expected/path");
}
```

### 3. Test Data Builders

Use builders for complex test data:
```rust
let error = ErrorBuilder::missing_dependency("Node.js")
    .version("18.0.0")
    .instructions(install_instructions)
    .build();
```

### 4. Error Testing

Always test error paths:
```rust
#[test]
fn test_invalid_config() {
    let result = validate_config(&invalid_config);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("expected message"));
}
```

### 5. Snapshot Testing

For complex outputs, consider snapshot testing:
```rust
#[test]
fn test_error_display() {
    let error = create_complex_error();
    let output = format!("{}", error);
    
    // Manually verify output matches expected format
    assert!(output.contains("✗ Error"));
    assert!(output.contains("→ Suggestion"));
}
```

## Common Patterns

### Testing Builders

```rust
mod builder_tests {
    use super::*;
    
    #[test]
    fn test_minimal_build() {
        let obj = Builder::new("name").build();
        assert_eq!(obj.name(), "name");
    }
    
    #[test]
    fn test_full_build() {
        let obj = Builder::new("name")
            .with_option("value")
            .with_flag(true)
            .build();
        // Test all fields
    }
}
```

### Testing Trait Implementations

```rust
#[test]
fn test_trait_implementation() {
    let mock = MockImplementation::new();
    
    // Test each trait method
    assert_eq!(mock.method1(), expected);
    assert!(mock.method2(input).is_ok());
}
```

### Testing Async Code

```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

## Troubleshooting

### Common Issues

1. **Coverage Not Updating**
   - Ensure tests are in `#[cfg(test)]` modules within source files
   - Run `cargo clean` before regenerating coverage

2. **Platform-Specific Failures**
   - Check `#[cfg]` attributes
   - Ensure mock filesystem paths work on all platforms

3. **Flaky Tests**
   - Remove time dependencies
   - Use deterministic test data
   - Mock external services

### Debugging Tests

```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run tests in single thread
cargo test -- --test-threads=1

# Show test timings
cargo test -- --show-output
```

## Contributing

When adding new features:

1. Write tests first (TDD)
2. Follow the inline test organization pattern
3. Use existing test utilities where possible
4. Ensure all platforms are covered
5. Run coverage to verify >80% coverage
6. Document any coverage exemptions

## Future Improvements

- [x] Add property-based testing with `proptest` ✅
- [x] Implement comprehensive E2E test suite ✅
- [x] Add performance benchmarks ✅
- [ ] Create test data generators
- [ ] Add mutation testing
- [ ] Implement fuzz testing

## Related Documentation

- [Test Organization Checklist](./test-organization-checklist.md) - Checklist for test structure
- [Test Coverage Improvement Plan](./test-coverage-improvement-plan.md) - Coverage improvement strategy
- [Property Testing Implementation](./property-testing-implementation.md) - Property-based testing guide
- [Performance Benchmarks](./performance-benchmarks.md) - Performance testing documentation
- [Test Strategy Validation Report](./test-strategy-validation-report.md) - Implementation results

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [mockall](https://docs.rs/mockall/latest/mockall/)
- [proptest](https://github.com/proptest-rs/proptest)
- [criterion](https://bheisler.github.io/criterion.rs/book/)