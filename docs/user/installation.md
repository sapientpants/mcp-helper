# MCP Helper Installation Guide

MCP Helper is a cross-platform tool that makes MCP servers "just work" on Windows, macOS, and Linux by eliminating compatibility issues and providing universal server management.

## Quick Start

### 1. Install MCP Helper

**From Binary Releases (Recommended)**
```bash
# Download the latest release for your platform from:
# https://github.com/sapientpants/mcp-helper/releases

# macOS/Linux - make executable
chmod +x mcp
sudo mv mcp /usr/local/bin/

# Windows - place mcp.exe in your PATH
```

**From Source**
```bash
# Requires Rust 1.88.0+
git clone https://github.com/sapientpants/mcp-helper.git
cd mcp-helper
cargo build --release
# Binary will be in target/release/mcp
```

### 2. Verify Installation

```bash
mcp --version
# Should output: mcp 0.1.0
```

## Installing Your First MCP Server

### Example 1: File System Server

The most common MCP server provides file system access:

```bash
# Install the official filesystem server
mcp install @modelcontextprotocol/server-filesystem

# The tool will:
# ✓ Check for Node.js (required dependency)
# ✓ Validate server source security
# ✓ Detect your MCP clients (Claude Desktop, etc.)
# ✓ Prompt for configuration
# ✓ Update client configs automatically
```

**Configuration Prompts:**
- `allowedDirectories`: Directories the server can access (required)
- `allowedFileTypes`: File extensions to allow (optional)

**Example:**
```
✓ Installing MCP server: @modelcontextprotocol/server-filesystem
→ Security validation passed
→ Found Node.js v20.11.0
→ Detected MCP clients: Claude Desktop
→ Enter allowed directories (comma-separated): /Users/yourname/Documents,/Users/yourname/Projects
→ Enter allowed file types (optional, e.g., .txt,.md): .md,.txt,.json,.py
→ Installing to Claude Desktop...
✓ Successfully installed @modelcontextprotocol/server-filesystem to 1 client(s)
```

### Example 2: Docker Server

```bash
# Install a Docker-based MCP server
mcp install docker:nginx:alpine

# Configuration options:
# - ports: Host:container port mappings
# - volumes: Host:container volume mounts
# - environment: Environment variables
```

### Example 3: GitHub Repository Server

```bash
# Install directly from GitHub
mcp install anthropic/mcp-server-sqlite

# Or with a specific version/branch
mcp install anthropic/mcp-server-sqlite@v1.0.0
```

## Advanced Installation Options

### Batch Installation

Create a `servers.txt` file:
```
@modelcontextprotocol/server-filesystem
@anthropic/mcp-server-slack
docker:postgres:13
```

Install all at once:
```bash
mcp install --batch servers.txt
```

### Non-Interactive Installation

Skip prompts with configuration flags:
```bash
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="/home/user/docs,/home/user/projects" \
  --config allowedFileTypes=".md,.txt,.json"
```

### Dependency Auto-Installation

Automatically install missing dependencies:
```bash
mcp install @modelcontextprotocol/server-filesystem --auto-install-deps

# Will automatically install Node.js if missing
# Prompts for confirmation before installing
```

### Dry Run Mode

See what would happen without making changes:
```bash
mcp install @modelcontextprotocol/server-filesystem --dry-run

# Shows:
# - Security validation results
# - Dependencies that would be checked/installed
# - Configuration that would be applied
# - Clients that would be updated
```

## Server Types Supported

### 1. NPM Packages
```bash
# Official packages
mcp install @modelcontextprotocol/server-filesystem

# Scoped packages
mcp install @anthropic/mcp-server-slack

# Specific versions
mcp install express-mcp-server@1.2.3
```

### 2. Docker Images
```bash
# Official images
mcp install docker:nginx

# With specific tags
mcp install docker:postgres:13

# With configuration
mcp install docker:postgres:13 \
  --config ports="5432:5432" \
  --config environment="POSTGRES_PASSWORD=secret,POSTGRES_DB=mydb" \
  --config volumes="/data:/var/lib/postgresql/data"
```

### 3. GitHub Repositories
```bash
# Public repositories
mcp install username/repo-name

# Specific branches/tags
mcp install username/repo-name@main
mcp install username/repo-name@v1.0.0

# Full GitHub URLs also work
mcp install https://github.com/username/repo-name
```

