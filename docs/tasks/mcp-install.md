# MCP Install Implementation Tasks

## Overview
This document provides a detailed task breakdown for implementing the `mcp install` command as specified in `docs/features/mcp-install.md`. Tasks are organized by phase and include specific implementation details.

## Phase 1: Foundation & NPM Support

### 1.1 Core Architecture Setup
- [ ] Create `src/client/mod.rs` with `McpClient` trait
  - [ ] Define trait methods: `name()`, `config_path()`, `is_installed()`, `add_server()`, `list_servers()`
  - [ ] Implement client detection logic
  - [ ] Add client registry for managing multiple clients
- [ ] Create `src/server/mod.rs` with `McpServer` trait
  - [ ] Define server type enum (Npm, Binary, Python, Docker)
  - [ ] Implement server detection from package name
  - [ ] Add server metadata structure
- [ ] Create `src/deps/mod.rs` for dependency management
  - [ ] Define `DependencyChecker` trait
  - [ ] Implement `DependencyCheck` result structure
  - [ ] Add `InstallInstructions` for each platform

### 1.2 Claude Desktop Client Implementation
- [ ] Create `src/client/claude_desktop.rs`
  - [ ] Implement platform-specific config path resolution
    - [ ] Windows: `%APPDATA%\Claude\claude_desktop_config.json`
    - [ ] macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
    - [ ] Linux: `~/.config/Claude/claude_desktop_config.json`
  - [ ] Implement config reading with JSON preservation
  - [ ] Implement config writing with atomic operations
  - [ ] Add config backup before modifications
  - [ ] Validate JSON structure before/after modifications

### 1.3 NPM Server Support
- [ ] Create `src/server/npm.rs`
  - [ ] Detect NPM packages (@ prefix, contains /)
  - [ ] Generate NPX command with proper arguments
  - [ ] Handle scoped packages (@org/package)
  - [ ] Add support for specific versions (@1.0.0)
- [ ] Create `src/deps/node.rs`
  - [ ] Implement Node.js detection using `which` crate
  - [ ] Parse version from `node --version`
  - [ ] Compare versions using semver logic
  - [ ] Generate platform-specific install instructions

### 1.4 Basic Install Command
- [ ] Update `src/main.rs` install command handler
  - [ ] Parse server argument
  - [ ] Run dependency checks
  - [ ] Show missing dependency errors
  - [ ] Generate and apply configuration
- [ ] Add configuration prompting
  - [ ] Detect required config from server metadata
  - [ ] Use `dialoguer` for interactive prompts
  - [ ] Validate user input
  - [ ] Store config in appropriate format

### 1.5 Dependency Management
- [ ] Create `src/deps/version.rs`
  - [ ] Implement version parsing (semver)
  - [ ] Add version comparison logic
  - [ ] Support version ranges (^, ~, >=)
- [ ] Create `src/deps/installers.rs`
  - [ ] Windows: winget, chocolatey, scoop commands
  - [ ] macOS: Homebrew, MacPorts commands
  - [ ] Linux: apt, dnf, yum, snap commands
  - [ ] Add direct download URLs as fallback

### 1.6 Error Handling
- [ ] Define error types in `src/error.rs`
  - [ ] `MissingDependency` with install instructions
  - [ ] `VersionMismatch` with upgrade paths
  - [ ] `ConfigurationRequired` with field details
  - [ ] `ClientNotFound` with installation guidance
- [ ] Implement user-friendly error display
  - [ ] Use `colored` crate for formatting
  - [ ] Show actionable next steps
  - [ ] Include platform-specific commands

### 1.7 Testing Phase 1
- [ ] Unit tests for version parsing and comparison
- [ ] Unit tests for config file manipulation
- [ ] Integration test for NPM server installation
- [ ] Test platform-specific path handling
- [ ] Test error message generation

## Phase 2: Multi-Client & Binary Support

### 2.1 Additional Client Implementations
- [ ] Create `src/client/cursor.rs`
  - [ ] Global config: `~/.cursor/mcp.json`
  - [ ] Project config: `.cursor/mcp.json`
  - [ ] Handle "type": "stdio" field
- [ ] Create `src/client/vscode.rs`
  - [ ] Config path: `~/.vscode/mcp.json`
  - [ ] Check for GitHub Copilot requirement
  - [ ] Handle Agent mode requirement
- [ ] Create `src/client/windsurf.rs`
  - [ ] Config path: `~/.codeium/windsurf/mcp_config.json`
  - [ ] Handle `serverUrl` vs `url` difference
  - [ ] Global configuration only

### 2.2 Client Auto-Detection
- [ ] Implement client detection logic
  - [ ] Check for installed applications
  - [ ] Verify config directories exist
  - [ ] Order by priority/popularity
- [ ] Add `--client` flag support
  - [ ] Parse client selection
  - [ ] Validate client is installed
  - [ ] Allow multiple client selection

### 2.3 Binary Server Support
- [ ] Create `src/server/binary.rs`
  - [ ] Implement GitHub releases API integration
  - [ ] Parse platform-specific download URLs
  - [ ] Handle different naming conventions
- [ ] Add binary download functionality
  - [ ] Use `reqwest` for HTTP downloads
  - [ ] Show progress with `indicatif`
  - [ ] Verify checksums when available
  - [ ] Extract archives (zip, tar.gz)
- [ ] Implement binary installation
  - [ ] Create `~/.mcp/bin` directory
  - [ ] Set executable permissions (Unix)
  - [ ] Update PATH if needed

### 2.4 Python Server Support
- [ ] Create `src/server/python.rs`
  - [ ] Detect Python servers from metadata
  - [ ] Generate appropriate Python commands
  - [ ] Support venv/virtualenv detection
- [ ] Create `src/deps/python.rs`
  - [ ] Check python/python3 commands
  - [ ] Parse version output
  - [ ] Handle different Python distributions
  - [ ] Support pyenv, conda detection

### 2.5 Server Metadata System
- [ ] Create `src/server/metadata.rs`
  - [ ] Define metadata structure
  - [ ] Load from package.json
  - [ ] Load from server registry
  - [ ] Cache metadata locally
- [ ] Implement server registry client
  - [ ] Connect to awesome-mcp-servers data
  - [ ] Search functionality
  - [ ] Offline fallback

### 2.6 Enhanced Configuration
- [ ] Add dry-run support (`--dry-run`)
  - [ ] Show what would be changed
  - [ ] Display generated configs
  - [ ] No filesystem modifications
- [ ] Add force mode (`--force`)
  - [ ] Skip dependency checks
  - [ ] Overwrite existing configs
  - [ ] Proceed despite warnings

### 2.7 Testing Phase 2
- [ ] Test multi-client installation
- [ ] Test binary download and extraction
- [ ] Test Python version detection
- [ ] Mock HTTP requests for registry
- [ ] Cross-platform binary tests

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
- [ ] `mcp install @modelcontextprotocol/server-filesystem` works
- [ ] Missing Node.js shows helpful instructions
- [ ] Claude Desktop config is updated correctly
- [ ] All tests pass on Windows/macOS/Linux

### Phase 2 Complete When:
- [ ] All 4 clients are supported
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