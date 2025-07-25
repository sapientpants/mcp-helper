# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MCP Helper is a cross-platform tool written in Rust that eliminates compatibility issues when working with Model Context Protocol (MCP) servers. It acts as a universal launcher and configuration manager for MCP servers, making them "just work" on Windows, macOS, and Linux.

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
- `src/runner.rs` - Core server runner with platform abstraction
- `src/client/` - MCP client implementations (Claude Desktop, etc.)
- `src/server/` - MCP server types and implementations (NPM, Binary, etc.)
- `src/deps/` - Dependency checking and installation instructions
- `src/install.rs` - Install command implementation with interactive configuration

### Key Components

**Platform Abstraction**: The `Platform` enum handles OS-specific differences:
- Windows: Uses `cmd.exe` and `npx.cmd`
- macOS/Linux: Uses standard shell and `npx`

**ServerRunner**: Main execution logic that:
- Detects the current platform
- Resolves server paths (npm packages or local files)
- Normalizes path separators for cross-platform compatibility
- Executes servers with proper error handling

**McpClient Trait**: Abstraction for different MCP clients:
- Provides unified interface for client configuration
- Supports Claude Desktop, with extensibility for Cursor, VS Code, etc.
- Handles platform-specific config paths and JSON preservation

**McpServer Trait**: Abstraction for different server types:
- NPM packages (with scoped package and version support)
- Binary releases (planned)
- Python scripts (planned)
- Docker containers (planned)

**Dependency Management**: Intelligent dependency checking:
- Detects missing dependencies (Node.js, Python, Docker, etc.)
- Provides platform-specific installation instructions
- Supports version requirements and validation

**Install Command**: Interactive server installation:
- Auto-detects installed MCP clients
- Checks and validates dependencies before installation
- Prompts for required and optional configuration
- Updates client configs with atomic file operations

### Implementation Progress

**Completed Features:**
- ✅ `mcp run` - Cross-platform server execution
- ✅ Core architecture (client, server, deps modules)
- ✅ Claude Desktop client support
- ✅ NPM server support with version handling
- ✅ Dependency checking with install instructions
- ✅ `mcp install` - Interactive server installation (NPM servers)

**In Progress:**
- 📦 Additional client support (Cursor, VS Code, Windsurf)
- 📦 Binary server downloads
- 📦 Python server support

### Implementation Phases (from docs/plan.md)
- Phase 1: Core Runner ✅ COMPLETE - Basic `mcp run` command
- Phase 2: Installation & Setup 🚧 IN PROGRESS - `mcp install` (NPM ✅) and `mcp setup`
- Phase 3: Configuration Management ⚙️ - Config CRUD operations
- Phase 4: Diagnostics & Polish 🏥 - `mcp doctor` and auto-fixes
- Phase 5: Enhanced Features 🚀 - Path conversion, env management

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

## Current Capabilities

The `mcp install` command now supports:
- **NPM Server Installation**: Installs any NPM-based MCP server
  - Handles scoped packages (e.g., `@modelcontextprotocol/server-filesystem`)
  - Supports version specifications (e.g., `express@4.18.0`)
- **Dependency Validation**: Checks for Node.js with version requirements
- **Interactive Configuration**: Prompts for required and optional settings
- **Multi-Client Support**: Can install to multiple MCP clients simultaneously
- **Safe Config Updates**: Atomic writes with automatic backups

Example usage:
```bash
mcp install @modelcontextprotocol/server-filesystem
mcp install @anthropic/mcp-server-slack
mcp install some-mcp-server@1.2.3
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