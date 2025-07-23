# MCP Install Command - Detailed Requirements

## Overview

The `mcp install <server>` command is responsible for adding MCP server configurations to various MCP clients (Claude Desktop, Cursor, VSCode, Windsurf) while handling diverse server types and their dependencies.

## Core Functionality

### What `mcp install` Does

1. **Analyzes server requirements** - Determines server type (npm, binary, Python, Docker)
2. **Checks dependencies** - Verifies required runtimes and versions
3. **Configures MCP clients** - Adds appropriate configuration to detected clients
4. **Handles platform differences** - Ensures cross-platform compatibility

### What `mcp install` Does NOT Do

- Does not download npm packages (npx handles this on-demand)
- Does not manage server runtime lifecycle
- Does not modify system PATH permanently
- Does not install system dependencies automatically (unless --auto-install-deps flag is used)

## Server Types

### 1. NPM/Node.js Servers (Most Common)
- **Example**: `@modelcontextprotocol/server-filesystem`
- **Requirement**: Node.js v18+
- **Command**: `npx -y <package>`
- **Detection**: Package name starts with @ or contains /

### 2. Binary Servers (Rust, Go, etc.)
- **Example**: `rust-docs-mcp-server`
- **Requirement**: Platform-specific binary
- **Command**: Direct binary path
- **Handling**: Download appropriate binary for platform

### 3. Python Servers
- **Example**: `mcp-server-fetch`
- **Requirement**: Python 3.10+
- **Command**: `python -m <module>` or `uvx <package>`
- **Detection**: Check server metadata/registry

### 4. Docker Servers
- **Example**: `docker:mycompany/mcp-server`
- **Requirement**: Docker installed and running
- **Command**: `docker run ...`
- **Detection**: Prefix with `docker:`

## MCP Client Support

### Claude Desktop
- **Config Location**:
  - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
  - Linux: `~/.config/Claude/claude_desktop_config.json`
- **Format**: 
  ```json
  {
    "mcpServers": {
      "name": {
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem"],
        "env": {}
      }
    }
  }
  ```

### Cursor
- **Config Location**:
  - Global: `~/.cursor/mcp.json`
  - Project: `.cursor/mcp.json`
- **Format**:
  ```json
  {
    "servers": {
      "name": {
        "command": "npx",
        "args": ["-y", "@modelcontextprotocol/server-filesystem"],
        "type": "stdio"
      }
    }
  }
  ```

### VSCode
- **Config Location**: `~/.vscode/mcp.json`
- **Requirement**: GitHub Copilot must be enabled
- **Note**: Only works in Agent mode

### Windsurf
- **Config Location**: `~/.codeium/windsurf/mcp_config.json`
- **Note**: Uses `serverUrl` instead of `url` for SSE connections

## Dependency Management

### Detection Strategy

1. **Check for required runtime** (Node.js, Python, Docker)
2. **Verify version meets requirements**
3. **Provide platform-specific installation instructions**
4. **Offer alternatives when possible**

### Node.js Dependencies

```bash
# Required version detected from package.json or defaults to v18+
# Check with: node --version

# Missing Node.js example:
❌ Node.js not found (required: v18+)

To install Node.js on Windows:
  → Using winget: winget install OpenJS.NodeJS
  → Using Chocolatey: choco install nodejs
  → Download from: https://nodejs.org/

To install Node.js on macOS:
  → Using Homebrew: brew install node
  → Using MacPorts: sudo port install nodejs18
  → Download from: https://nodejs.org/

To install Node.js on Linux:
  → Using apt: sudo apt install nodejs npm
  → Using dnf: sudo dnf install nodejs
  → Using snap: sudo snap install node --classic
```

### Python Dependencies

```bash
# Check with: python --version or python3 --version

# Version mismatch example:
⚠️  Python 3.8.0 found (required: 3.10+)

This server requires Python 3.10 or higher.

To upgrade Python:
  → Using pyenv: pyenv install 3.10.0 && pyenv global 3.10.0
  → Using Homebrew: brew upgrade python@3.10
  → Download latest from: https://python.org/
```

### Docker Dependencies

```bash
# Check with: docker --version

# Missing Docker example:
❌ Docker not found!

To use Docker-based MCP servers:
1. Install Docker Desktop from https://docker.com
2. Start Docker
3. Run this command again

Alternative: Look for a non-Docker version of this server
```

## Installation Flow

### Basic Flow

```bash
$ mcp install @modelcontextprotocol/server-filesystem

🔍 Analyzing server requirements...
✓ Type: NPM package
✓ Requires: Node.js v18+ (found: v20.11.0)

📋 This server needs configuration:
- Allowed directories: ~/Documents,~/Downloads

🎯 Install to which clients?
✓ Claude Desktop
✓ Cursor
✗ VSCode (GitHub Copilot required)
✓ Windsurf

✅ Added to 3 client configs
🔄 Please restart your MCP clients
```

### Binary Server Flow

```bash
$ mcp install rust-docs-mcp-server

🔍 Analyzing server requirements...
✓ Type: Binary (Rust)
🖥️  Platform: macOS ARM64

📥 Downloading binary...
✓ Downloaded: rust-docs-mcp-server-darwin-arm64
✓ Verified checksum
✓ Installed to: ~/.mcp/bin/rust-docs-mcp-server

🎯 Adding to clients...
✅ Configuration added
```

### Missing Dependencies Flow

