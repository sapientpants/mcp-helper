# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MCP Helper is a cross-platform configuration and launcher tool written in Rust that eliminates compatibility issues when working with Model Context Protocol (MCP) servers. It focuses on configuring MCP servers in client applications (Claude Desktop, VS Code, etc.) and ensuring they run correctly across Windows, macOS, and Linux.

**Key Design Principle**: MCP Helper is a configuration tool, NOT a package manager. It leverages existing tools (npx, docker) for dependency management rather than reimplementing them.

## Development Commands

### Building
- `make build` - Build debug binary
- `make build-release` - Build optimized release binary
- `cargo build` - Direct cargo build

### Testing
- `make test` - Run all tests (unit + integration)
- `make test-unit` - Run unit tests only
- `make test-integration` - Run integration tests only
- `cargo test -- --nocapture` - Run tests with output

### Code Quality
- `make fmt` - Format code
- `make fmt-check` - Check formatting without changes
- `make lint` - Run clippy linter on library code
- `make lint-all` - Run clippy linter on all targets including tests
- `make check` - Run cargo check
- `make pre-commit` - Run all pre-commit checks (fmt-check + lint-all + quick-test)

### Development Workflow
- `make dev` - Run full development checks (fmt + lint + build + test)
- `make clean` - Remove build artifacts

## Code Architecture

### Module Structure
- `src/main.rs` - CLI entry point using clap for argument parsing
- `src/lib.rs` - Library root, re-exports public APIs
- `src/add.rs` - Unified add command for configuring servers
- `src/client/` - MCP client implementations (Claude Desktop, etc.)
- `src/server/` - MCP server types and implementations (NPM, Binary, etc.)
- `src/deps/` - Dependency checking and installation instructions
- `src/install.rs` - Legacy install command (deprecated, redirects to add)
- `src/config_commands.rs` - List and remove commands (moved to top-level)
- `src/setup.rs` - Environment setup and verification
- `src/doctor.rs` - Diagnostics and troubleshooting

### Key Components

**Platform Abstraction**: The `Platform` enum handles OS-specific differences:
- Windows: Uses `cmd.exe` and `npx.cmd`
- macOS/Linux: Uses standard shell and `npx`

**Add Command**: Unified command for configuring servers:
- Auto-detects server type (NPM, Docker, Python, Binary)
- Handles platform-specific commands (npx vs npx.cmd on Windows)
- Supports both interactive and non-interactive modes
- Can add to multiple MCP clients simultaneously

**McpClient Trait**: Abstraction for different MCP clients:
- Provides unified interface for client configuration
- Supports Claude Desktop, with extensibility for Cursor, VS Code, etc.
- Handles platform-specific config paths and JSON preservation

**McpServer Trait**: Abstraction for different server types:
- NPM packages (with scoped package and version support)
- Binary releases (planned)
- Python scripts (planned)
- Docker containers (planned)

**Dependency Checking**: Validates required tools are available:
- Verifies Node.js/npm/npx presence for npm-based servers
- Checks Docker availability for container-based servers
- Provides guidance when tools are missing
- Does NOT install dependencies (users should use official installers)

**Install Command**: Configures servers in MCP clients:
- Auto-detects installed MCP clients (Claude Desktop, VS Code, etc.)
- Validates server exists in registry (npm, Docker Hub, etc.)
- Prompts for server-specific configuration (API keys, paths, etc.)
- Updates client configs to use npx/docker commands (which handle actual installation)

### Implementation Progress

**Completed Features:**
- ✅ Core architecture (client, server, deps modules)
- ✅ Claude Desktop client support
- ✅ NPM server configuration (handles npx vs npx.cmd at config time)
- ✅ Dependency checking (verifies tools are available)
- ✅ `mcp add` - Unified server configuration command
- ✅ `mcp list` - List all configured servers
- ✅ `mcp remove` - Remove servers from configuration
- ✅ `mcp setup` - Environment verification
- ✅ `mcp doctor` - Comprehensive diagnostics

**In Progress:**
- 📦 Additional client support (Cursor, VS Code, Windsurf)
- 📦 Binary server downloads
- 📦 Python server support

