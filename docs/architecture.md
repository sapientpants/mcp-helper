# MCP Helper Architecture

> **Note**: This document describes the technical architecture of MCP Helper.
> - For user-facing features, see [features.md](features.md)
> - For implementation timeline, see [plan.md](plan.md)

## Core Architecture Principles

1. **Platform Abstraction Layer** - All platform-specific logic isolated in traits
2. **Modular Design** - Each major feature in its own module with clear interfaces
3. **Configuration-Driven** - Use serde for JSON/TOML config management
4. **Error Handling** - Rich error types with context and recovery suggestions
5. **Testability** - Dependency injection and trait-based design for easy mocking

### Proposed Module Structure

```
src/
├── main.rs                    # CLI entry point (existing)
├── lib.rs                     # Library root
├── runner.rs                  # (existing, to be enhanced)
├── platform/                  # Platform abstraction layer
│   ├── mod.rs                # Platform trait definition
│   ├── windows.rs            # Windows-specific implementations
│   ├── macos.rs              # macOS-specific implementations
│   └── linux.rs              # Linux-specific implementations
├── config/                    # Configuration management
│   ├── mod.rs                # Config traits and types
│   ├── locations.rs          # Platform-specific config paths
│   ├── reader.rs             # Config file reading
│   └── writer.rs             # Config file writing
├── package/                   # Package management
│   ├── mod.rs                # Package manager abstraction
│   ├── npm.rs                # NPM-specific logic
│   ├── yarn.rs               # Yarn support
│   └── pnpm.rs               # pnpm support
├── environment/               # Environment management
│   ├── mod.rs                # Environment utilities
│   ├── path.rs               # PATH management
│   └── variables.rs          # Environment variable handling
├── diagnostics/               # Doctor/verification features
│   ├── mod.rs                # Diagnostic framework
│   ├── checks.rs             # Individual diagnostic checks
│   └── report.rs             # Diagnostic reporting
├── ide/                       # IDE integration
│   ├── mod.rs                # IDE abstraction
│   ├── vscode.rs             # VS Code config generation
│   └── intellij.rs           # IntelliJ config generation
└── utils/                     # Shared utilities
    ├── mod.rs
    ├── paths.rs              # Path normalization utilities
    └── shell.rs              # Shell command utilities
```

### Key Components

#### 1. Platform Abstraction Layer
```rust
trait PlatformOperations {
    fn get_config_dir(&self) -> PathBuf;
    fn normalize_path(&self, path: &str) -> String;
    fn get_shell_command(&self, cmd: &str) -> Command;
    fn get_env_vars(&self) -> HashMap<String, String>;
}
```

#### 2. Configuration Management
- Use `serde` and `serde_json` for config serialization
- Platform-aware config file locations
- Support for Claude, VS Code, and other MCP clients
- Migration utilities for config format changes

#### 3. Package Management Abstraction
```rust
trait PackageManager {
    fn install(&self, package: &str) -> Result<()>;
    fn list_installed(&self) -> Result<Vec<Package>>;
    fn run(&self, package: &str, args: &[String]) -> Result<()>;
}
```

#### 4. Diagnostic System
- Modular health checks
- Rich error reporting with fix suggestions
- Platform-specific issue detection
- Dependency verification

#### 5. Path and Environment Utilities
- Bidirectional path conversion
- Environment variable management
- Shell-aware command execution

### Dependencies to Add
```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
directories = "5.0"  # Cross-platform config/data directories
thiserror = "2.0"    # Better error handling
tracing = "0.1"      # Structured logging
dialoguer = "0.11"   # Interactive prompts
indicatif = "0.18"   # Progress bars
```

### Platform-Specific Config Locations

The configuration system must handle platform-specific paths:
- Windows: `%APPDATA%\Claude\`
- macOS: `~/Library/Application Support/Claude/`
- Linux: `~/.config/Claude/`

### Testing Strategy

- Unit tests for each module with >80% coverage target
- Integration tests for platform-specific behavior
- Mock implementations for external dependencies
- Property-based testing for path normalization
- Platform matrix testing:
  - Windows 10/11 (cmd, PowerShell, Git Bash)
  - macOS 12+ (Intel and Apple Silicon)
  - Ubuntu 20.04+ and other Linux distributions