### 4. Binary Releases
```bash
# Download and install binary releases
mcp install https://github.com/org/project/releases/download/v1.0/server-linux-x64

# Automatically detects platform-appropriate binaries
```

## Client Support

MCP Helper supports multiple MCP clients:

### Claude Desktop (Primary)
- **Auto-detected** on all platforms
- **Config location**: 
  - Windows: `%APPDATA%\Claude\claude_desktop_config.json`
  - macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`  
  - Linux: `~/.config/Claude/claude_desktop_config.json`

### Other Supported Clients
- **Cursor**: Global and project-level configurations
- **VS Code**: With GitHub Copilot integration
- **Windsurf**: Codeium-based MCP client
- **Claude Code**: CLI-based client

## Security Features

MCP Helper includes comprehensive security validation:

### Trusted Sources
- **NPM Registry**: `npmjs.org` packages are trusted
- **GitHub**: `github.com` repositories are trusted  
- **Docker Hub**: Official Docker images are trusted

### Security Warnings
```bash
# Example warning for untrusted source
⚠ Security warnings detected:
  • Domain 'untrusted.com' is not in the list of trusted sources. Proceed with caution.
  
Do you want to proceed despite these warnings? [y/N]
```

### Blocked Packages
Automatically blocks dangerous packages:
- System command names (`rm`, `sudo`, `admin`)
- Path traversal patterns (`../`, `..\\`)
- Suspicious short names (potential typosquatting)

## Configuration Management

### View Installed Servers
```bash
# List all installed servers (planned feature)
mcp config list

# Show specific server details (planned feature)  
mcp config show server-name
```

### Update Server Configuration
```bash
# Update existing server (planned feature)
mcp config update server-name --config newSetting=value
```

### Remove Servers
```bash
# Remove server from clients (planned feature)
mcp config remove server-name
```

## Troubleshooting

### Common Issues

**"Node.js not found"**
```bash
# Install Node.js first, then retry
# Windows: winget install OpenJS.NodeJS  
# macOS: brew install node
# Linux: apt install nodejs npm (Ubuntu/Debian)

# Or use auto-install
mcp install server-name --auto-install-deps
```

**"npx not available"**
- Usually resolved by installing/updating Node.js
- On Windows, ensure both `node` and `npx` commands work

**"Docker not running"**
```bash
# Start Docker Desktop (Windows/macOS)
# Or start Docker daemon (Linux)
sudo systemctl start docker
```

**"Permission denied"**
- On Linux/macOS, some operations may require `sudo`
- MCP Helper will prompt when elevated privileges are needed

### Verbose Mode

Get detailed information about what's happening:
```bash
mcp install server-name --verbose

# Shows:
# - Platform detection
# - Dependency checks with versions
# - Security validation details  
# - Configuration steps
# - Client update process
```

### Getting Help

```bash
# Show all available commands
mcp --help

# Get help for specific commands
mcp install --help
mcp config --help  # (planned)
```

## What's Next?

After installing servers:

1. **Restart your MCP client** (Claude Desktop, etc.)
2. **Verify the server appears** in your client's server list
3. **Test server functionality** by using its features
4. **Check logs** if something isn't working (use `--verbose`)

## Examples by Use Case

### Web Development
```bash
# File system access for code editing
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="$HOME/projects" \
  --config allowedFileTypes=".js,.ts,.json,.md"

# Database access
mcp install docker:postgres:13 \
  --config ports="5432:5432" \
  --config environment="POSTGRES_PASSWORD=dev"
```

### Data Analysis  
```bash
# SQLite database server
mcp install @anthropic/mcp-server-sqlite

# CSV/data file access
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="$HOME/data" \
  --config allowedFileTypes=".csv,.json,.xlsx"
```

### Research & Documentation
```bash
# File system for document access
mcp install @modelcontextprotocol/server-filesystem \
  --config allowedDirectories="$HOME/Documents,/Users/Shared/Research" \
  --config allowedFileTypes=".md,.txt,.pdf,.docx"

# Web scraping capabilities (hypothetical)
mcp install research-tools/web-scraper
```

This installation guide provides everything users need to get started with MCP Helper and understand its capabilities!