### Implementation Phases (from docs/plan.md)
- Phase 1: Core Commands ✅ COMPLETE - `mcp add/list/remove`
- Phase 2: Environment & Diagnostics ✅ COMPLETE - `mcp setup` and `mcp doctor`
- Phase 3: Enhanced Server Support 🚧 IN PROGRESS - Binary, Python, Docker servers
- Phase 4: Additional Clients 📦 - Cursor, VS Code, Windsurf support
- Phase 5: Polish & Features 🚀 - Shell completions, batch operations

### Testing Approach
- Platform-specific behavior tested with mocks
- All tests moved to `tests/` directory for better organization
- Integration tests for CLI, clients, servers, dependencies, and install command
- Target: >80% code coverage (currently achieving 100% for new modules)
- CI runs tests on Windows, macOS, and Linux

### Dependencies Added
- `serde` & `serde_json` - JSON serialization with order preservation
- `directories` - Cross-platform user directories
- `tempfile` - Safe atomic file operations
- `semver` - Semantic version parsing and comparison
- `dialoguer` - Interactive CLI prompts and multi-select

## Architectural Philosophy

MCP Helper follows the Unix philosophy of "do one thing well":
- **We configure, not install** - npx/docker handle package management
- **We validate, not fix** - Users install Node.js/Docker through official channels
- **We detect, not assume** - Auto-detect clients and validate environments
- **We simplify, not complicate** - Leverage existing tools rather than reinventing

The main value propositions are:
1. **Cross-platform compatibility** - Handle npx vs npx.cmd, path separators, etc.
2. **Configuration management** - Update complex JSON configs safely
3. **Interactive setup** - Guide users through configuration
4. **Environment validation** - Ensure required tools are available

## Important Context

### Platform-Specific Paths
- Windows: `%APPDATA%\Claude\`
- macOS: `~/Library/Application Support/Claude/`
- Linux: `~/.config/Claude/`

### Error Handling Philosophy
Every error should be actionable - tell users exactly how to fix the issue. Use the `colored` crate for clear, readable error messages.

### Git Hooks & Trunk-Based Development
Pre-commit hooks are configured via rusty-hook to run formatting and linting checks automatically. The project uses trunk-based development, so the pre-commit hook runs `lint-all` to catch issues in both library and test code before they reach main. This ensures CI stays green despite the ~1 second overhead compared to linting library code only.

### Performance Goals
- Startup time < 100ms
- No background processes
- Minimal memory footprint
- Fast CI builds (~1-2 minutes)

### Future Architecture (from docs/architecture.md)
The codebase will expand to include:
- `platform/` - Platform-specific implementations
- `config/` - Configuration management
- `package/` - Package manager abstractions
- `environment/` - PATH and env var handling
- `diagnostics/` - Doctor command implementation
- `ide/` - IDE configuration generators

## Current CLI Commands

MCP Helper focuses on configuration management with these simplified commands:

### Core Commands
- `mcp add <server>` - Add a server to MCP client configuration
  - Auto-detects server type (NPM, Docker, Python, Binary)
  - Handles platform-specific commands (npx vs npx.cmd on Windows)
  - Supports interactive configuration for environment variables
  - Can add to multiple MCP clients simultaneously
- `mcp list` - List all configured servers across all clients
- `mcp remove <server>` - Remove a server from configuration
  - `--all` flag to remove from all clients at once
- `mcp setup` - One-time environment setup and verification
- `mcp doctor` - Diagnose and fix common MCP issues

### Deprecated Commands (still work but show warnings)
- `mcp install` - Use `mcp add` instead
- `mcp config add/list/remove` - Use top-level commands instead

### Command Examples
```bash
# Add servers (auto-detects type)
mcp add @modelcontextprotocol/server-filesystem
mcp add @anthropic/mcp-server-slack
mcp add docker:nginx:alpine
mcp add https://github.com/org/mcp-server/releases/latest

# List configured servers
mcp list

# Remove a server
mcp remove server-filesystem
mcp remove server-slack --all  # Remove from all clients

# Check environment
mcp setup
mcp doctor
```

## Working with MCP Helper

When implementing new features:
1. Follow the phased approach outlined in docs/plan.md
2. Maintain cross-platform compatibility - platform differences are bugs
3. Write comprehensive tests, especially for Windows-specific behavior
4. Keep error messages actionable and helpful
5. Use the existing platform abstraction patterns
6. Use trait-based abstractions for extensibility (McpClient, McpServer, DependencyChecker)
7. Prefer interactive prompts over command-line flags for better UX

The project uses standard Rust tooling with Make for convenience. All CI checks can be run locally with `make ci`.