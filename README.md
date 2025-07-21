# MCP Helper - Make MCP Just Work™

<div align="center">

[![Crates.io](https://img.shields.io/crates/v/mcp-helper.svg)](https://crates.io/crates/mcp-helper)
[![CI](https://github.com/mcp-helper/mcp-helper/actions/workflows/ci.yml/badge.svg)](https://github.com/mcp-helper/mcp-helper/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform Support](https://img.shields.io/badge/platform-windows%20%7C%20macos%20%7C%20linux-lightgrey)](https://github.com/mcp-helper/mcp-helper/releases)

**Cross-platform tooling for Model Context Protocol (MCP) servers**

[Installation](#installation) • [Quick Start](#quick-start) • [Features](#features) • [Contributing](#contributing)

</div>

## What is MCP Helper?

MCP Helper is a developer tool that eliminates cross-platform compatibility issues when working with Model Context Protocol servers. If you've ever struggled with:

- ❌ `npx` not working properly on Windows
- ❌ Path separators breaking your config files
- ❌ Environment variables not propagating to GUI apps
- ❌ "Works on my machine" when sharing MCP servers

Then MCP Helper is for you. **One tool, zero platform headaches.**

## Installation

### Quick Install

**macOS/Linux:**

```bash
curl -fsSL https://get.mcp.dev | sh
```

**Windows (PowerShell):**

```powershell
iwr https://get.mcp.dev/install.ps1 -useb | iex
```

### Package Managers

**Homebrew (macOS/Linux):**

```bash
brew install mcp-helper
```

**Scoop (Windows):**

```powershell
scoop install mcp-helper
```

**Cargo (All platforms):**

```bash
cargo install mcp-helper
```

## Quick Start

```bash
# One-time setup - fixes platform-specific issues
mcp setup

# Install any MCP server (works identically on all platforms)
mcp install server-name

# Run a server without worrying about paths or npx
mcp run server-name
```

That's it. No manual PATH configuration, no debugging Windows-specific issues, no platform-specific documentation needed.

## Features

### 🚀 Universal MCP Launcher

```bash
mcp run my-server    # Works on Windows/Mac/Linux
```

- Handles `npx`/`npx.cmd` detection automatically
- Manages path separators transparently  
- Inherits shell environment on macOS/Linux GUI apps

### 📁 Smart Config Management

```bash
mcp config add my-server      # Adds to the right location
mcp config list               # Shows all configured servers
mcp config remove my-server   # Safely removes server config
```

- Auto-detects Claude config location on each platform
- Atomic writes prevent corruption
- Automatic path normalization for cross-platform configs

### 🔧 Development Environment Normalization

```bash
mcp init                    # Creates cross-platform project
mcp generate-ide-config     # VS Code settings that work everywhere
```

- Handles Node.js version differences
- Generates `.gitignore` with platform-specific exclusions
- Creates IDE configurations that work across teams

### 🏥 Built-in Diagnostics

```bash
mcp doctor
```

```
🔍 Checking MCP environment...
  ✓ Node.js: v20.11.0
  ✗ npx availability: npx not found in PATH on Windows
    → Auto-fix available. Apply? [Y/n] Y
    ✓ Fixed!
  ✓ Config permissions: Read/write access confirmed
  ✓ PATH setup: All required tools accessible

All checks passed! MCP is ready to use.
```

### 🔄 Path and Environment Utilities

```bash
# Set environment variables with platform-specific storage
mcp env set API_KEY=your-secret-key

# Convert paths to platform-appropriate format
mcp path convert ./my/unix/path  # Returns .\my\unix\path on Windows
```

- Handles environment variable storage per platform
- Automatic path conversion for configs
- No more manual escaping or format issues

### ✅ Installation Verification

```bash
# Verify a server meets platform requirements
mcp verify my-server
```

- Checks for platform-specific dependencies
- Validates path requirements
- Ensures compatibility before installation

### 🛡️ What This Does NOT Do

This tool has a focused scope. It does NOT handle:

- ❌ Authentication or authorization
- ❌ Runtime monitoring or resource limits  
- ❌ Network traffic inspection
- ❌ Server marketplace features

**Think of it like `nvm` for Node.js** - it just makes MCP work consistently everywhere.

## Usage Examples

### Installing and Running a Server

```bash
# Install a server from npm
mcp install @modelcontextprotocol/server-filesystem

# Or install from a local directory
mcp install ./my-local-server

# Run it without platform-specific setup
mcp run server-filesystem
```

### Managing Configurations

```bash
# Add a server with custom environment variables
mcp config add my-server --env API_KEY=secret --env PORT=3000

# List all configured servers
mcp config list

# Show details for a specific server
mcp config show my-server

# Update server configuration
mcp config update my-server --env API_KEY=new-secret
```

### Creating a New MCP Server Project

```bash
# Initialize a new MCP server project
mcp init my-new-server
cd my-new-server

# Generate IDE configurations
mcp generate-ide-config

# Your project now has:
# - Cross-platform npm scripts
# - Proper .gitignore
# - VS Code settings that work for everyone
```

## Platform-Specific Notes

### Windows

- Automatically uses `cmd.exe` for proper npx execution
- Converts forward slashes to backslashes where needed
- Handles spaces in paths without manual escaping

### macOS  

- Sources shell profile for GUI applications
- Respects `~/Library/Application Support/Claude/` location
- Handles both Intel and Apple Silicon architectures

### Linux

- Follows XDG Base Directory specification
- Works with both `.deb` and `.rpm` based distributions
- Handles both X11 and Wayland environments

## Troubleshooting

### Command Not Found

Run `mcp doctor` - it will diagnose and offer to fix PATH issues.

### Permission Denied

```bash
# On macOS/Linux, you may need to make it executable
chmod +x ~/.local/bin/mcp
```

### Config File Not Found

MCP Helper will create the config file if it doesn't exist. Run `mcp config list` to initialize.

## Current Implementation Status

🚀 **Phase 1 Complete: Core Runner**

The `mcp run` command is now fully functional with:
- ✅ Cross-platform npx wrapper (Windows/macOS/Linux)
- ✅ Automatic path separator normalization
- ✅ Intelligent error messages with actionable fixes
- ✅ Comprehensive test coverage

### Using the Run Command

```bash
# Run any npm-based MCP server
mcp run @modelcontextprotocol/server-filesystem

# Pass arguments to the server
mcp run my-server --port 3000 --config ./config.json

# Enable verbose output for debugging
mcp run my-server --verbose
```

The run command automatically:
- Detects your operating system
- Uses the correct npx command (npx.cmd on Windows)
- Normalizes path separators in arguments
- Provides helpful error messages if something goes wrong

### Coming Soon

- `mcp install` - Smart package installation
- `mcp setup` - One-time system configuration
- `mcp config` - Server configuration management
- `mcp doctor` - System diagnostics and auto-fixes

## Performance

The project uses several optimizations for fast builds:

- **Optimized CI caching** - Builds complete in ~1-2 minutes
- **Enhanced Cargo caching** - Dependencies cached across all crates
- **cargo-binstall** - CI tools install in seconds instead of minutes
- **Parallel job execution** - Tests run concurrently on all platforms
- **Minimal dependencies** - Fast compilation with only essential crates

Run `scripts/benchmark-build.sh` to measure build performance locally.

## Development Setup

### Git Hooks

This project uses [rusty-hook](https://github.com/swellaby/rusty-hook) to ensure code quality. Git hooks are automatically installed when you run tests or build the project.

#### Pre-commit Hooks
Before each commit, the following checks run automatically:
- `make fmt-check` - Ensures code is properly formatted
- `make lint` - Runs clippy to catch common mistakes

These quick checks ensure code quality without slowing down your workflow.

#### Manual Hook Management
```bash
# Install/update git hooks
make hooks

# Run pre-commit checks manually
make pre-commit
```

If a hook fails, you can debug by running the specific make target that failed.

## Contributing

We welcome contributions! The codebase is focused and approachable:

```bash
# Clone the repository
git clone https://github.com/mcp-helper/mcp-helper
cd mcp-helper

# Run tests on your platform
cargo test

# Build locally
cargo build --release

# Run your local build
./target/release/mcp --help
```

### Development Principles

1. **Platform differences are bugs** - If it works differently on different platforms, it's a bug
2. **Errors should be actionable** - Every error should tell users how to fix it
3. **Zero configuration** - It should work out of the box
4. **No scope creep** - We make MCP work cross-platform, nothing else

### Testing

We test on:

- Windows 10/11 (cmd, PowerShell, Git Bash)
- macOS 12+ (Intel and Apple Silicon)
- Ubuntu 20.04+ and other major Linux distributions

## Architecture

Built with Rust for:

- Single binary distribution
- Excellent error messages
- Memory safety when handling configs
- Fast startup time
- Trust from the developer community

Key dependencies:

- `clap` - CLI argument parsing
- `dirs` - Cross-platform directory detection
- `which` - Executable detection
- `serde_json` - Config file handling
- `thiserror` - Error management

## License

MIT License - see [LICENSE](LICENSE) file

## Acknowledgments

- The [Model Context Protocol](https://modelcontextprotocol.io) team for creating MCP
- The Rust CLI working group for excellent libraries and patterns
- Early adopters who reported cross-platform issues

---

<div align="center">

**Stop debugging platform issues. Start building MCP servers.**

[Report an Issue](https://github.com/mcp-helper/mcp-helper/issues) • [Join Discord](https://discord.gg/mcp-helper) • [Read the Docs](https://mcp-helper.dev)

</div>