```bash
$ mcp install complex-ml-server

🔍 Checking dependencies...
❌ Multiple issues found:
   - Python 3.8 installed (requires 3.10+)
   - CUDA not found (required for GPU support)

Suggestions:
1. This server has lighter alternatives:
   - simple-ml-server (CPU only, Python 3.8+)
   - ml-server-lite (no GPU required)

2. Or address the issues:
   - Upgrade Python: pyenv install 3.10.0
   - Install CUDA: https://nvidia.com/cuda

What would you like to do?
```

## Configuration Generation

### NPM Server Configuration

```json
{
  "command": "npx",
  "args": [
    "-y",
    "@modelcontextprotocol/server-filesystem",
    "/Users/username/Documents",
    "/Users/username/Downloads"
  ]
}
```

### Binary Server Configuration

```json
{
  "command": "/Users/username/.mcp/bin/rust-docs-mcp-server",
  "args": [],
  "env": {
    "RUST_LOG": "info"
  }
}
```

### Python Server Configuration

```json
{
  "command": "python",
  "args": [
    "-m",
    "mcp_server_fetch"
  ],
  "env": {
    "PYTHONPATH": "/path/to/modules"
  }
}
```

### Docker Server Configuration

```json
{
  "command": "docker",
  "args": [
    "run",
    "-i",
    "--rm",
    "mycompany/mcp-server:latest"
  ]
}
```

## Error Handling

### Dependency Errors

```rust
enum InstallError {
    MissingDependency { 
        dep: String, 
        required_version: String,
        install_instructions: Vec<String> 
    },
    VersionMismatch {
        dep: String,
        found: String,
        required: String,
        upgrade_instructions: Vec<String>
    },
    PlatformNotSupported { 
        server: String, 
        platform: Platform,
        alternatives: Vec<String>
    },
    ServerNotFound { 
        name: String,
        suggestions: Vec<String>
    },
    ConfigurationRequired { 
        fields: Vec<ConfigField> 
    },
}
```

### Recovery Strategies

1. **Missing Dependencies**: Provide install instructions, suggest alternatives
2. **Version Mismatches**: Offer upgrade paths, allow proceeding with warning
3. **Platform Issues**: Suggest platform-specific alternatives
4. **Configuration Errors**: Interactive prompts for required fields

## Command Line Options

```bash
# Basic installation
mcp install @modelcontextprotocol/server-filesystem

# Target specific client
mcp install --client claude @modelcontextprotocol/server-filesystem

# Auto-install dependencies (where possible)
mcp install --auto-install-deps mcp-server-python

# Force installation despite warnings
mcp install --force old-server

# Specify configuration values
mcp install --config "dirs=/home,/tmp" server-filesystem

# Dry run (show what would be done)
mcp install --dry-run @modelcontextprotocol/server-github
```

## Implementation Architecture

```
src/
├── client/                    # MCP client abstractions
│   ├── mod.rs                # Client trait & detection
│   ├── claude_desktop.rs     # Claude Desktop implementation
│   ├── cursor.rs             # Cursor implementation
│   ├── vscode.rs             # VSCode implementation
│   └── windsurf.rs           # Windsurf implementation
├── server/                   # MCP server handling
│   ├── mod.rs               # Server trait & detection
│   ├── npm.rs               # NPM package handling
│   ├── binary.rs            # Binary download & verification
│   ├── python.rs            # Python package handling
│   ├── docker.rs            # Docker container handling
│   └── metadata.rs          # Server metadata/requirements
├── deps/                     # Dependency management
│   ├── mod.rs               # Main dependency checker
│   ├── node.rs              # Node.js version detection
│   ├── python.rs            # Python version detection
│   ├── docker.rs            # Docker detection
│   ├── installers.rs        # Platform-specific install commands
│   └── version.rs           # Version comparison utilities
├── package/                  # Package management
│   ├── mod.rs               # Package manager abstraction
│   ├── resolver.rs          # Resolve server metadata
│   └── registry.rs          # Server registry interaction
```

## Success Criteria

### Phase 1 - Basic NPM Support
- ✅ Install NPM-based servers to Claude Desktop
- ✅ Detect missing Node.js with helpful errors
- ✅ Handle basic version requirements
- ✅ Platform-specific path normalization

### Phase 2 - Multi-Client & Binary Support
- ✅ Support Cursor, VSCode, Windsurf configs
- ✅ Download and install binary servers
- ✅ Handle Python server requirements
- ✅ Client auto-detection

### Phase 3 - Advanced Features
- ✅ Docker server support
- ✅ Server registry/metadata integration
- ✅ Auto-dependency installation
- ✅ Alternative server suggestions

## Testing Requirements

### Unit Tests
- Dependency version parsing and comparison
- Configuration generation for each client
- Platform-specific path handling
- Error message generation

### Integration Tests
- NPM server installation flow
- Binary download and verification
- Multi-client configuration updates
- Dependency detection accuracy

### Platform Tests
- Windows: npx.cmd handling, path separators
- macOS: Homebrew integration, Apple Silicon
- Linux: Distribution-specific package managers

## Future Considerations

1. **Desktop Extensions (.dxt)**: Self-contained server bundles
2. **Server Discovery**: Search/browse available servers
3. **Update Management**: Update installed servers
4. **Dependency Caching**: Cache downloaded binaries
5. **Rollback Support**: Undo configuration changes
6. **Server Verification**: Validate server functionality post-install