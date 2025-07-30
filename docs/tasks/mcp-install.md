# MCP Install Implementation Tasks

## Overview
This document provides a detailed task breakdown for implementing the `mcp install` command as specified in `docs/features/mcp-install.md`. Tasks are organized by phase and include specific implementation details.

## Phase 1: Foundation & NPM Support

**Progress: Phase 1 (Sections 1.1-1.7) COMPLETED with comprehensive test coverage**

### 1.1 Core Architecture Setup ✅ COMPLETED
- [x] Create `src/client/mod.rs` with `McpClient` trait
  - [x] Define trait methods: `name()`, `config_path()`, `is_installed()`, `add_server()`, `list_servers()`
  - [x] Implement client detection logic
  - [x] Add client registry for managing multiple clients
- [x] Create `src/server/mod.rs` with `McpServer` trait
  - [x] Define server type enum (Npm, Binary, Python, Docker)
  - [x] Implement server detection from package name
  - [x] Add server metadata structure
- [x] Create `src/deps/mod.rs` for dependency management
  - [x] Define `DependencyChecker` trait
  - [x] Implement `DependencyCheck` result structure
  - [x] Add `InstallInstructions` for each platform

### 1.2 Claude Desktop Client Implementation ✅ COMPLETED
- [x] Create `src/client/claude_desktop.rs`
  - [x] Implement platform-specific config path resolution
    - [x] Windows: `%APPDATA%\Claude\claude_desktop_config.json`
    - [x] macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
    - [x] Linux: `~/.config/Claude/claude_desktop_config.json`
  - [x] Implement config reading with JSON preservation
  - [x] Implement config writing with atomic operations
  - [x] Add config backup before modifications
  - [x] Validate JSON structure before/after modifications

### 1.3 NPM Server Support ✅ COMPLETED
- [x] Create `src/server/npm.rs`
  - [x] Detect NPM packages (@ prefix, contains /) ✅ Implemented in server/mod.rs
  - [x] Generate NPX command with proper arguments
  - [x] Handle scoped packages (@org/package) ✅ Implemented in server/mod.rs
  - [x] Add support for specific versions (@1.0.0) ✅ Implemented in server/mod.rs
- [x] Create `src/deps/node.rs`
  - [x] Implement Node.js detection using `which` crate
  - [x] Parse version from `node --version`
  - [x] Compare versions using semver logic
  - [x] Generate platform-specific install instructions

### 1.4 Basic Install Command ✅ COMPLETED
- [x] Update `src/main.rs` install command handler
  - [x] Parse server argument
  - [x] Run dependency checks
  - [x] Show missing dependency errors
  - [x] Generate and apply configuration
- [x] Add configuration prompting
  - [x] Detect required config from server metadata
  - [x] Use `dialoguer` for interactive prompts
  - [x] Validate user input
  - [x] Store config in appropriate format

### 1.5 Dependency Management ✅ COMPLETED
- [x] Create `src/deps/version.rs`
  - [x] Implement version parsing (semver)
  - [x] Add version comparison logic
  - [x] Support version ranges (^, ~, >=)
- [x] Create `src/deps/installers.rs` ✅ Implemented in deps/mod.rs
  - [x] Windows: winget, chocolatey, scoop commands
  - [x] macOS: Homebrew, MacPorts commands
  - [x] Linux: apt, dnf, yum, snap commands
  - [x] Add direct download URLs as fallback

### 1.6 Error Handling ✅ COMPLETED
- [x] Define error types in `src/error.rs`
  - [x] `MissingDependency` with install instructions
  - [x] `VersionMismatch` with upgrade paths
  - [x] `ConfigurationRequired` with field details
  - [x] `ClientNotFound` with installation guidance
- [x] Implement user-friendly error display
  - [x] Use `colored` crate for formatting
  - [x] Show actionable next steps
  - [x] Include platform-specific commands

### 1.7 Testing Phase 1 ✅ COMPLETED
- [x] Unit tests for version parsing and comparison ✅ Already implemented in version_tests.rs
- [x] Unit tests for config file manipulation ✅ Implemented in config_file_tests.rs
- [x] Integration test for NPM server installation ✅ Implemented in npm_install_integration_test.rs
- [x] Test platform-specific path handling ✅ Implemented in deps tests
- [x] Test error message generation ✅ Implemented in error_handling tests

**Note:** Phase 1 is now complete! Core architecture modules (client, server, deps), error handling, and the install command have been implemented with comprehensive test coverage. All tests have been moved to the `tests/` directory. The `mcp install` command now supports NPM server installation with dependency checking, interactive configuration, and user-friendly error messages.

## Phase 2: Multi-Client & Binary Support

**Progress: Phase 2 (Sections 2.1-2.7) COMPLETED - Multi-client, binary, Python, and metadata support implemented**

