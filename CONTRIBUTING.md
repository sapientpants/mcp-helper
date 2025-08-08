# Contributing to MCP Helper

Thank you for contributing to MCP Helper! This guide will help you get started with the development process.

## Development Setup

### Prerequisites

- Rust 1.70+ (latest stable recommended)
- Git
- Platform-specific dependencies:
  - **Windows**: Visual Studio Build Tools or Visual Studio Community
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: build-essential package

### Getting Started

1. Fork and clone the repository:
   ```bash
   git clone https://github.com/your-username/mcp-helper.git
   cd mcp-helper
   ```

2. Install dependencies and run tests:
   ```bash
   cargo build
   cargo test
   ```

3. Verify pre-commit checks pass:
   ```bash
   make pre-commit
   ```

## Development Workflow

### Make Commands

We use Make for convenient development commands:

```bash
# Development
make dev          # Full development check (fmt + lint + build + test)
make build        # Build debug binary
make test         # Run all tests
make lint         # Run clippy linter
make fmt          # Format code

# Pre-commit checks
make pre-commit   # Run all pre-commit checks
make ci           # Run CI checks locally

# Testing
make test-unit         # Run unit tests only
make test-integration  # Run integration tests only
make coverage          # Generate coverage report
```

### Code Style

- Run `make fmt` before committing
- Follow Rust naming conventions
- Use meaningful variable and function names
- Add documentation for public APIs
- Keep functions focused and small

## Testing Requirements

### Overview

MCP Helper has a comprehensive testing strategy with specific requirements due to coverage tool constraints. Please read the [Testing Guide](docs/testing-guide.md) for complete details.

### Key Requirements

1. **Coverage Target**: >80% overall, >90% for critical paths
2. **Inline Tests**: Unit tests must be in `#[cfg(test)]` modules within source files
3. **Test Organization**: Follow the established submodule pattern
4. **Platform Testing**: Test all supported platforms (Windows, macOS, Linux)

### Test Organization Pattern

All inline test modules should follow this structure:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    mod unit {
        use super::*;
        
        mod feature_name {
            use super::*;
            
            #[test]
            fn descriptive_test_name() {
                // Test implementation
            }
        }
    }
}
```

### Using Test Utilities

Use shared test utilities to reduce duplication:

```rust
use crate::test_utils::{mocks::*, fixtures::*, assertions::*};

// Use builders for complex test data
let server = MockServerBuilder::new("test-server")
    .with_dependency(Dependency::NodeJs { min_version: None })
    .build();

// Use fixtures for common data
let config = sample_server_config();

// Use custom assertions
assert_file_exists(path);
assert_contains_all(output, &["expected", "text"]);
```

### Before Submitting Tests

Use the [Test Organization Checklist](docs/test-organization-checklist.md) to ensure your tests meet all requirements:

- [ ] Follow the inline test module structure
- [ ] Use descriptive test names
- [ ] Test both success and error paths
- [ ] Use shared test utilities where possible
- [ ] Ensure platform-specific code is tested
- [ ] Run `cargo test` and verify all tests pass
- [ ] Check coverage with `make coverage`

## Architecture Guidelines

### Code Organization

- **src/client/**: MCP client implementations (Claude Desktop, VS Code, etc.)
- **src/server/**: MCP server types (NPM, Docker, Binary, Python)
- **src/deps/**: Dependency checking and installation
- **src/error/**: Error types and user-friendly error handling
- **src/install.rs**: Main installation command logic
- **src/runner.rs**: Core server execution logic

### Design Principles

1. **Cross-Platform First**: All code must work on Windows, macOS, and Linux
2. **User-Friendly Errors**: Every error should be actionable
3. **Trait-Based Design**: Use traits for extensibility
4. **Security-Conscious**: Validate all inputs and external data
5. **Performance**: Sub-100ms startup time, minimal memory usage

### Error Handling

- Use the `McpError` type for user-facing errors
- Provide actionable error messages with solutions
- Include context for debugging when possible
- Use the `ErrorBuilder` for complex error construction

Example:
```rust
return Err(ErrorBuilder::missing_dependency("Node.js")
    .version("18.0.0")
    .instructions(get_node_install_instructions())
    .build());
```

## Pull Request Process

### Before Opening a PR

1. **Test Your Changes**:
   ```bash
   make dev          # Run all checks
   make coverage     # Verify coverage
   ```

2. **Check Platform Compatibility**:
   - Windows-specific code uses appropriate APIs
   - Path handling works cross-platform
   - Shell commands are platform-aware

3. **Update Documentation**:
   - Update CLAUDE.md if changing capabilities
   - Add inline documentation for public APIs
   - Update relevant guides if changing workflows

### PR Requirements

- [ ] All tests pass (`cargo test`)
- [ ] Code is formatted (`make fmt`)
- [ ] No clippy warnings (`make lint`)
- [ ] Coverage is maintained or improved
- [ ] Changes are tested on multiple platforms (CI will verify)
- [ ] Breaking changes are documented
- [ ] CLAUDE.md is updated if capabilities change

### PR Description Template

```markdown
## Summary
Brief description of changes and motivation.

## Changes Made
- List of specific changes
- Any new features or capabilities
- Bug fixes

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated  
- [ ] Manual testing performed
- [ ] Coverage verified

## Platform Testing
- [ ] Tested on Windows
- [ ] Tested on macOS
- [ ] Tested on Linux (or CI passes)

## Breaking Changes
None / Description of breaking changes

## Documentation
- [ ] CLAUDE.md updated (if needed)
- [ ] Inline documentation added
- [ ] README updated (if needed)
```

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - New features
- `fix:` - Bug fixes  
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, missing semicolons, etc.)
- `refactor:` - Code refactoring without behavior changes
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks, dependency updates

Examples:
```
feat: add Docker server support
fix: handle NPM scoped packages correctly
docs: update installation guide
test: add cross-platform path tests
```

## Release Process

### Version Bumping

- Follow [Semantic Versioning](https://semver.org/)
- Update version in `Cargo.toml`
- Update CHANGELOG.md
- Tag releases with `v<version>` (e.g., `v0.2.0`)

### Release Checklist

- [ ] All tests pass on all platforms
- [ ] Documentation is up to date
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Performance benchmarks run
- [ ] Security review completed (if applicable)

## Getting Help

### Resources

- [Testing Guide](docs/testing-guide.md) - Comprehensive testing documentation
- [Test Organization Checklist](docs/test-organization-checklist.md) - Quick reference
- [Architecture Documentation](docs/architecture.md) - System design overview
- [CLAUDE.md](CLAUDE.md) - Project context and capabilities

### Communication

- Open an issue for bugs or feature requests
- Use discussions for design questions
- Tag maintainers for urgent issues
- Be respectful and constructive in all interactions

### Common Issues

1. **Tests failing on Windows**: Check path separator handling
2. **Coverage not updating**: Ensure tests are in `#[cfg(test)]` modules
3. **Clippy warnings**: Run `make lint` and fix issues
4. **Platform differences**: Use `#[cfg(target_os = "...")]` attributes

## Code of Conduct

Be respectful, inclusive, and constructive. We're building something useful together!

---

Thank you for contributing to MCP Helper! Your efforts help make MCP servers more accessible to everyone.