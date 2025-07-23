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
- `make lint` - Run clippy linter
- `make check` - Run cargo check
- `make pre-commit` - Run all pre-commit checks (fmt-check + lint + quick-test)

### Development Workflow
- `make dev` - Run full development checks (fmt + lint + build + test)
- `make clean` - Remove build artifacts

## Code Architecture

### Module Structure
- `src/main.rs` - CLI entry point using clap for argument parsing
- `src/lib.rs` - Library root, re-exports public APIs
- `src/runner.rs` - Core server runner with platform abstraction

### Key Components

**Platform Abstraction**: The `Platform` enum handles OS-specific differences:
- Windows: Uses `cmd.exe` and `npx.cmd`
- macOS/Linux: Uses standard shell and `npx`

**ServerRunner**: Main execution logic that:
- Detects the current platform
- Resolves server paths (npm packages or local files)
- Normalizes path separators for cross-platform compatibility
- Executes servers with proper error handling

### Implementation Phases (from docs/plan.md)
- Phase 1: Core Runner ✅ COMPLETE - Basic `mcp run` command
- Phase 2: Installation & Setup 📦 - `mcp install` and `mcp setup`
- Phase 3: Configuration Management ⚙️ - Config CRUD operations
- Phase 4: Diagnostics & Polish 🏥 - `mcp doctor` and auto-fixes
- Phase 5: Enhanced Features 🚀 - Path conversion, env management

### Testing Approach
- Platform-specific behavior tested with mocks
- Integration tests in `tests/cli_tests.rs`
- Target: >80% code coverage
- CI runs tests on Windows, macOS, and Linux

## Important Context

### Platform-Specific Paths
- Windows: `%APPDATA%\Claude\`
- macOS: `~/Library/Application Support/Claude/`
- Linux: `~/.config/Claude/`

### Error Handling Philosophy
Every error should be actionable - tell users exactly how to fix the issue. Use the `colored` crate for clear, readable error messages.

### Git Hooks
Pre-commit hooks are configured via rusty-hook to run formatting and linting checks automatically.

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

## Working with MCP Helper

When implementing new features:
1. Follow the phased approach outlined in docs/plan.md
2. Maintain cross-platform compatibility - platform differences are bugs
3. Write comprehensive tests, especially for Windows-specific behavior
4. Keep error messages actionable and helpful
5. Use the existing platform abstraction patterns

The project uses standard Rust tooling with Make for convenience. All CI checks can be run locally with `make ci`.