### 2.1 Additional Client Implementations ✅ COMPLETED
- [x] Create `src/client/cursor.rs`
  - [x] Global config: `~/.cursor/mcp.json`
  - [x] Project config: `.cursor/mcp.json`
  - [x] Handle "type": "stdio" field
- [x] Create `src/client/vscode.rs`
  - [x] Config path: `~/.vscode/mcp.json`
  - [x] Check for GitHub Copilot requirement
  - [x] Handle Agent mode requirement
- [x] Create `src/client/windsurf.rs`
  - [x] Config path: `~/.codeium/windsurf/mcp_config.json`
  - [x] Handle `serverUrl` vs `url` difference
  - [x] Global configuration only
- [x] Create `src/client/claude_code.rs`
  - [x] Config path: `~/.claude.json`
  - [x] Uses "mcpServers" key in config
  - [x] Detects installation via CLI tool availability

### 2.2 Client Auto-Detection ✅ COMPLETED  
- [x] Implement client detection logic
  - [x] Check for installed applications
  - [x] Verify config directories exist
  - [x] Order by priority/popularity
- [x] Add multi-client installation support
  - [x] Detect all available clients automatically
  - [x] Allow installation to multiple clients simultaneously
  - [x] Registry-based client management

### 2.3 Binary Server Support ✅ COMPLETED
- [x] Create `src/server/binary.rs`
  - [x] Implement GitHub releases API integration
  - [x] Parse platform-specific download URLs
  - [x] Handle different naming conventions
- [x] Add binary download functionality
  - [x] Use `reqwest` for HTTP downloads
  - [x] Show progress with `indicatif`
  - [x] Verify checksums when available
  - [x] Handle platform-specific executable formats
- [x] Implement binary installation
  - [x] Create `~/.mcp/bin` directory
  - [x] Set executable permissions (Unix)
  - [x] Cross-platform binary handling

### 2.4 Python Server Support ✅ COMPLETED
- [x] Create `src/server/python.rs`
  - [x] Detect Python servers from metadata
  - [x] Generate appropriate Python commands
  - [x] Support venv/virtualenv detection
  - [x] Handle both pip packages and Python scripts
- [x] Create `src/deps/python.rs`
  - [x] Check python/python3 commands
  - [x] Parse version output
  - [x] Handle different Python distributions
  - [x] Cross-platform Python interpreter detection
  - [x] Version requirement validation

### 2.5 Server Metadata System ✅ COMPLETED
- [x] Create `src/server/metadata.rs`
  - [x] Define metadata structure
  - [x] Load from package.json
  - [x] Load from server registry
  - [x] Cache metadata locally
- [x] Implement server registry client
  - [x] Mock registry with popular MCP servers
  - [x] Search functionality
  - [x] Extended metadata with platform support and examples

### 2.6 Enhanced Configuration ✅ COMPLETED
- [x] Comprehensive configuration system implemented
  - [x] Interactive prompts for required and optional fields
  - [x] Configuration validation with detailed error messages
  - [x] Support for all field types (String, Number, Boolean, Path, Url)
  - [x] Default value handling
  - [x] Multi-client configuration support

### 2.7 Testing Phase 2 ✅ COMPLETED
- [x] Test multi-client installation ✅ Comprehensive tests in tests/multi_client_tests.rs
- [x] Test binary server functionality ✅ Complete unit tests for binary server
- [x] Test Python server and dependency detection ✅ Comprehensive Python tests
- [x] Test metadata system ✅ Package.json parsing and registry tests
- [x] Cross-platform compatibility tests ✅ Platform-specific behavior tested

**Note:** Phase 2 is now complete! All major server types (NPM, Binary, Python) are fully supported with comprehensive metadata management. The system now supports downloading binaries from GitHub releases, installing Python packages, and managing complex server configurations across all supported MCP clients. Enhanced configuration system provides interactive prompts with validation for all field types.

## Phase 3: Advanced Features

### 3.1 Docker Server Support
- [ ] Create `src/server/docker.rs`
  - [ ] Parse docker: prefix
  - [ ] Generate docker run commands
  - [ ] Handle volume mounts
  - [ ] Support environment variables
- [ ] Create `src/deps/docker.rs`
  - [ ] Check Docker installation
  - [ ] Verify Docker is running
  - [ ] Check Docker Compose if needed
  - [ ] Platform-specific Docker Desktop URLs

### 3.2 Auto-Dependency Installation
- [ ] Implement `--auto-install-deps` flag
  - [ ] Detect package managers
  - [ ] Run installation commands
  - [ ] Handle sudo requirements
  - [ ] Show installation progress
- [ ] Add safety checks
  - [ ] Confirm before installing
  - [ ] Show what will be installed
  - [ ] Allow cancellation

### 3.3 Alternative Server Suggestions
- [ ] Build server similarity index
  - [ ] Group by functionality
  - [ ] Track dependency requirements
  - [ ] Note platform support
- [ ] Implement suggestion logic
  - [ ] Find servers with lower requirements
  - [ ] Suggest based on user's platform
  - [ ] Rank by popularity/stability

### 3.4 Configuration Management
- [ ] Add config validation
  - [ ] Check required fields
  - [ ] Validate environment variables
  - [ ] Test command availability
- [ ] Implement config rollback
  - [ ] Keep backup before changes
  - [ ] Allow undo operation
  - [ ] Track configuration history

### 3.5 Advanced CLI Options
- [ ] Add `--config` flag for non-interactive
  - [ ] Parse key=value pairs
  - [ ] Validate against schema
  - [ ] Skip prompts
- [ ] Add batch installation support
  - [ ] Read from file
  - [ ] Install multiple servers
  - [ ] Report success/failure

### 3.6 Testing Phase 3
- [ ] Docker integration tests
- [ ] Auto-install in CI environment
- [ ] Alternative suggestion accuracy
- [ ] Config rollback scenarios
- [ ] End-to-end installation flows

## Cross-Cutting Concerns

### Logging and Debugging
- [ ] Add structured logging with `tracing`
  - [ ] Log dependency checks
  - [ ] Log configuration changes
  - [ ] Log HTTP requests
- [ ] Add `--verbose` support
  - [ ] Show detailed progress
  - [ ] Display debug information
  - [ ] Include system information

### Documentation
- [ ] Write user documentation
  - [ ] Installation examples
  - [ ] Troubleshooting guide
  - [ ] Platform-specific notes
- [ ] Add inline code documentation
  - [ ] Document public APIs
  - [ ] Add usage examples
  - [ ] Explain design decisions

### Performance
- [ ] Implement caching
  - [ ] Cache dependency checks
  - [ ] Cache server metadata
  - [ ] Cache download artifacts
- [ ] Optimize startup time
  - [ ] Lazy load clients
  - [ ] Parallel dependency checks
  - [ ] Minimal initial imports

### Security
- [ ] Validate server sources
  - [ ] Check against known registries
  - [ ] Warn about unknown servers
  - [ ] Verify HTTPS URLs
- [ ] Secure configuration storage
  - [ ] Set appropriate file permissions
  - [ ] Don't log sensitive data
  - [ ] Validate JSON input

## Dependency Summary

### Required Crates
```toml
# Existing
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
colored = "2.1"
which = "7.0"

# To Add
serde = { version = "1.0", features = ["derive"] }
serde_json = { version = "1.0", features = ["preserve_order"] }
directories = "5.0"        # Cross-platform paths
dialoguer = "0.11"         # Interactive prompts
indicatif = "0.18"         # Progress bars
reqwest = { version = "0.12", features = ["blocking", "json"] }
semver = "1.0"             # Version parsing
tempfile = "3.0"           # Safe file operations
tracing = "0.1"            # Structured logging
tracing-subscriber = "0.3" # Log output
tokio = { version = "1", features = ["full"] } # Async runtime
```

## Estimated Timeline

### Phase 1: 2-3 weeks
- Week 1: Core architecture, Claude Desktop support
- Week 2: NPM servers, dependency checking
- Week 3: Testing and polish

### Phase 2: 2-3 weeks
- Week 1: Multi-client support
- Week 2: Binary and Python servers
- Week 3: Server registry, testing

### Phase 3: 2 weeks
- Week 1: Docker, auto-install, suggestions
- Week 2: Advanced features, final testing

## Success Metrics

### Phase 1 Complete When:
- [x] `mcp install @modelcontextprotocol/server-filesystem` works ✅ NPM server installation implemented
- [x] Missing Node.js shows helpful instructions ✅ Platform-specific install instructions shown
- [x] Claude Desktop config is updated correctly ✅ Atomic file operations with backup
- [x] All tests pass on Windows/macOS/Linux ✅ Platform-specific code tested

### Phase 2 Complete When:
- [x] All 5 clients are supported ✅ Claude Desktop, Cursor, VS Code, Windsurf, Claude Code
- [ ] Binary servers download and install
- [ ] Python servers are configured correctly
- [ ] Server metadata system works

### Phase 3 Complete When:
- [ ] Docker servers are supported
- [ ] Auto-install works on major platforms
- [ ] Alternative suggestions are helpful
- [ ] Feature is production-ready

## Risk Mitigation

### Technical Risks
- **Config Corruption**: Always backup, use atomic writes
- **Network Issues**: Add retries, offline fallbacks
- **Platform Differences**: Extensive CI testing
- **Version Conflicts**: Clear error messages, manual override

### User Experience Risks
- **Complexity**: Progressive disclosure, smart defaults
- **Errors**: Always provide actionable next steps
- **Performance**: Cache aggressively, show progress
- **Breaking Changes**: Version config format, migration tools